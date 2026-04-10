#!/usr/bin/env bash
set -euo pipefail

# run_difftest.sh — run differential tests using separate-binary approach.
#
# Compiles test against C library, runs it, captures stdout.
# Compiles test against Rust library, runs it, captures stdout + stderr.
# Reports three failure types with structured info:
#   1. Compile errors — exact compiler output with file:line
#   2. Panics — exact backtrace with function:file:line
#   3. Output divergences — differing lines from both outputs
#
# Also includes a function→file map so the AI can look up source locations.
#
# Contract: takes $1 = output report path.
# Uses env vars: DIFFGEN_WORKDIR, RUST_DIR, C_SRC_DIRS, C_INCLUDE_DIRS, CC, EXP_DIR

REPORT="${1:?Usage: run_difftest.sh <report_path>}"

CC="${CC:-clang-21}"
DIFFTEST="${DIFFGEN_WORKDIR}/difftest_suite.c"
BRIDGE="${DIFFGEN_WORKDIR}/test_bridge.c"
INCDIR="${C_INCLUDE_DIRS}"

[ -f "$DIFFTEST" ] || { echo "Error: $DIFFTEST not found" >&2; exit 1; }

BDIR=$(mktemp -d)
trap 'rm -rf "$BDIR"' EXIT

INC_FLAGS=""
for d in $INCDIR; do INC_FLAGS="$INC_FLAGS -I$d"; done
# Also include diffgen dir for test_bridge.h
INC_FLAGS="$INC_FLAGS -I${DIFFGEN_WORKDIR}"

# ── Use pre-built function→file map ──
FUNC_MAP="${WORK_DIR:-${EXP_DIR}}/work-func-map.txt"
[ -f "$FUNC_MAP" ] || { echo "Error: function map not found at $FUNC_MAP" >&2; exit 1; }

# ── Build C library ──
echo "Building C library..."
for d in $C_SRC_DIRS; do
    [ -d "$d" ] || continue
    find "$d" -name '*.c' -type f | grep -v fenv.c | while read -r cfile; do
        $CC $INC_FLAGS -c "$cfile" -o "${BDIR}/c_$(basename "$cfile" .c).o" 2>/dev/null || true
    done
done
ar rcs "${BDIR}/libc_impl.a" "${BDIR}"/c_*.o 2>/dev/null
rm -f "${BDIR}"/c_*.o

# Compile bridge if it exists
BRIDGE_OBJ=""
if [ -f "$BRIDGE" ]; then
    $CC $INC_FLAGS -c "$BRIDGE" -o "${BDIR}/bridge.o" 2>/dev/null && \
        BRIDGE_OBJ="${BDIR}/bridge.o" || true
fi

# ── Build Rust library ──
echo "Building Rust library..."
RUST_TARGET="${BDIR}/rust_target"
RUST_BUILD_ERR=""
if ! cargo build --release --lib \
    --manifest-path "${RUST_DIR}/Cargo.toml" \
    --target-dir "$RUST_TARGET" 2>"${BDIR}/rust_build_err.txt"; then
    RUST_BUILD_ERR=$(cat "${BDIR}/rust_build_err.txt")
fi
RUST_LIB=$(find "${RUST_TARGET}/release" -name '*.a' -type f 2>/dev/null | head -1)

# ── Check which functions Rust implements vs falls back to C ──
RUST_SYMS="${BDIR}/rust_symbols.txt"
C_SYMS="${BDIR}/c_symbols.txt"
MISSING_IMPL="${BDIR}/missing_impl.txt"
if [ -n "$RUST_LIB" ]; then
    nm "$RUST_LIB" 2>/dev/null | grep " T " | awk '{print $3}' | sort -u > "$RUST_SYMS"
    nm "${BDIR}/libc_impl.a" 2>/dev/null | grep " T " | awk '{print $3}' | sort -u > "$C_SYMS"
    # Functions in C but not in Rust = will fall back to C
    # Functions in C but not in Rust = will fall back to C
    comm -23 "$C_SYMS" "$RUST_SYMS" > "$MISSING_IMPL"
fi

# ── Compile + run against C library ──
echo "Compiling test against C library..."
C_COMPILE_ERR=""
if $CC $INC_FLAGS -Wno-implicit-function-declaration \
    "$DIFFTEST" $BRIDGE_OBJ "${BDIR}/libc_impl.a" \
    -lm -o "${BDIR}/test_c" 2>"${BDIR}/c_compile_err.txt"; then
    echo "Running C test..."
    stdbuf -oL timeout 600 "${BDIR}/test_c" > "${BDIR}/c_out.txt" 2>"${BDIR}/c_stderr.txt" || true
else
    C_COMPILE_ERR=$(cat "${BDIR}/c_compile_err.txt")
fi

# ── Compile + run against Rust library ──
R_COMPILE_ERR=""
R_PANIC=""
if [ -n "$RUST_BUILD_ERR" ]; then
    R_COMPILE_ERR="Rust cargo build failed:
${RUST_BUILD_ERR}"
elif [ -z "$RUST_LIB" ]; then
    R_COMPILE_ERR="No .a produced by cargo build"
else
    echo "Compiling test against Rust library..."
    # Rust binary: link with Rust lib ONLY. No C fallback, no C bridge.
    # The Rust lib provides its own bridge exports via test_bridge.rs.
    if $CC $INC_FLAGS -Wno-implicit-function-declaration \
        "$DIFFTEST" "$RUST_LIB" \
        -lm -lpthread -ldl \
        -o "${BDIR}/test_r" 2>"${BDIR}/r_compile_err.txt"; then
        echo "Running Rust test..."
        export RUST_BACKTRACE=1
        R_EXIT=0
        stdbuf -oL timeout 600 "${BDIR}/test_r" > "${BDIR}/r_out.txt" 2>"${BDIR}/r_stderr.txt" || R_EXIT=$?

        # Detect timeout, crash, or panic
        if [ "$R_EXIT" -eq 124 ]; then
            LAST_LINE=$(tail -1 "${BDIR}/r_out.txt" 2>/dev/null || true)
            LAST_FUNC=$(echo "$LAST_LINE" | awk '{print $1}')
            R_PANIC="TIMEOUT: Rust binary hung (likely infinite loop/recursion in '${LAST_FUNC}' or the function after it in test order).
Last output line: ${LAST_LINE}"
        elif [ "$R_EXIT" -gt 128 ]; then
            SIG=$((R_EXIT - 128))
            LAST_LINE=$(tail -1 "${BDIR}/r_out.txt" 2>/dev/null || true)
            LAST_FUNC=$(echo "$LAST_LINE" | awk '{print $1}')
            R_PANIC="CRASH: Rust binary killed by signal ${SIG} ($(kill -l $SIG 2>/dev/null || echo unknown)) near function '${LAST_FUNC}'.
Last output line: ${LAST_LINE}"
            [ -s "${BDIR}/r_stderr.txt" ] && R_PANIC="${R_PANIC}
Stderr: $(cat "${BDIR}/r_stderr.txt")"
        elif grep -q "panicked\|SIGSEGV\|signal: 11\|Aborted" "${BDIR}/r_stderr.txt" 2>/dev/null; then
            R_PANIC=$(cat "${BDIR}/r_stderr.txt")
        fi
    else
        R_COMPILE_ERR=$(cat "${BDIR}/r_compile_err.txt")
    fi
fi

# ── Save raw outputs ──
OUTPUTS_DIR="$(dirname "$REPORT")"
[ -f "${BDIR}/c_out.txt" ] && cp "${BDIR}/c_out.txt" "${OUTPUTS_DIR}/c_output.txt"
[ -f "${BDIR}/r_out.txt" ] && cp "${BDIR}/r_out.txt" "${OUTPUTS_DIR}/r_output.txt"
[ -f "${BDIR}/r_stderr.txt" ] && cp "${BDIR}/r_stderr.txt" "${OUTPUTS_DIR}/r_stderr.txt"
[ -f "${BDIR}/c_stderr.txt" ] && cp "${BDIR}/c_stderr.txt" "${OUTPUTS_DIR}/c_stderr.txt"

# ── Generate structured report ──
{
    echo "============================================================"
    echo "Differential Test Report (C vs Rust)"
    echo "Generated: $(date '+%Y-%m-%d %H:%M')"
    echo "============================================================"
    echo ""

    TOTAL_FAILURES=0

    # ── Section 1: Compile errors ──
    if [ -n "$C_COMPILE_ERR" ]; then
        echo "## COMPILE ERROR (C)"
        echo ""
        echo "$C_COMPILE_ERR"
        echo ""
        TOTAL_FAILURES=1
    fi
    if [ -n "$R_COMPILE_ERR" ]; then
        echo "## COMPILE ERROR (Rust)"
        echo ""
        echo "$R_COMPILE_ERR"
        echo ""
        TOTAL_FAILURES=1
    fi

    # ── Section 2: Panics ──
    if [ -n "$R_PANIC" ]; then
        echo "## RUNTIME PANIC"
        echo ""
        echo "The Rust test binary panicked or crashed."
        echo "Backtrace:"
        echo "$R_PANIC"
        echo ""
        TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
    fi

    # ── Section 3: Output divergences ──
    if [ -f "${BDIR}/c_out.txt" ] && [ -f "${BDIR}/r_out.txt" ]; then
        C_LINES=$(wc -l < "${BDIR}/c_out.txt")
        R_LINES=$(wc -l < "${BDIR}/r_out.txt")

        echo "## OUTPUT COMPARISON"
        echo ""
        echo "C test output:    ${C_LINES} lines"
        echo "Rust test output: ${R_LINES} lines"
        echo ""

        if [ "$R_LINES" -lt "$C_LINES" ]; then
            echo "WARNING: Rust produced fewer output lines (${R_LINES} vs ${C_LINES})."
            echo "Likely cause: crash or missing functions. Lines after crash are missing."
            echo ""
        fi

        # Key-based comparison
        _SCRIPTS="${HARNESS_DIR:-${EXP_DIR}}/scripts"
        python3 "${_SCRIPTS}/compare_outputs.py" \
            "${BDIR}/c_out.txt" "${BDIR}/r_out.txt" 2>/dev/null || \
            echo "(comparison script failed — raw diff follows)"

        TOTAL_FAILURES=$(python3 -c "
import sys
sys.path.insert(0, '${_SCRIPTS}')
from compare_outputs import compare
m, mm, faults, cc, rc = compare('${BDIR}/c_out.txt', '${BDIR}/r_out.txt')
print(len(m) + len(mm))
" 2>/dev/null || echo 0)
    elif [ -f "${BDIR}/c_out.txt" ]; then
        C_LINES=$(wc -l < "${BDIR}/c_out.txt")
        echo "## OUTPUT COMPARISON"
        echo ""
        echo "C test output:    ${C_LINES} lines"
        echo "Rust test output: 0 lines (no output — binary crashed immediately or not built)"
        echo ""
        echo "MISSING (all ${C_LINES} tests):"
        echo "Rust binary produced no output at all."
        TOTAL_FAILURES=$C_LINES
    fi

    # ── Section 4: Missing implementations ──
    if [ -f "$MISSING_IMPL" ] && [ -s "$MISSING_IMPL" ]; then
        N_MISSING=$(wc -l < "$MISSING_IMPL")
        echo ""
        echo "## NOT IMPLEMENTED IN RUST (${N_MISSING} functions)"
        echo ""
        echo "These C functions have no Rust implementation. The Rust test binary"
        echo "falls back to the C version for these, so they won't show as divergences."
        echo "They need to be implemented in Rust:"
        echo ""
        while read -r sym; do
            loc=$(grep "^${sym} -> " "$FUNC_MAP" 2>/dev/null | head -1 || true)
            if [ -n "$loc" ]; then
                echo "  ${loc}"
            else
                echo "  ${sym} -> (unknown source file)"
            fi
        done < "$MISSING_IMPL"
        # Note: unimplemented functions fall back to C, so they pass tests
        # but aren't truly tested. Don't add to TOTAL_FAILURES.
    fi

    # ── Function → file map ──
    echo ""
    echo "## FUNCTION LOCATION MAP"
    echo ""
    echo "Use this to find where each function is defined (do NOT read all files):"
    echo ""
    cat "$FUNC_MAP"

    # ── Summary ──
    echo ""
    echo "============================================================"
    echo "Tests failed:     ${TOTAL_FAILURES}"
    if [ -f "$MISSING_IMPL" ] && [ -s "$MISSING_IMPL" ]; then
        echo "Not in Rust:      $(wc -l < "$MISSING_IMPL") (tested against C fallback)"
    fi
    echo "============================================================"
} > "$REPORT"

cat "$REPORT"

[ "${TOTAL_FAILURES}" -eq 0 ] && exit 0 || exit 1

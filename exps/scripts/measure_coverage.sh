#!/usr/bin/env bash
set -euo pipefail

# measure_coverage.sh — measure function and branch coverage of a test suite
# against the C library. Uses clang-21 + llvm-cov-21.
#
# Links all .o files directly (NOT via archive) so all library branches
# are counted regardless of whether the test calls them.
#
# Usage: measure_coverage.sh <test_suite.c> [output_dir]
#
# Requires env vars or defaults:
#   CC=clang-21, LLVM_PROFDATA=llvm-profdata-21, LLVM_COV=llvm-cov-21
#   C_SRC_DIRS, C_INCLUDE_DIRS (from common.sh or defaults)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXP_DIR="$(dirname "$SCRIPT_DIR")"
LIBMCS="${LIBMCS:-${TEST_CASE_DIR:-}}"

TEST_SUITE="${1:?Usage: measure_coverage.sh <test_suite.c> [output_dir]}"
OUTPUT_DIR="${2:-}"

CC="${CC:-clang-21}"
LLVM_PROFDATA="${LLVM_PROFDATA:-llvm-profdata-21}"
LLVM_COV="${LLVM_COV:-llvm-cov-21}"

: "${C_SRC_DIRS:=${LIBMCS}/mathd ${LIBMCS}/mathf ${LIBMCS}/common ${LIBMCS}/complexd ${LIBMCS}/complexf}"
: "${C_INCLUDE_DIRS:=${LIBMCS}/include}"

# Files to exclude (doc-only, no compiled code)
EXCLUDE_FILES="cmplx.c|isfinite.c|isgreater.c|isgreaterequal.c|isinf.c|isless.c|islessequal.c|islessgreater.c|isnan.c|isnormal.c|isunordered.c|fenv.c"

BDIR=$(mktemp -d)
trap 'rm -rf "$BDIR"' EXIT

# Build include flags
INC_FLAGS=""
for d in $C_INCLUDE_DIRS; do INC_FLAGS="$INC_FLAGS -I$d"; done

# Coverage compile flags (match Makefile: -O0 -fno-builtin)
COV_FLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -fno-builtin"

echo "Measuring coverage for: $(basename "$TEST_SUITE")"
echo "  CC: $CC"

# ── Step 1: Compile all library .c files with coverage ──
echo "  Compiling library..."
OBJ_FILES=""
for d in $C_SRC_DIRS; do
    [ -d "$d" ] || continue
    find "$d" -name '*.c' -type f | grep -vE "$EXCLUDE_FILES" | while read -r cfile; do
        oname="${BDIR}/c_$(basename "$(dirname "$cfile")")_$(basename "$cfile" .c).o"
        $CC $INC_FLAGS $COV_FLAGS -c "$cfile" -o "$oname" 2>/dev/null || true
    done
done
OBJ_FILES=$(find "$BDIR" -name 'c_*.o' -type f | sort)
OBJ_COUNT=$(echo "$OBJ_FILES" | wc -l)
echo "  Compiled $OBJ_COUNT object files"

# ── Step 2: Compile bridge if present ──
BRIDGE_OBJ=""
TEST_DIR="$(dirname "$TEST_SUITE")"
# Search for bridge in test dir, parent dir, or harness/
BRIDGE_DIR=""
for _bd in "$TEST_DIR" "$(dirname "$TEST_DIR")" "${EXP_DIR}"; do
    if [ -f "${_bd}/test_bridge.c" ]; then
        BRIDGE_DIR="$_bd"
        break
    fi
done
if [ -n "$BRIDGE_DIR" ]; then
    $CC $INC_FLAGS $COV_FLAGS -c "${BRIDGE_DIR}/test_bridge.c" \
        -o "${BDIR}/bridge.o" 2>/dev/null && BRIDGE_OBJ="${BDIR}/bridge.o"
fi

# ── Step 3: Link test binary with ALL .o files (not archive) ──
# This ensures all library branches are counted in the denominator.
echo "  Linking test binary..."
BRIDGE_INC=""
[ -n "$BRIDGE_DIR" ] && BRIDGE_INC="-I${BRIDGE_DIR}"
if ! $CC $INC_FLAGS -I"$TEST_DIR" $BRIDGE_INC -Wno-implicit-function-declaration \
    $COV_FLAGS \
    "$TEST_SUITE" $OBJ_FILES $BRIDGE_OBJ \
    -lm -Wl,--allow-multiple-definition \
    -o "${BDIR}/test" 2>"${BDIR}/link_err.txt"; then
    echo "ERROR: link failed" >&2
    head -5 "${BDIR}/link_err.txt" >&2
    exit 1
fi

# ── Step 4: Run test ──
echo "  Running test..."
LLVM_PROFILE_FILE="${BDIR}/test.profraw" timeout 120 "${BDIR}/test" \
    > "${BDIR}/test_out.txt" 2>/dev/null || true

if [ ! -f "${BDIR}/test.profraw" ]; then
    echo "ERROR: no profile data generated" >&2
    exit 1
fi

$LLVM_PROFDATA merge "${BDIR}/test.profraw" -o "${BDIR}/test.profdata" 2>/dev/null

# ── Step 5: Generate report ──
# Per-file report filtered to library sources only
$LLVM_COV report "${BDIR}/test" -instr-profile="${BDIR}/test.profdata" 2>/dev/null \
    > "${BDIR}/full_report.txt"

grep "libm/" "${BDIR}/full_report.txt" > "${BDIR}/lib_report.txt"

# Parse library-only totals
# llvm-cov report columns: Filename Regions Missed Cover% Functions Missed Cover% Lines Missed Cover% Branches Missed Cover%
read -r total_branches missed_branches total_funcs missed_funcs total_lines missed_lines <<< $(
    awk '{
        n = NF
        b = $(n-2) + 0; bm = $(n-1) + 0
        l = $(n-5) + 0; lm = $(n-4) + 0
        f = $(n-8) + 0; fm = $(n-7) + 0
        if ($(n) == "-") next
        tb += b; tbm += bm; tl += l; tlm += lm; tf += f; tfm += fm
    } END {
        print tb, tbm, tf, tfm, tl, tlm
    }' "${BDIR}/lib_report.txt"
)

covered_branches=$((total_branches - missed_branches))
covered_funcs=$((total_funcs - missed_funcs))
covered_lines=$((total_lines - missed_lines))

if [ "$total_branches" -gt 0 ]; then
    branch_pct=$(awk "BEGIN { printf \"%.1f\", ($covered_branches / $total_branches) * 100 }")
else
    branch_pct="0.0"
fi
if [ "$total_funcs" -gt 0 ]; then
    func_pct=$(awk "BEGIN { printf \"%.1f\", ($covered_funcs / $total_funcs) * 100 }")
else
    func_pct="0.0"
fi

echo ""
echo "========================================"
echo "COVERAGE REPORT"
echo "========================================"
echo "Functions: ${covered_funcs}/${total_funcs} (${func_pct}%)"
echo "Lines:     ${covered_lines}/${total_lines}"
echo "Branches:  ${covered_branches}/${total_branches} (${branch_pct}%)"
echo "========================================"

# ── Step 6: Save results if output dir specified ──
if [ -n "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
    cp "${BDIR}/lib_report.txt" "${OUTPUT_DIR}/coverage_per_file.txt"
    cat > "${OUTPUT_DIR}/coverage_summary.txt" << EOF
functions_covered=${covered_funcs}
functions_total=${total_funcs}
functions_pct=${func_pct}
branches_covered=${covered_branches}
branches_total=${total_branches}
branches_pct=${branch_pct}
lines_covered=${covered_lines}
lines_total=${total_lines}
tool=${CC}
llvm_cov=${LLVM_COV}
test_suite=$(basename "$TEST_SUITE")
EOF
    echo "Saved to: ${OUTPUT_DIR}/"
fi

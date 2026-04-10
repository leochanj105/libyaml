#!/usr/bin/env bash
set -euo pipefail

# run_testgen_loop.sh — coverage-guided test generation loop.
#
# Expects env vars:
#   TESTGEN_WORKDIR, TEST_CASE_DIR, C_SRC_DIRS, C_INCLUDE_DIRS,
#   COVERAGE_MODES, MAX_ROUNDS, STALL_LIMIT, EXPANDED_PROMPTS_DIR,
#   EXP_DIR, HARNESS_DIR
#
# Uses pre-extracted function/branch lists from ${EXP_DIR}/work-*.
# Selects prompt based on COVERAGE_MODES:
#   "function"  → s4_testgen.md
#   "branch"    → s5_testgen.md
#
# Usage: run_testgen_loop.sh [-v]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS="${HARNESS_DIR:?HARNESS_DIR not set}"
SCRIPTS="${EXP_DIR}/scripts"

source "${HARNESS}/scripts/ai_runner.sh"

VERBOSE=""
[ "${1:-}" = "-v" ] && VERBOSE="-v"

WORKDIR="${TESTGEN_WORKDIR:?TESTGEN_WORKDIR not set}"
mkdir -p "$WORKDIR/rounds"

# Ensure version-matched LLVM tools
export CC="${CC:-clang-21}"
export LLVM_PROFDATA="${LLVM_PROFDATA:-llvm-profdata-21}"
export LLVM_COV="${LLVM_COV:-llvm-cov-21}"

# Parse coverage modes
: "${COVERAGE_MODES:=function}"
: "${MAX_ROUNDS:=5}"
: "${STALL_LIMIT:=2}"
: "${MAX_JOBS:=$(nproc 2>/dev/null || echo 4)}"
has_mode() { echo ",${COVERAGE_MODES}," | grep -q ",$1,"; }

# ── Panic: abort with clear error ──
panic() {
    echo "" >&2
    echo "PANIC: $*" >&2
    echo "  Coverage data is likely corrupt or missing." >&2
    echo "  Check tool versions: CC=${CC:-clang-21}, LLVM_PROFDATA=${LLVM_PROFDATA}, LLVM_COV=${LLVM_COV}" >&2
    exit 1
}

# Select prompt based on mode
if has_mode "branch"; then
    TESTGEN_PROMPT="${EXPANDED_PROMPTS_DIR}/s5_testgen.md"
else
    TESTGEN_PROMPT="${EXPANDED_PROMPTS_DIR}/s4_testgen.md"
fi
[ -f "$TESTGEN_PROMPT" ] || panic "Prompt not found: ${TESTGEN_PROMPT}"

echo ""
echo "========================================"
echo "TEST GENERATION (coverage-guided)"
echo "========================================"
echo "WORKDIR:        ${WORKDIR}"
echo "COVERAGE:       ${COVERAGE_MODES}"
echo "PROMPT:         $(basename "$TESTGEN_PROMPT")"
echo "MAX_ROUNDS:     ${MAX_ROUNDS}"
echo "STALL_LIMIT:    ${STALL_LIMIT}"
echo ""

# ── Ensure function list exists (extracted once, persistent) ──
FUNCTIONS_FILE="${EXP_DIR}/work-functions.md"
if has_mode "function"; then
    bash "${EXP_DIR}/scripts/extract_functions.sh" "$FUNCTIONS_FILE"
    _total_funcs=$(grep -c -v '^#\|^$' "$FUNCTIONS_FILE" || echo 0)
    [ "$_total_funcs" -gt 0 ] || panic "Function list is empty: ${FUNCTIONS_FILE}"
    echo "  Total functions: ${_total_funcs}"
fi

# ── Ensure branch ground truth exists (extracted once, persistent) ──
BRANCHES_JSON="${EXP_DIR}/work-branches.json"
if has_mode "branch"; then
    if [ ! -f "$BRANCHES_JSON" ]; then
        echo "  Extracting branch ground truth..."
        # Build library with coverage, export, extract branches
        _BBDIR=$(mktemp -d)
        _EXCL="cmplx.c|isfinite.c|isgreater.c|isgreaterequal.c|isinf.c|isless.c|islessequal.c|islessgreater.c|isnan.c|isnormal.c|isunordered.c|fenv.c"
        _INC=""
        for _id in $C_INCLUDE_DIRS; do _INC="$_INC -I$_id"; done
        for _d in $C_SRC_DIRS; do
            [ -d "$_d" ] || continue
            find "$_d" -name '*.c' -type f | grep -vE "$_EXCL" | while read -r _cf; do
                $CC $_INC -fprofile-instr-generate -fcoverage-mapping -O0 -fno-builtin \
                    -c "$_cf" -o "${_BBDIR}/c_$(basename "$_cf" .c).o" 2>/dev/null || true
            done
        done
        _OBJ=$(find "$_BBDIR" -name 'c_*.o' -type f | sort)
        echo 'int main(void){return 0;}' > "${_BBDIR}/m.c"
        $CC $_INC -fprofile-instr-generate -fcoverage-mapping -O0 "${_BBDIR}/m.c" $_OBJ \
            -lm -Wl,--allow-multiple-definition -o "${_BBDIR}/b" 2>/dev/null
        ${LLVM_COV} export "${_BBDIR}/b" -empty-profile > "${_BBDIR}/static.json" 2>/dev/null
        python3 "${SCRIPTS}/branch_coverage.py" extract "${_BBDIR}/static.json" "$BRANCHES_JSON"
        rm -rf "$_BBDIR"
    fi
    _total_conditions=$(python3 -c "import json; print(json.load(open('$BRANCHES_JSON'))['total_conditions'])")
    echo "  Total branch conditions: ${_total_conditions}"
fi

# ── Compute uncovered functions (grep-based, with call-pattern matching) ──
compute_uncovered_functions() {
    local test_file="$1"
    local out_file="$2"
    local total=0 covered=0

    : > "$out_file"
    while IFS= read -r func; do
        [[ "$func" =~ ^# ]] && continue
        [ -z "$func" ] && continue
        total=$((total + 1))
        _bare="${func#\[static\] }"
        # Match function call pattern: name( or bridge_name(
        # Also matches bridge wrappers for static functions
        if grep -qP "\b(bridge_)?${_bare}\s*\(" "$test_file" 2>/dev/null; then
            covered=$((covered + 1))
        else
            echo "$func" >> "$out_file"
        fi
    done < "$FUNCTIONS_FILE"

    local uncov=$((total - covered))
    echo "  Function coverage: ${covered}/${total} covered, ${uncov} uncovered"

    # Sanity: if round > 1 and 0 covered, something is wrong
    if [ "$covered" -eq 0 ] && [ "$total" -gt 0 ]; then
        panic "0 functions covered but test_suite.c exists (${total} total). Grep logic may be broken."
    fi
}

# ── Compute uncovered branches (via LLVM coverage + branch_coverage.py) ──
compute_uncovered_branches() {
    local round_dir="$1"

    echo "  Running coverage instrumentation..."
    # run_all_configs compiles test_suite.c with coverage, runs it, writes feedback/
    if ! "${SCRIPTS}/run_all_configs.sh" -w "$WORKDIR" -j "${MAX_JOBS:-$(nproc)}" 2>&1 | tail -5; then
        echo "  WARNING: run_all_configs.sh had errors (may be test crashes)"
    fi

    # Check that export JSON exists
    local feedback_dir="${WORKDIR}/feedback"
    local export_json=""
    for ej in "${feedback_dir}"/*_export.json; do
        [ -f "$ej" ] || continue
        export_json="$ej"
        break
    done

    if [ -z "$export_json" ]; then
        panic "No export JSON produced by build_and_cover.sh"
    fi

    # Use branch_coverage.py to extract uncovered conditions
    python3 "${SCRIPTS}/branch_coverage.py" uncovered "$export_json" \
        "${WORKDIR}/uncovered.md"

    if [ ! -f "${WORKDIR}/uncovered.md" ]; then
        panic "branch_coverage.py did not produce uncovered.md"
    fi

    local uncov_count
    uncov_count=$(grep -c -v '^#\|^$' "${WORKDIR}/uncovered.md" || echo 0)
    local total_count
    total_count=$(head -3 "${WORKDIR}/uncovered.md" | grep -oP 'Total conditions: \K[0-9]+' || echo 0)
    local covered_count=$((total_count - uncov_count))
    echo "  Branch conditions: ${covered_count}/${total_count} covered, ${uncov_count} uncovered"

    cp "${WORKDIR}/uncovered.md" "${round_dir}/uncovered_snapshot.md"
}

# ── Determine starting round ──
start_round=1
max_existing=0
for dir in "${WORKDIR}/rounds"/*/; do
    [ -d "$dir" ] || continue
    n=$(basename "$dir")
    [[ "$n" =~ ^[0-9]+$ ]] || continue
    [ "$n" -gt "$max_existing" ] && max_existing=$n
done
if [ "$max_existing" -gt 0 ]; then
    if [ -f "${WORKDIR}/rounds/${max_existing}/.round_done" ]; then
        start_round=$((max_existing + 1))
    else
        start_round=$max_existing
    fi
fi

stall=0
[ -f "${WORKDIR}/stall_count" ] && stall=$(cat "${WORKDIR}/stall_count")
echo "Starting from round: ${start_round} (stall=${stall})"

# ── Main loop ──
for round in $(seq "$start_round" "$MAX_ROUNDS"); do
    ROUND_DIR="${WORKDIR}/rounds/${round}"
    mkdir -p "$ROUND_DIR"

    echo ""
    echo "========================================"
    echo "TESTGEN ROUND ${round}/${MAX_ROUNDS}"
    echo "========================================"

    step1="${ROUND_DIR}/.step1_done"
    step2="${ROUND_DIR}/.step2_done"
    step3="${ROUND_DIR}/.step3_done"

    # ── Step 1: Check coverage ──
    if [ ! -f "$step1" ]; then
        echo "--- Step 1: Checking coverage ---"

        if has_mode "branch"; then
            compute_uncovered_branches "$ROUND_DIR"
        fi

        if has_mode "function"; then
            uncovered_funcs="${ROUND_DIR}/uncovered_functions.md"
            if [ ! -f "${WORKDIR}/test_suite.c" ]; then
                # Round 1: every function is uncovered
                grep -v "^#" "$FUNCTIONS_FILE" | grep -v "^$" > "$uncovered_funcs" || true
                echo "  Round 1: all $(wc -l < "$uncovered_funcs") functions uncovered"
            else
                compute_uncovered_functions "${WORKDIR}/test_suite.c" "$uncovered_funcs"
            fi
            cp "$uncovered_funcs" "${WORKDIR}/uncovered_functions.md"
        fi

        # Early stop: if nothing is uncovered, we're done
        _all_covered=1
        if has_mode "function" && [ -f "${WORKDIR}/uncovered_functions.md" ] && \
           [ -s "${WORKDIR}/uncovered_functions.md" ]; then
            _all_covered=0
        fi
        if has_mode "branch" && [ -f "${WORKDIR}/uncovered.md" ]; then
            _uncov_branches=$(grep -c -v '^#\|^$' "${WORKDIR}/uncovered.md" || echo 0)
            [ "$_uncov_branches" -gt 0 ] && _all_covered=0
        fi
        if [ "$_all_covered" -eq 1 ] && [ "$round" -gt 1 ]; then
            echo "  100% coverage reached — stopping early."
            touch "$step1" "$step2" "$step3" "${ROUND_DIR}/.round_done"
            break
        fi

        touch "$step1"
    fi

    # ── Step 2: Generate tests (single AI call) ──
    if [ ! -f "$step2" ]; then
        echo "--- Step 2: Test generation ---"

        ln -sfn "$ROUND_DIR" "${WORKDIR}/rounds/current"
        [ -f "${ROUND_DIR}/crash_summary.md" ] || : > "${ROUND_DIR}/crash_summary.md"

        cd "$WORKDIR"

        PROMPT_CTX="Working directory: ${WORKDIR}
File map (use these absolute paths):
  test_suite.c             = ${WORKDIR}/test_suite.c
  crash_summary.md         = ${ROUND_DIR}/crash_summary.md
  uncovered_functions.md   = ${WORKDIR}/uncovered_functions.md
  uncovered.md             = ${WORKDIR}/uncovered.md
  C source tree            = ${TEST_CASE_DIR}/
Output: write the updated test_suite.c to ${WORKDIR}/test_suite.c
If you create test_bridge.c, write it to ${WORKDIR}/test_bridge.c

"
        OUTFILE="${ROUND_DIR}/testgen_output"

        echo "  Using ${CODE_GEN_CMD}..."
        echo "  Prompt: $(basename "$TESTGEN_PROMPT")"
        run_codegen "${PROMPT_CTX}Follow instruction in ${TESTGEN_PROMPT}." \
            "$OUTFILE" "$VERBOSE"

        # Sanity: test_suite.c should exist after generation
        if [ ! -f "${WORKDIR}/test_suite.c" ]; then
            echo "  WARNING: test_suite.c not produced in round ${round}"
        fi

        # Compile check: if AI broke the test suite, fix or revert
        _INC=""
        for _id in $C_INCLUDE_DIRS; do _INC="$_INC -I$_id"; done
        _compile_ok=0
        if $CC $_INC -I"${WORKDIR}" -Wno-implicit-function-declaration \
            -fprofile-instr-generate -fcoverage-mapping \
            -fsyntax-only "${WORKDIR}/test_suite.c" 2>"${ROUND_DIR}/compile_err.txt"; then
            _compile_ok=1
        else
            echo "  WARNING: test_suite.c has compile errors. Attempting auto-fix..."
            # Common fix: missing includes
            _errs=$(cat "${ROUND_DIR}/compile_err.txt")
            if echo "$_errs" | grep -q "memcpy\|memset\|memmove\|strlen"; then
                sed -i '1s/^/#include <string.h>\n/' "${WORKDIR}/test_suite.c"
            fi
            if echo "$_errs" | grep -q "stdlib\|malloc\|free\|exit"; then
                sed -i '1s/^/#include <stdlib.h>\n/' "${WORKDIR}/test_suite.c"
            fi
            # Common fix: forward declaration (function used before defined)
            # Try reordering: move static void test_xxx definitions before main
            # This is hard to do generically, so just try recompiling after includes fix
            if $CC $_INC -I"${WORKDIR}" -Wno-implicit-function-declaration \
                -fprofile-instr-generate -fcoverage-mapping \
                -fsyntax-only "${WORKDIR}/test_suite.c" 2>/dev/null; then
                echo "  Auto-fix succeeded."
                _compile_ok=1
            fi
        fi

        if [ "$_compile_ok" -eq 0 ]; then
            echo "  Auto-fix failed. Reverting to previous round's test suite."
            head -3 "${ROUND_DIR}/compile_err.txt"
            # Revert to previous round
            _prev_snap="${WORKDIR}/rounds/$((round-1))/test_suite_snapshot.c"
            if [ -f "$_prev_snap" ]; then
                cp "$_prev_snap" "${WORKDIR}/test_suite.c"
            fi
        fi

        cp "${WORKDIR}/test_suite.c" "${ROUND_DIR}/test_suite_snapshot.c" 2>/dev/null || true
        touch "$step2"
    fi

    # ── Step 3: Stall check ──
    if [ ! -f "$step3" ]; then
        echo "--- Step 3: Stall check ---"
        progress=0

        if has_mode "branch" && [ "$round" -gt 1 ]; then
            prev="${WORKDIR}/rounds/$((round-1))/uncovered_snapshot.md"
            curr="${ROUND_DIR}/uncovered_snapshot.md"
            if [ -f "$prev" ] && [ -f "$curr" ]; then
                prev_count=$(grep -c -v '^#\|^$' "$prev" || echo 0)
                curr_count=$(grep -c -v '^#\|^$' "$curr" || echo 0)
                echo "  Branch: ${prev_count} -> ${curr_count} uncovered"
                [ "$curr_count" -lt "$prev_count" ] && progress=1
            fi
        fi

        if has_mode "function" && [ "$round" -gt 1 ]; then
            prev="${WORKDIR}/rounds/$((round-1))/uncovered_functions.md"
            curr="${ROUND_DIR}/uncovered_functions.md"
            if [ -f "$prev" ] && [ -f "$curr" ]; then
                prev_count=$(wc -l < "$prev")
                curr_count=$(wc -l < "$curr")
                echo "  Function: ${prev_count} -> ${curr_count} uncovered"
                [ "$curr_count" -lt "$prev_count" ] && progress=1
            fi
        fi

        [ "$round" -eq 1 ] && progress=1

        if [ "$progress" -eq 0 ]; then
            stall=$((stall + 1))
            echo "${stall}" > "${WORKDIR}/stall_count"
            echo "No progress (stall ${stall}/${STALL_LIMIT})"
            [ "$stall" -ge "$STALL_LIMIT" ] && { echo "Stalled."; touch "$step3" "${ROUND_DIR}/.round_done"; break; }
        else
            stall=0
            echo "0" > "${WORKDIR}/stall_count"
        fi

        touch "$step3" "${ROUND_DIR}/.round_done"
    fi
done

echo ""
echo "========================================"
echo "TEST GENERATION COMPLETE"
echo "========================================"
echo "test_suite.c: ${WORKDIR}/test_suite.c"

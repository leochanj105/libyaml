#!/usr/bin/env bash
set -euo pipefail

# run_diffgen.sh — generate difftest_suite.c with compile-fix loop.
# Sources config.sh for all paths.

_DIFFGEN_HARNESS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "${_DIFFGEN_HARNESS}/config.sh"
source "${_DIFFGEN_HARNESS}/scripts/ai_runner.sh"

VERBOSE=""
while getopts "v" opt; do
    case $opt in v) VERBOSE="-v" ;; *) echo "Usage: $0 [-v]"; exit 1 ;; esac
done

WORKDIR="$TESTGEN_WORKDIR"
OUTDIR="$DIFFGEN_WORKDIR"
PROMPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/prompts" && pwd)"

[ -f "${WORKDIR}/test_suite.c" ] || { echo "Error: test_suite.c not found in ${WORKDIR}. Run testgen first."; exit 1; }

mkdir -p "$OUTDIR"
cp "${WORKDIR}/test_suite.c" "${OUTDIR}/test_suite.c"
[ -f "${WORKDIR}/test_manifest.txt" ] && cp "${WORKDIR}/test_manifest.txt" "${OUTDIR}/test_manifest.txt"

if [ ! -d "${OUTDIR}/.claude" ]; then
    cp -r "${HARNESS_DIR}/.claude" "${OUTDIR}/.claude"
fi

_diffgen_run_codegen() {
    local prompt="$1" outfile="$2"
    cd "$OUTDIR"
    run_codegen "$prompt" "$outfile" "$VERBOSE"
}

# Compile-check configs: use all configs if multi-config, or "default" for single-config
read_configs

try_compile() {
    local error_file="$1"
    local difftest="${OUTDIR}/difftest_suite.c"
    local all_ok=0
    : > "$error_file"
    local run_macros
    run_macros=$(grep -oP '(?<=#ifdef RUN_)T\d+' "$difftest" 2>/dev/null | sort -u | while read -r t; do echo -n "-DRUN_$t "; done)

    for config in "${CONFIGS[@]}"; do
        local args
        args=$(get_compile_args "$config")
        if ! $CC $args $run_macros -c "$difftest" -o /dev/null 2>>"$error_file"; then
            echo "  FAIL: ${config}" >&2
            echo "" >> "$error_file"
            all_ok=1
        else
            echo "  OK:   ${config}" >&2
        fi
    done
    return $all_ok
}

# Step 1: Generate
echo "========================================"
echo "GENERATING DIFFTEST"
echo "========================================"
OUTFILE="${OUTDIR}/diffgen_output"

GEN_STARTED="${OUTDIR}/.gen_started"

if [ ! -f "${OUTDIR}/difftest_suite.c" ]; then
    if [ -f "$GEN_STARTED" ]; then
        echo "WARNING: previous difftest generation was interrupted mid-execution."
        echo "  Removing any partial difftest_suite.c and retrying."
        rm -f "${OUTDIR}/difftest_suite.c"
    fi
    touch "$GEN_STARTED"
    echo "Running initial difftest generation ..."
    PROMPT_CTX="Working directory: ${OUTDIR}
File map (use these absolute paths — do NOT guess relative paths):
  test_suite.c     = ${OUTDIR}/test_suite.c
  Rust source tree = ${RUST_DIR}/
Output: write difftest_suite.c to ${OUTDIR}/difftest_suite.c

"
    _diffgen_run_codegen "${PROMPT_CTX}Follow instruction in ${EXPANDED_PROMPTS_DIR}/difftest.md." "$OUTFILE"
    [ -f "${OUTDIR}/difftest_suite.c" ] || { echo "Error: difftest_suite.c not created."; exit 1; }
    rm -f "$GEN_STARTED"
else
    echo "difftest_suite.c already exists — skipping generation."
fi

# Step 2: Compile-fix loop
echo ""
echo "========================================"
echo "COMPILE-CHECK LOOP (max ${MAX_FIX_ROUNDS} rounds)"
echo "========================================"

ERRORS_FILE="${OUTDIR}/compile_errors.txt"

for fix_round in $(seq 1 "$MAX_FIX_ROUNDS"); do
    echo ""
    echo "--- Compile check (round ${fix_round}) ---"
    if try_compile "$ERRORS_FILE"; then
        echo "All check configs compile!"
        break
    fi

    if [ "$fix_round" -ge "$MAX_FIX_ROUNDS" ]; then
        echo "ERROR: Still not compiling after ${MAX_FIX_ROUNDS} rounds."
        exit 1
    fi

    FIX_STARTED="${OUTDIR}/.fix_round_${fix_round}_started"
    if [ -f "$FIX_STARTED" ]; then
        echo "WARNING: fix round ${fix_round} was interrupted mid-execution. Retrying."
    fi
    echo "Feeding errors back to Claude ..."
    FIX_OUTPUT="${OUTDIR}/diffgen_fix_round_${fix_round}"
    touch "$FIX_STARTED"
    _diffgen_run_codegen "The difftest_suite.c you generated has compilation errors. Fix them.
Read difftest_suite.c and fix ALL errors below. Write the corrected file back.

=== COMPILATION ERRORS ===
$(cat "$ERRORS_FILE")" "$FIX_OUTPUT"
    rm -f "$FIX_STARTED"
done

echo ""
if try_compile "$ERRORS_FILE"; then
    echo "difftest_suite.c compiles cleanly."
else
    echo "WARNING: still has compile errors."
    exit 1
fi

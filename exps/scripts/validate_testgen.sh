#!/usr/bin/env bash
set -euo pipefail

# validate_testgen.sh — compile-validate and fix test_suite.c after testgen.
#
# Runs a narrow AI fix loop addressing ONLY compile errors, not semantic
# quality. The goal is to salvage test suites with trivial syntax bugs so
# downstream phases have something to work with — without contaminating
# the scenario's "strategy quality" signal.
#
# Design constraints:
#   - Fix prompt is NARROW: only shows test_suite.c + compile errors.
#     No scenario prompt, no library docs, no coverage info.
#   - Bounded rounds (default 3). Beyond that, marked as compile-failed.
#   - Logs rounds, diffs, and final status for later analysis.
#
# Usage:
#   ./scripts/validate_testgen.sh <scenario> [--max-rounds N] [--model NAME]
#
# Output:
#   $WORK_DIR/test_suite.c              (fixed in place if modifications made)
#   $WORK_DIR/test_suite.c.orig         (backup of pre-validation file)
#   $WORK_DIR/validate_log.json         (round stats)
#   $WORK_DIR/rounds/validate/N/        (per-round artifacts)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"
LIBYAML_DIR="$(dirname "$EXPS_DIR")"

MAX_ROUNDS=3
MODEL="sonnet"
SCENARIO=""

while [ $# -gt 0 ]; do
    case "$1" in
        --max-rounds) shift; MAX_ROUNDS="$1" ;;
        --model)      shift; MODEL="$1" ;;
        -*)           echo "Unknown option: $1" >&2; exit 1 ;;
        *)            SCENARIO="$1" ;;
    esac
    shift
done

[ -n "$SCENARIO" ] || { echo "Usage: $0 <scenario> [--max-rounds N] [--model NAME]"; exit 1; }

# ── Load config ──
source "${SCRIPT_DIR}/configs/${SCENARIO}.sh"
[ -d "$WORK_DIR" ]                || { echo "Error: WORK_DIR not prepared: $WORK_DIR"; exit 1; }
[ -f "${WORK_DIR}/test_suite.c" ] || { echo "Error: test_suite.c not found in $WORK_DIR"; exit 1; }

INC_FLAGS="-DHAVE_CONFIG_H=1 -I${LIBYAML_DIR}/include -I${LIBYAML_DIR}/build/include -I${LIBYAML_DIR}/src -I${WORK_DIR}"

echo "============================================================"
echo "Validating test_suite.c for: $SCENARIO"
echo "  WORK_DIR:   $WORK_DIR"
echo "  MAX_ROUNDS: $MAX_ROUNDS"
echo "  MODEL:      $MODEL"
echo "============================================================"

mkdir -p "${WORK_DIR}/rounds/validate"

# Backup original (only the first time)
[ -f "${WORK_DIR}/test_suite.c.orig" ] || cp "${WORK_DIR}/test_suite.c" "${WORK_DIR}/test_suite.c.orig"

compile_check() {
    local err_file="$1"
    clang-21 $INC_FLAGS -O0 -fsyntax-only "${WORK_DIR}/test_suite.c" 2>"$err_file"
}

round=0
final_status="initial"
final_errors=0

for round in $(seq 1 "$MAX_ROUNDS"); do
    round_dir="${WORK_DIR}/rounds/validate/${round}"
    mkdir -p "$round_dir"
    err_file="${round_dir}/compile_errors.txt"

    if compile_check "$err_file"; then
        echo ""
        echo "Round $round: compile OK"
        final_status="pass"
        break
    fi

    n_errors=$(grep -c 'error:' "$err_file" || true)
    echo ""
    echo "Round $round: $n_errors compile errors"
    head -5 "$err_file" | sed 's/^/  /'

    # Snapshot pre-fix
    cp "${WORK_DIR}/test_suite.c" "${round_dir}/test_suite.pre.c"

    # Narrow fix prompt — no library knowledge, no scenario context.
    errors_content=$(cat "$err_file")
    PROMPT="You are fixing compile errors in a test file called test_suite.c.

The file is in your current directory. It was just compiled with clang and produced these errors:

=== COMPILE ERRORS ===
${errors_content}
=== END ERRORS ===

Instructions:
- Read test_suite.c in the current directory to see the existing code.
- Fix ONLY the compile errors shown above. Do NOT add, remove, or rewrite tests.
- Change as little as possible. Preserve all existing test logic, test names, and inputs.
- You may read test_bridge.h (in the same directory) and /home/leochanj/Desktop/libyaml/include/yaml.h if you need to check a type signature or function name.
- Do NOT read any other files. Do NOT consult coverage, strategy, or scenario documents.
- Write the fixed test_suite.c back to the same path.

Return when done."

    echo "  Invoking claude (model: $MODEL)..."
    (
        cd "$WORK_DIR"
        claude --model "$MODEL" \
               --permission-mode dontAsk \
               --output-format stream-json --verbose \
               -p "$PROMPT" > "${round_dir}/fix_output.jsonl"
    ) || {
        echo "  WARNING: claude invocation failed in round $round"
    }

    # Snapshot post-fix and compute diff size
    cp "${WORK_DIR}/test_suite.c" "${round_dir}/test_suite.post.c"
    diff -u "${round_dir}/test_suite.pre.c" "${round_dir}/test_suite.post.c" > "${round_dir}/fix.diff" 2>/dev/null || true
    n_lines_changed=$(grep -c '^[+-][^+-]' "${round_dir}/fix.diff" 2>/dev/null || echo 0)
    echo "  Round $round: ${n_lines_changed} lines changed"

    # If the file wasn't modified at all, the fixer is stuck — bail out.
    if [ "$n_lines_changed" -eq 0 ]; then
        echo "  WARNING: no changes made; fixer appears stuck"
        final_status="stuck"
        break
    fi
done

# Final compile check
final_err="${WORK_DIR}/rounds/validate/final_errors.txt"
if compile_check "$final_err"; then
    final_status="pass"
    final_errors=0
    echo ""
    echo "FINAL: test_suite.c compiles OK"
else
    [ "$final_status" = "initial" ] && final_status="fail"
    final_errors=$(grep -c 'error:' "$final_err" || true)
    echo ""
    echo "FINAL: test_suite.c still has $final_errors errors after $round round(s)"
fi

# Diff the fully-accumulated change vs the original
final_diff="${WORK_DIR}/rounds/validate/total.diff"
diff -u "${WORK_DIR}/test_suite.c.orig" "${WORK_DIR}/test_suite.c" > "$final_diff" 2>/dev/null || true
total_lines_changed=$(grep -c '^[+-][^+-]' "$final_diff" 2>/dev/null || echo 0)

# Write JSON log
cat > "${WORK_DIR}/validate_log.json" <<LOG
{
  "scenario":            "${SCENARIO}",
  "max_rounds":          ${MAX_ROUNDS},
  "rounds_used":         ${round},
  "final_status":        "${final_status}",
  "final_errors":        ${final_errors},
  "total_lines_changed": ${total_lines_changed}
}
LOG

echo ""
cat "${WORK_DIR}/validate_log.json"

[ "$final_status" = "pass" ] && exit 0 || exit 1

#!/usr/bin/env bash
# judge.sh — End-to-end differential test pipeline.
#
# Runs all test cases against two linked libraries, then compares outputs.
# Reports: passed (match), failed-diff (output mismatch), failed-panic (crash/hang).
#
# Usage:
#   ./judge.sh --lib-a=<path> --lib-b=<path> [--outdir=<dir>] [--timeout=SEC]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LIB_A=""
LIB_B=""
OUTDIR="/tmp/judger-run-$$"
TIMEOUT_SEC=10
INCLUDE_DIR="${SCRIPT_DIR}/../include"
SRC_DIR="${SCRIPT_DIR}/../src"

for arg in "$@"; do
    case "$arg" in
        --lib-a=*)   LIB_A="${arg#--lib-a=}" ;;
        --lib-b=*)   LIB_B="${arg#--lib-b=}" ;;
        --outdir=*)  OUTDIR="${arg#--outdir=}" ;;
        --timeout=*) TIMEOUT_SEC="${arg#--timeout=}" ;;
        --include=*) INCLUDE_DIR="${arg#--include=}" ;;
        --src=*)     SRC_DIR="${arg#--src=}" ;;
        -h|--help)   sed -n '2,8p' "$0"; exit 0 ;;
    esac
done

[ -z "$LIB_A" ] && { echo "Error: --lib-a required" >&2; exit 1; }
[ -z "$LIB_B" ] && { echo "Error: --lib-b required" >&2; exit 1; }
[ -f "$LIB_A" ] || { echo "Error: $LIB_A not found" >&2; exit 1; }
[ -f "$LIB_B" ] || { echo "Error: $LIB_B not found" >&2; exit 1; }

mkdir -p "$OUTDIR"
DIR_A="$OUTDIR/A"
DIR_B="$OUTDIR/B"

echo "[judge] Library A: $LIB_A"
echo "[judge] Library B: $LIB_B"
echo "[judge] Output:    $OUTDIR"

# Run against both libraries
for label in A B; do
    eval lib=\$LIB_$label
    eval dir=\$DIR_$label
    echo ""
    echo "[judge] Running against library $label..."
    LIBYAML_LIB="$lib" \
    INCLUDE_DIR="$INCLUDE_DIR" \
    SRC_DIR="$SRC_DIR" \
    "$SCRIPT_DIR/run-tests.sh" --outdir="$dir" --timeout="$TIMEOUT_SEC"
done

# =========================================================================
# Compare suite by suite, case by case
# =========================================================================
echo ""
echo "[judge] Comparing outputs..."
echo ""

total_pass=0
total_diff=0
total_panic=0

for suite in suite1.out suite2.out; do
    fa="$DIR_A/$suite"
    fb="$DIR_B/$suite"

    # Split each file into one chunk per case
    tmpa=$(mktemp -d); tmpb=$(mktemp -d)
    ( cd "$tmpa" && csplit -sz -f c -b '%05d' "$fa" '/^=== CASE /' '{*}' )
    ( cd "$tmpb" && csplit -sz -f c -b '%05d' "$fb" '/^=== CASE /' '{*}' )

    cases=0
    pass=0
    diff_fail=0
    panic_fail=0
    > "$OUTDIR/$suite.failed"

    for chunk in "$tmpa"/c*; do
        base=$(basename "$chunk")
        cases=$((cases + 1))

        if cmp -s "$chunk" "$tmpb/$base"; then
            pass=$((pass + 1))
            continue
        fi

        # Differs — classify as panic (signal/timeout in either) or diff
        label=$(head -1 "$chunk")
        if grep -qE 'SIGNAL|TIMEOUT' "$chunk" "$tmpb/$base"; then
            panic_fail=$((panic_fail + 1))
            echo "PANIC: $label" >> "$OUTDIR/$suite.failed"
        else
            diff_fail=$((diff_fail + 1))
            echo "DIFF:  $label" >> "$OUTDIR/$suite.failed"
        fi
    done

    rm -rf "$tmpa" "$tmpb"

    total_pass=$((total_pass + pass))
    total_diff=$((total_diff + diff_fail))
    total_panic=$((total_panic + panic_fail))

    fail=$((diff_fail + panic_fail))
    if [ "$fail" -eq 0 ]; then
        echo "$suite: $pass/$cases pass"
    else
        echo "$suite: $pass/$cases pass, $diff_fail diff, $panic_fail panic (labels: $OUTDIR/$suite.failed)"
    fi
done

total=$((total_pass + total_diff + total_panic))

echo ""
echo "===================================="
echo "RESULT"
echo "  Pass:  $total_pass / $total"
echo "  Diff:  $total_diff"
echo "  Panic: $total_panic"
echo "===================================="

[ "$total_diff" -eq 0 ] && [ "$total_panic" -eq 0 ]

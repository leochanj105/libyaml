#!/usr/bin/env bash
set -euo pipefail

# Analyze differential testing results and produce fix goals.
#
# 1. Clears analysis_tmp/
# 2. Claude reads analyze_basic.md + test results/errors → writes goals to analysis_tmp/
# 3. Copies goals from analysis_tmp/ to STEPS_DIR/
#
# Usage: analysis.sh [-v] -d <steps_dir> <result_or_error_file> [...]
#
# Example:
#   ./analysis.sh -d steps_round2 ../agenttest/global_diff_testresult ../agenttest/global_diff_error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../config.sh"
source "${SCRIPT_DIR}/ai_runner.sh"
ANALYSIS_TMP="${WORK_DIR}/analysis_tmp"

usage() {
  echo "Usage: $0 [-v] -d <steps_dir> [-t <difftest_source>] <result_or_error_file> [...]"
  echo ""
  echo "  -v            stream intermediate details to stdout"
  echo "  -d steps_dir  directory to copy goals into (e.g. steps, steps_round2)"
  echo "  -t file       path to the differential test source (e.g. difftest_suite.c, *_diff.c)"
  echo "  Files: diff_testresult and/or diff_error files from agenttest/"
  exit 1
}

VERBOSE=0
STEPS_DIR=""
DIFFTEST_SRC=""
while getopts "vd:t:" opt; do
  case $opt in
    v) VERBOSE=1 ;;
    d) STEPS_DIR="$OPTARG" ;;
    t) DIFFTEST_SRC="$OPTARG" ;;
    *) usage ;;
  esac
done
shift $((OPTIND - 1))

[ -z "$STEPS_DIR" ] && { echo "Error: -d <steps_dir> is required."; usage; }
[ $# -eq 0 ] && usage

# Make STEPS_DIR absolute if relative
case "$STEPS_DIR" in
  /*) ;;
  *) STEPS_DIR="${SCRIPT_DIR}/${STEPS_DIR}" ;;
esac

# Validate input files exist
for f in "$@"; do
  if [ ! -f "$f" ]; then
    echo "Error: file not found: $f"
    exit 1
  fi
done

ANALYZE_OUTPUT="${ANALYSIS_TMP}/analyze_output"

# Build the context from all input files
CONTEXT=$(mktemp)
trap 'rm -f "$CONTEXT"' EXIT

{
  echo "=== DIFFERENTIAL TESTING RESULTS / ERRORS ==="
  echo ""
  for f in "$@"; do
    echo "--- $(basename "$f") ---"
    cat "$f"
    echo ""
  done
  if [ -n "$DIFFTEST_SRC" ] && [ -f "$DIFFTEST_SRC" ]; then
    echo "=== DIFFERENTIAL TEST SOURCE ==="
    echo "--- $(basename "$DIFFTEST_SRC") ---"
    echo "Path: $(realpath "$DIFFTEST_SRC")"
    echo ""
    cat "$DIFFTEST_SRC"
    echo ""
  fi
} > "$CONTEXT"


# =========================================================================
# Step 1: Clear analysis_tmp/
# =========================================================================
echo "Clearing ${ANALYSIS_TMP}/ ..."
rm -rf "$ANALYSIS_TMP"
mkdir -p "$ANALYSIS_TMP"

# =========================================================================
# Step 2: Run Claude to analyze and produce goals in analysis_tmp/
# =========================================================================
echo "========================================"
echo "ANALYSIS"
echo "========================================"

# Write context to a file Claude can read (avoids ARG_MAX limits)
CONTEXT_FILE="${ANALYSIS_TMP}/analysis_context.txt"
mkdir -p "$ANALYSIS_TMP"
cp "$CONTEXT" "$CONTEXT_FILE"

RUST_CHANGES="${WORK_DIR}/rust_changes.md"
_rust_changes_ref=""
if [ -f "$RUST_CHANGES" ]; then
  _rust_changes_ref="
Recent Rust changes (incremental diff): ${RUST_CHANGES}
  Read this file for context on what has already been modified."
fi

ANALYZE_PROMPT="Working directory: ${HARNESS_DIR}
C source: ${TEST_CASE_DIR}/
Rust source: ${RUST_DIR}/
Goal output directory (write all goal files here as absolute paths):
  goal_1.md = ${ANALYSIS_TMP}/goal_1.md
  goal_2.md = ${ANALYSIS_TMP}/goal_2.md
  ... (continue numbering)
${_rust_changes_ref}

$(cat "${EXPANDED_PROMPTS_DIR}/analyze.md")

Read the differential test results/errors from: ${CONTEXT_FILE}"

cd "$HARNESS_DIR"
echo "Running analysis (using ${ANALYSIS_CMD:-claude})..."
echo "Output: $ANALYZE_OUTPUT"
echo "Context file: $CONTEXT_FILE"
echo ""

run_analysis "$ANALYZE_PROMPT" "$ANALYZE_OUTPUT" "$VERBOSE"

rm -f "$CONTEXT_FILE"

# =========================================================================
# Step 3: Copy goals from analysis_tmp/ to STEPS_DIR/
# =========================================================================
echo ""
echo "========================================"
echo "COPYING GOALS"
echo "========================================"

GOAL_COUNT=$(ls -1 "${ANALYSIS_TMP}"/goal_*.md 2>/dev/null | wc -l)

if [ "$GOAL_COUNT" -eq 0 ]; then
  echo "Warning: no goal files generated in ${ANALYSIS_TMP}/"
  exit 1
fi

mkdir -p "$STEPS_DIR"
cp "${ANALYSIS_TMP}"/goal_*.md "$STEPS_DIR/"

echo "Copied ${GOAL_COUNT} goals from analysis_tmp/ to ${STEPS_DIR}/:"
ls -1 "${STEPS_DIR}"/goal_*.md

echo ""
echo "Done."

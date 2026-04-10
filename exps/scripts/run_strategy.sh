#!/usr/bin/env bash
set -euo pipefail

# run_strategy.sh — invoke Claude to analyze uncovered branches and produce
# a strategy document (rounds/<N>/strategy.md) and config plan
# describing which tests to write next round.
#
# Usage: run_strategy.sh <round_number> <workdir> [-v]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../config.sh"
source "${SCRIPT_DIR}/ai_runner.sh"

usage() {
  echo "Usage: $0 <round_number> <workdir> [-v]"
  exit 1
}

[ $# -lt 2 ] && usage
ROUND="$1"
WORKDIR="$2"
VERBOSE="${3:-}"

ROUND_DIR="${WORKDIR}/rounds/${ROUND}"
OUTFILE="${ROUND_DIR}/strategy_output"
mkdir -p "$ROUND_DIR"

# Point rounds/current symlink at this round so strategy.md can use a fixed path
ln -sfn "${ROUND_DIR}" "${WORKDIR}/rounds/current"

cd "$WORKDIR"
echo "Running strategy analysis for round ${ROUND} (using ${ANALYSIS_CMD:-claude})..."
echo "Output: $OUTFILE"

PROMPT_CTX="Working directory: ${WORKDIR}
File map (use these absolute paths — do NOT guess relative paths):
  uncovered.md                    = ${WORKDIR}/uncovered.md
  rounds/current/newly_covered.md = ${ROUND_DIR}/newly_covered.md
  test_suite.c                    = ${WORKDIR}/test_suite.c
  unreachable.md                  = ${WORKDIR}/unreachable.md
Output files to write:
  rounds/current/strategy.md      = ${ROUND_DIR}/strategy.md
Append (do not rewrite): unreachable.md = ${WORKDIR}/unreachable.md

"
run_analysis "${PROMPT_CTX}Follow instruction in ${EXPANDED_PROMPTS_DIR}/strategy.md." "$OUTFILE" "$VERBOSE"

echo "Strategy written to: ${ROUND_DIR}/strategy.md"

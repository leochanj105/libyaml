#!/usr/bin/env bash
# judger.sh — Held-out judger for libyaml C-to-Rust transpilation.
#
# Contract:
#   - Takes $1 = output report path
#   - Uses RUST_DIR (Rust transpilation), JUDGER_DIR (test data)
#   - Compares C libyaml vs Rust libyaml on real YAML inputs
#   - Exit 0 = all pass, non-zero = failures
#
# Delegates to the judger in testing/ which has the actual test infrastructure.

set -euo pipefail

REPORT="${1:-/dev/stdout}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIBYAML_DIR="$(dirname "$SCRIPT_DIR")"

: "${TEST_CASE_DIR:=${LIBYAML_DIR}}"
: "${RUST_DIR:?RUST_DIR must be set}"
: "${JUDGER_DIR:=${LIBYAML_DIR}/testing}"
: "${CC:=clang-21}"

# Use the judger runner from testing/ if it exists
if [ -x "${JUDGER_DIR}/run-tests.sh" ]; then
    exec "${JUDGER_DIR}/run-tests.sh" "$REPORT"
fi

# Fallback: delegate to project/judger.sh if it exists
if [ -x "${LIBYAML_DIR}/project/judger.sh" ]; then
    exec "${LIBYAML_DIR}/project/judger.sh" "$REPORT"
fi

echo "ERROR: No judger implementation found." >&2
echo "  Expected: ${JUDGER_DIR}/run-tests.sh" >&2
echo "  Or:       ${LIBYAML_DIR}/project/judger.sh" >&2
exit 1

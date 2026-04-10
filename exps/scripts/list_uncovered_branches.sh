#!/usr/bin/env bash
set -euo pipefail

# list_uncovered_branches.sh — thin wrapper around cov.list_uncovered_branches
#
# Usage:
#   ./list_uncovered_branches.sh                    # all uncovered conditions
#   ./list_uncovered_branches.sh --summary          # per-file summary only
#   ./list_uncovered_branches.sh --exclude-headers
#   ./list_uncovered_branches.sh --file reader.c
#
# Prerequisites: ./build_cov.sh <test.c> --run (needs profdata)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="${SCRIPT_DIR}/build_cov/test_bin"
PROFDATA="${SCRIPT_DIR}/build_cov/coverage.profdata"

cd "$SCRIPT_DIR"
exec python3 -m cov.list_uncovered_branches "$BIN" --profdata "$PROFDATA" "$@"

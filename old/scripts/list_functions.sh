#!/usr/bin/env bash
set -euo pipefail

# list_functions.sh — thin wrapper around cov.list_functions
#
# Usage:
#   ./list_functions.sh                  # all functions
#   ./list_functions.sh --static-only    # only static functions
#   ./list_functions.sh --public-only    # only public functions
#   ./list_functions.sh --uncovered      # only NOT executed in last run
#   ./list_functions.sh --executed       # only executed in last run
#
# Prerequisites: ./build_cov.sh <test.c> [--run]
# (--uncovered/--executed require --run, since they need profdata)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"
BIN="${EXPS_DIR}/build_cov/test_bin"
PROFDATA="${EXPS_DIR}/build_cov/coverage.profdata"

ARGS=()
WANT_PROFDATA=0
for arg in "$@"; do
    case "$arg" in
        --uncovered|--executed) WANT_PROFDATA=1 ;;
    esac
    ARGS+=("$arg")
done

if [ "$WANT_PROFDATA" -eq 1 ] && [ -f "$PROFDATA" ]; then
    ARGS+=(--profdata "$PROFDATA")
fi

cd "$EXPS_DIR"
exec python3 -m cov.list_functions "$BIN" "${ARGS[@]}"

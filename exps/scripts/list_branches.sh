#!/usr/bin/env bash
set -euo pipefail

# list_branches.sh — thin wrapper around cov.list_branches
#
# Usage:
#   ./list_branches.sh                       # all branches, file:line
#   ./list_branches.sh --by-file             # per-file counts
#   ./list_branches.sh --exclude-headers     # skip yaml_private.h macros
#   ./list_branches.sh --file scanner.c      # one file only
#   ./list_branches.sh --with-cols           # include column numbers

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"
BIN="${EXPS_DIR}/build_cov/test_bin"

cd "$EXPS_DIR"
exec python3 -m cov.list_branches "$BIN" "$@"

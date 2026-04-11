#!/usr/bin/env bash
set -euo pipefail

# prepare_transpile.sh — Set up exps/rust-baseline/ for the transpile phase.
#
# What it does:
#   1. Creates exps/rust-baseline/ (shared across scenarios; transpiled ONCE)
#   2. Copies prompts/transpile.md → rust-baseline/transpile_prompt.md
#   3. Writes rust-baseline/.claude/settings.json with restrictive permissions:
#        - Read: libyaml src/, include/, build/include/
#        - Write/Edit: rust-baseline/ only
#        - Bash: only cargo invocations (for compile verification)
#        - Deny: WebFetch, WebSearch
#
# Usage:
#   ./scripts/prepare_transpile.sh
#
# After running: ./scripts/run_transpile.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"
LIBYAML_DIR="$(dirname "$EXPS_DIR")"

RUST_BASELINE="${EXPS_DIR}/rust-baseline"
PROMPT_FILE="${EXPS_DIR}/prompts/transpile.md"

[ -f "$PROMPT_FILE" ] || { echo "Error: prompt not found: $PROMPT_FILE"; exit 1; }

echo "============================================================"
echo "Preparing transpile phase"
echo "  LIBYAML_DIR:   $LIBYAML_DIR"
echo "  RUST_BASELINE: $RUST_BASELINE"
echo "  PROMPT:        $PROMPT_FILE"
echo "============================================================"

if [ -d "$RUST_BASELINE" ] && [ "$(ls -A "$RUST_BASELINE" 2>/dev/null | grep -v '^\.claude$' | grep -v '^transpile_prompt.md$' || true)" ]; then
    echo ""
    echo "WARNING: ${RUST_BASELINE} already contains files:"
    ls -la "$RUST_BASELINE" | head -10 | sed 's/^/  /'
    echo ""
    echo "Remove it manually if you want a fresh transpile, then re-run this script."
    exit 1
fi

# ── 1. Create rust-baseline/ ──
mkdir -p "$RUST_BASELINE"

# ── 2. Copy transpile prompt ──
cp "$PROMPT_FILE" "${RUST_BASELINE}/transpile_prompt.md"
echo "  Copied: ${RUST_BASELINE}/transpile_prompt.md"

# ── 3. Write Claude Code settings.json ──
mkdir -p "${RUST_BASELINE}/.claude"
cat > "${RUST_BASELINE}/.claude/settings.json" <<SETTINGS
{
  "permissions": {
    "deny": [
      "WebFetch",
      "WebSearch"
    ],
    "allow": [
      "Read(./**)",
      "Write(./**)",
      "Edit(./**)",
      "Glob(./**)",
      "Grep(./**)",
      "Read(/${LIBYAML_DIR}/src/**)",
      "Glob(/${LIBYAML_DIR}/src/**)",
      "Grep(/${LIBYAML_DIR}/src/**)",
      "Read(/${LIBYAML_DIR}/include/**)",
      "Glob(/${LIBYAML_DIR}/include/**)",
      "Grep(/${LIBYAML_DIR}/include/**)",
      "Read(/${LIBYAML_DIR}/build/include/**)",
      "Bash(cargo:*)"
    ]
  }
}
SETTINGS
echo "  Wrote:  ${RUST_BASELINE}/.claude/settings.json"

echo ""
echo "Prepared. Layout:"
ls -la "$RUST_BASELINE" | sed 's/^/  /'
echo ""
echo "Next: ./scripts/run_transpile.sh"

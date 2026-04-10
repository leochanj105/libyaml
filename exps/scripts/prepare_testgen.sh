#!/usr/bin/env bash
set -euo pipefail

# prepare_testgen.sh — Set up a scenario's work directory for testgen.
#
# What it does:
#   1. Sources scripts/configs/<scenario>.sh to get WORK_DIR
#   2. Creates WORK_DIR (e.g., exps/work-s1)
#   3. Symlinks bridge files (test_bridge.h + bridge_*.c) into WORK_DIR
#   4. Copies the scenario's testgen prompt into WORK_DIR/testgen_prompt.md
#   5. Writes WORK_DIR/.claude/settings.json with restrictive permissions:
#        - Read: libyaml src/include, build/include, build_cov/bridges, WORK_DIR
#        - Write/Edit: WORK_DIR only
#        - Bash, WebFetch, WebSearch: denied
#
# Usage:
#   ./scripts/prepare_testgen.sh <scenario>
#
# Scenarios: s1_naive, s2_explicit, s3_expert, s4_function, s5_branch
#
# Prerequisites:
#   - exps/build_cov/bridges/ exists (run cov.gen_bridges first)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"
LIBYAML_DIR="$(dirname "$EXPS_DIR")"

SCENARIO="${1:-}"
[ -n "$SCENARIO" ] || { echo "Usage: $0 <scenario>"; exit 1; }

CONFIG_FILE="${SCRIPT_DIR}/configs/${SCENARIO}.sh"
PROMPT_FILE="${EXPS_DIR}/prompts/${SCENARIO}_testgen.md"
BRIDGE_DIR="${EXPS_DIR}/build_cov/bridges"

[ -f "$CONFIG_FILE" ] || { echo "Error: config not found: $CONFIG_FILE"; exit 1; }
[ -f "$PROMPT_FILE" ] || { echo "Error: prompt not found: $PROMPT_FILE"; exit 1; }
[ -d "$BRIDGE_DIR"  ] || { echo "Error: bridges not found: $BRIDGE_DIR (run: python3 -m cov.gen_bridges ../src build_cov/bridges --binary build_cov/test_bin)"; exit 1; }

# Source config to get WORK_DIR
source "$CONFIG_FILE"

[ -n "${WORK_DIR:-}" ] || { echo "Error: WORK_DIR not set by config"; exit 1; }

echo "============================================================"
echo "Preparing scenario: $SCENARIO"
echo "  WORK_DIR:   $WORK_DIR"
echo "  PROMPT:     $PROMPT_FILE"
echo "  BRIDGE_DIR: $BRIDGE_DIR"
echo "============================================================"

# ── 1. Create work directory ──
mkdir -p "$WORK_DIR"

# ── 2. Symlink bridge header (so AI can read it via #include "test_bridge.h") ──
ln -sf "${BRIDGE_DIR}/test_bridge.h" "${WORK_DIR}/test_bridge.h"
echo "  Linked: ${WORK_DIR}/test_bridge.h -> ${BRIDGE_DIR}/test_bridge.h"

# ── 3. Symlink each bridge_*.c so the build script (later) can find them ──
for src in "${BRIDGE_DIR}"/bridge_*.c; do
    base=$(basename "$src")
    ln -sf "$src" "${WORK_DIR}/${base}"
done
echo "  Linked: bridge_*.c symlinks ($(ls "${BRIDGE_DIR}"/bridge_*.c | wc -l) files)"

# ── 4. Copy the testgen prompt ──
cp "$PROMPT_FILE" "${WORK_DIR}/testgen_prompt.md"
echo "  Copied: ${WORK_DIR}/testgen_prompt.md"

# ── 5. Write Claude Code settings.json ──
mkdir -p "${WORK_DIR}/.claude"
cat > "${WORK_DIR}/.claude/settings.json" <<SETTINGS
{
  "permissions": {
    "deny": [
      "Read",
      "Write",
      "Edit",
      "Glob",
      "Grep",
      "Bash",
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
      "Read(/${EXPS_DIR}/build_cov/bridges/**)",
      "Glob(/${EXPS_DIR}/build_cov/bridges/**)",
      "Grep(/${EXPS_DIR}/build_cov/bridges/**)"
    ]
  }
}
SETTINGS
echo "  Wrote:  ${WORK_DIR}/.claude/settings.json"

echo ""
echo "Prepared. Layout:"
ls -la "$WORK_DIR" | sed 's/^/  /'
echo ""
echo "Next: ./scripts/run_testgen.sh ${SCENARIO}"

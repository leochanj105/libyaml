#!/usr/bin/env bash
set -euo pipefail

# run_testgen.sh — Invoke Claude Code to run testgen for a scenario.
#
# Prerequisites: ./scripts/prepare_testgen.sh <scenario> must have run first.
#
# What it does:
#   1. Sources scripts/configs/<scenario>.sh to get WORK_DIR
#   2. cd into WORK_DIR (so .claude/settings.json applies)
#   3. Invokes claude with the testgen prompt
#   4. Streams the output to stdout AND saves a transcript
#
# The settings.json restricts the AI to:
#   - Read libyaml src/include + build_cov/bridges
#   - Write/Edit only inside WORK_DIR
#   - No Bash, no network
#
# Usage:
#   ./scripts/run_testgen.sh <scenario>          # run interactively (verbose)
#   ./scripts/run_testgen.sh <scenario> --quiet  # capture only

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"

# Default model — Sonnet 4.6 (override with --model <name>)
MODEL="sonnet"

SCENARIO="${1:-}"
[ -n "$SCENARIO" ] || { echo "Usage: $0 <scenario> [--quiet] [--model <name>]"; exit 1; }
shift || true

VERBOSE=1
while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) VERBOSE=0 ;;
        --model) shift; MODEL="$1" ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
    shift
done

CONFIG_FILE="${SCRIPT_DIR}/configs/${SCENARIO}.sh"
[ -f "$CONFIG_FILE" ] || { echo "Error: config not found: $CONFIG_FILE"; exit 1; }

source "$CONFIG_FILE"
[ -n "${WORK_DIR:-}" ] || { echo "Error: WORK_DIR not set"; exit 1; }
[ -d "$WORK_DIR" ]    || { echo "Error: WORK_DIR not prepared. Run: ./scripts/prepare_testgen.sh $SCENARIO"; exit 1; }

PROMPT_FILE="${WORK_DIR}/testgen_prompt.md"
[ -f "$PROMPT_FILE" ] || { echo "Error: prompt missing: $PROMPT_FILE (re-run prepare)"; exit 1; }

TRANSCRIPT="${WORK_DIR}/testgen_output.jsonl"

echo "============================================================"
echo "Running testgen: $SCENARIO"
echo "  MODEL:       $MODEL"
echo "  WORK_DIR:    $WORK_DIR"
echo "  PROMPT:      $PROMPT_FILE"
echo "  TRANSCRIPT:  $TRANSCRIPT"
echo "============================================================"
echo ""

# Read prompt content and pass directly (no indirection through "read this file").
PROMPT_CONTENT="$(cat "$PROMPT_FILE")"

# Run claude with cwd = WORK_DIR so .claude/settings.json applies.
cd "$WORK_DIR"

if [ "$VERBOSE" -eq 1 ]; then
    claude --model "$MODEL" \
           --permission-mode dontAsk \
           --output-format stream-json --verbose \
           -p "$PROMPT_CONTENT" \
        | tee "$TRANSCRIPT" \
        | python3 -c '
import sys, json
for line in sys.stdin:
    try:
        msg = json.loads(line)
    except json.JSONDecodeError:
        continue
    t = msg.get("type")
    if t == "assistant":
        for c in msg.get("message", {}).get("content", []):
            if c.get("type") == "text":
                sys.stdout.write(c["text"])
                sys.stdout.flush()
            elif c.get("type") == "tool_use":
                name = c.get("name", "?")
                inp = c.get("input", {})
                preview = json.dumps(inp)[:120]
                print(f"\n[tool:{name}] {preview}")
'
else
    claude --model "$MODEL" \
           --permission-mode dontAsk \
           --output-format stream-json --verbose \
           -p "$PROMPT_CONTENT" > "$TRANSCRIPT"
fi

echo ""
echo "============================================================"
echo "Done."
echo "  Transcript: $TRANSCRIPT"
if [ -f "${WORK_DIR}/test_suite.c" ]; then
    echo "  test_suite.c: $(wc -l < "${WORK_DIR}/test_suite.c") lines"
else
    echo "  WARNING: test_suite.c was not produced"
fi

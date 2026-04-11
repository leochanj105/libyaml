#!/usr/bin/env bash
set -euo pipefail

# run_transpile.sh — Invoke Claude Code to transpile libyaml C → Rust.
#
# Prerequisites: ./scripts/prepare_transpile.sh must have run first.
#
# What it does:
#   1. cd into exps/rust-baseline/ (so .claude/settings.json applies)
#   2. Invokes claude with the transpile prompt
#   3. Streams output to stdout AND saves a transcript
#
# The settings.json restricts the AI to:
#   - Read libyaml src/, include/, build/include/
#   - Write/Edit only inside rust-baseline/
#   - Bash: only `cargo ...` commands (for compile verification)
#   - No network
#
# Usage:
#   ./scripts/run_transpile.sh                   # default: sonnet, verbose
#   ./scripts/run_transpile.sh --model opus      # override model
#   ./scripts/run_transpile.sh --quiet           # capture only, no pretty stream

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPS_DIR="$(dirname "$SCRIPT_DIR")"

RUST_BASELINE="${EXPS_DIR}/rust-baseline"
PROMPT_FILE="${RUST_BASELINE}/transpile_prompt.md"

MODEL="sonnet"
VERBOSE=1

while [ $# -gt 0 ]; do
    case "$1" in
        --quiet) VERBOSE=0 ;;
        --model) shift; MODEL="$1" ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
    shift
done

[ -d "$RUST_BASELINE" ] || { echo "Error: not prepared. Run: ./scripts/prepare_transpile.sh"; exit 1; }
[ -f "$PROMPT_FILE" ]   || { echo "Error: prompt missing: $PROMPT_FILE (re-run prepare)"; exit 1; }

TRANSCRIPT="${RUST_BASELINE}/transpile_output.jsonl"

echo "============================================================"
echo "Running transpile"
echo "  MODEL:         $MODEL"
echo "  RUST_BASELINE: $RUST_BASELINE"
echo "  PROMPT:        $PROMPT_FILE"
echo "  TRANSCRIPT:    $TRANSCRIPT"
echo "============================================================"
echo ""

PROMPT_CONTENT="$(cat "$PROMPT_FILE")"

cd "$RUST_BASELINE"

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
if [ -f "${RUST_BASELINE}/Cargo.toml" ]; then
    rs_count=$(find "${RUST_BASELINE}/src" -name '*.rs' 2>/dev/null | wc -l)
    echo "  Cargo.toml: yes"
    echo "  .rs files:  ${rs_count}"
else
    echo "  WARNING: Cargo.toml was not produced"
fi

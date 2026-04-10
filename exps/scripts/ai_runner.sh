#!/usr/bin/env bash
# ai_runner.sh — sourceable helper for AI invocation.
# Provides run_codegen() and run_analysis() that dispatch to the configured AI binary.
#
# Controlled by:
#   CODE_GEN_CMD  (default: codex)  — binary used for code generation tasks
#   ANALYSIS_CMD  (default: claude) — binary used for analysis/planning tasks
#
# Usage in scripts:
#   source "${SCRIPT_DIR}/../scripts/ai_runner.sh"
#   run_codegen  "$prompt" "$outfile" "$VERBOSE"
#   run_analysis "$prompt" "$outfile" "$VERBOSE"
#
# VERBOSE: any non-empty, non-zero value enables streaming; "" or "0" = quiet.

# JQ filter for claude stream-json format
_AI_RUNNER_JQ_FILTER='
  if .type == "assistant" then
    .message.content[]? |
    if .type == "text" then "\n[assistant] " + .text
    elif .type == "tool_use" then "\n[tool:" + .name + "] " + (.input | tostring)
    else empty end
  elif .type == "tool_result" then
    "[result] " + (if .content then (.content | tostring) else "(no output)" end)
  else empty end
'

# _ai_runner_verbose_p VERBOSE — returns 0 (true) if verbose mode is on
_ai_runner_verbose_p() {
    local v="${1:-}"
    [ -n "$v" ] && [ "$v" != "0" ]
}

# =========================================================================
# Judger directory lock — chmod 000 before every AI call, restore after.
# This prevents any AI model (including Codex with danger-full-access) from
# reading held-out judger test data during the pipeline.
# =========================================================================
_JUDGER_LOCKED=0
_JUDGER_SAVED_PERMS="755"

_lock_judger() {
    _JUDGER_LOCKED=0
    if [ -n "${JUDGER_SCRIPT:-}" ] && [ -z "${JUDGER_DIR:-}" ]; then
        echo "ERROR: JUDGER_SCRIPT is set but JUDGER_DIR is empty." >&2
        echo "  Set JUDGER_DIR to the directory containing judger test data." >&2
        echo "  Without it the harness cannot lock the judger tests from AI access." >&2
        exit 1
    fi
    if [ -n "${JUDGER_DIR:-}" ] && [ -d "$JUDGER_DIR" ]; then
        _JUDGER_SAVED_PERMS=$(stat -c "%a" "$JUDGER_DIR" 2>/dev/null || echo "755")
        chmod 000 "$JUDGER_DIR" 2>/dev/null || true
        _JUDGER_LOCKED=1
    elif [ -n "${JUDGER_DIR:-}" ]; then
        echo "ERROR: JUDGER_DIR does not exist: ${JUDGER_DIR}" >&2
        exit 1
    fi
}

_unlock_judger() {
    if [ "${_JUDGER_LOCKED:-0}" -eq 1 ] && [ -n "${JUDGER_DIR:-}" ]; then
        chmod "${_JUDGER_SAVED_PERMS:-755}" "$JUDGER_DIR" 2>/dev/null || true
        _JUDGER_LOCKED=0
    fi
}

# _run_ai_cmd_inner AI_CMD VERBOSE PROMPT OUTFILE
# Raw AI invocation — called only from _run_ai_cmd (which holds the judger lock).
_run_ai_cmd_inner() {
    local ai_cmd="$1"
    local verbose="${2:-}"
    local prompt="$3"
    local outfile="$4"

    local bin_name
    bin_name=$(basename "$ai_cmd")

    case "$bin_name" in
        claude)
            if _ai_runner_verbose_p "$verbose"; then
                "$ai_cmd" --permission-mode dontAsk --output-format stream-json --verbose \
                    -p "$prompt" \
                    | tee "$outfile" \
                    | while IFS= read -r line; do
                        echo "$line" | jq -rj "$_AI_RUNNER_JQ_FILTER" 2>/dev/null || true
                      done
            else
                "$ai_cmd" --permission-mode dontAsk --output-format stream-json --verbose \
                    -p "$prompt" > "$outfile"
            fi
            ;;
        codex)
            if _ai_runner_verbose_p "$verbose"; then
                "$ai_cmd" exec --sandbox danger-full-access "$prompt" 2>&1 | tee "$outfile"
            else
                "$ai_cmd" exec --sandbox danger-full-access "$prompt" > "$outfile" 2>&1
            fi
            ;;
        *)
            # Unknown binary: try claude-style CLI
            if _ai_runner_verbose_p "$verbose"; then
                "$ai_cmd" --permission-mode dontAsk --output-format stream-json --verbose \
                    -p "$prompt" \
                    | tee "$outfile" \
                    | while IFS= read -r line; do
                        echo "$line" | jq -rj "$_AI_RUNNER_JQ_FILTER" 2>/dev/null || true
                      done
            else
                "$ai_cmd" --permission-mode dontAsk --output-format stream-json --verbose \
                    -p "$prompt" > "$outfile"
            fi
            ;;
    esac
}

# _run_ai_cmd AI_CMD VERBOSE PROMPT OUTFILE
# Invokes AI_CMD with judger directory locked for the duration of the call.
# Restores permissions even if the AI call fails.
_run_ai_cmd() {
    local ai_cmd="$1"
    local verbose="${2:-}"
    local prompt="$3"
    local outfile="$4"

    _lock_judger

    local _rc=0
    _run_ai_cmd_inner "$ai_cmd" "$verbose" "$prompt" "$outfile" || _rc=$?

    _unlock_judger
    return $_rc
}

# run_codegen PROMPT OUTFILE [VERBOSE]
# Use CODE_GEN_CMD (default: codex) for code generation tasks.
run_codegen() {
    local prompt="$1" outfile="$2" verbose="${3:-}"
    local cmd="${CODE_GEN_CMD:-codex}"
    _run_ai_cmd "$cmd" "$verbose" "$prompt" "$outfile"
}

# run_analysis PROMPT OUTFILE [VERBOSE]
# Use ANALYSIS_CMD (default: claude) for analysis/planning tasks.
run_analysis() {
    local prompt="$1" outfile="$2" verbose="${3:-}"
    local cmd="${ANALYSIS_CMD:-claude}"
    _run_ai_cmd "$cmd" "$verbose" "$prompt" "$outfile"
}

# run_analysis_text PROMPT [VERBOSE]
# Like run_analysis but prints the plain text response to stdout.
# Handles both claude (stream-json) and codex (plain text) output formats.
# Use this whenever you need to capture a short answer (e.g. FIXED/UNFIXED).
run_analysis_text() {
    local prompt="$1" verbose="${2:-}"
    local cmd="${ANALYSIS_CMD:-claude}"
    local bin_name
    bin_name=$(basename "$cmd")
    local tmpfile
    tmpfile=$(mktemp)

    local _rc=0
    run_analysis "$prompt" "$tmpfile" "$verbose" || _rc=$?

    if [ "$_rc" -eq 0 ]; then
        case "$bin_name" in
            claude)
                # stream-json: extract text fields from assistant messages
                jq -rj                     'select(.type == "assistant") | .message.content[]? | select(.type == "text") | .text'                     "$tmpfile" 2>/dev/null || cat "$tmpfile"
                ;;
            *)
                # codex and others: plain text output
                cat "$tmpfile"
                ;;
        esac
    fi

    rm -f "$tmpfile"
    return $_rc
}

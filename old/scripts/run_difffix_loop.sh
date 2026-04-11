#!/usr/bin/env bash
set -euo pipefail

# run_difffix_loop.sh — diff-test-fix loop.
#
# Each round:
#   1. Generate compact divergence context (per-function C+Rust source)
#   2. Analysis (claude) → goal files
#   3. Git snapshot (pre-fix), then Fixer (claude) → apply fixes to Rust code
#   4. Re-test
#   5. Record stats + stall check
#
# Expects env vars:
#   DIFFGEN_WORKDIR, DIFFFIX_WORKDIR, RUST_DIR, TEST_CASE_DIR,
#   C_SRC_DIRS, C_INCLUDE_DIRS, EXPANDED_PROMPTS_DIR,
#   EXP_DIR, HARNESS_DIR, MAX_ROUNDS, STALL_LIMIT, MAX_GOALS

HARNESS="${HARNESS_DIR:?HARNESS_DIR not set}"
source "${HARNESS}/scripts/ai_runner.sh"

: "${MAX_ROUNDS:=5}"
: "${STALL_LIMIT:=2}"
: "${MAX_GOALS:=5}"
: "${REACT_MODE:=0}"

VERBOSE=""
[ "${1:-}" = "-v" ] && VERBOSE="-v"

DIFFGEN="${DIFFGEN_WORKDIR:?DIFFGEN_WORKDIR not set}"
OUTDIR="${DIFFFIX_WORKDIR:?DIFFFIX_WORKDIR not set}"

mkdir -p "$OUTDIR"

# Ensure .claude config exists in difffix and rust dirs for AI permissions
if [ -d "${HARNESS}/.claude" ]; then
    [ -d "${OUTDIR}/.claude" ] || cp -r "${HARNESS}/.claude" "${OUTDIR}/.claude"
    [ -d "${RUST_DIR}/.claude" ] || cp -r "${HARNESS}/.claude" "${RUST_DIR}/.claude"
fi

echo ""
echo "========================================"
echo "DIFF-FIX LOOP"
echo "========================================"
echo "RUST_DIR:     ${RUST_DIR}"
echo "MAX_ROUNDS:   ${MAX_ROUNDS}"
echo "MAX_GOALS:    ${MAX_GOALS}"
echo "STALL_LIMIT:  ${STALL_LIMIT}"
echo ""

# ── Snapshot system for Rust code ──
# Uses file-based diffs stored in the round directory. No git init inside RUST_DIR
# (avoids submodule hell when committing from the outer repo).

_snapshot_src() {
    # Save a copy of src/ to a round directory for replay/fallback
    local dest="$1"
    cp -r "${RUST_DIR}/src" "${dest}/src_snapshot"
}

_record_diff() {
    # Record the diff between pre and post snapshots
    local round_dir="$1"
    local pre="${round_dir}/src_pre"
    local post="${RUST_DIR}/src"
    if [ -d "$pre" ]; then
        diff -ru "$pre" "$post" > "${round_dir}/code_changes.diff" 2>/dev/null || true
        rm -rf "$pre"
    fi
}

_save_pre_snapshot() {
    # Save src/ before fixes for diff recording
    local round_dir="$1"
    cp -r "${RUST_DIR}/src" "${round_dir}/src_pre"
}

# ── Token cost extraction from stream-json output ──
extract_cost() {
    local outfile="$1"
    python3 -c "
import json, sys
total_in = total_out = cache_read = cache_create = 0
cost = 0.0
with open('$outfile') as f:
    for line in f:
        try: d = json.loads(line.strip())
        except: continue
        if d.get('type') == 'result':
            u = d.get('usage', {})
            total_in += u.get('input_tokens', 0)
            total_out += u.get('output_tokens', 0)
            cache_read += u.get('cache_read_input_tokens', 0)
            cache_create += u.get('cache_creation_input_tokens', 0)
            cost += d.get('total_cost_usd', 0)
print(f'{total_in}\t{total_out}\t{cache_read}\t{cache_create}\t{cost:.4f}')
" 2>/dev/null || printf '0\t0\t0\t0\t0.0000'
}

# ── Stats file ──
STATS_FILE="${OUTDIR}/round_stats.tsv"
if [ ! -f "$STATS_FILE" ]; then
    printf "round\tprev_fails\tfails\tpassed\ttotal\tgoals\tin_tokens\tout_tokens\tcache_read\tcache_create\tcost_usd\n" > "$STATS_FILE"
fi

# ── Helpers ──
parse_fail_count() {
    local report="$1"
    local tf ce
    tf=$(grep -m1 '^Tests failed:' "$report" 2>/dev/null | awk '{print $3}' || echo 0)
    ce=$(grep -m1 '^Configs with compile error:' "$report" 2>/dev/null | awk '{print $5}' || echo 0)
    echo $(( ${tf:-0} + ${ce:-0} ))
}

parse_pass_count() {
    local report="$1"
    grep -m1 '^Tests passed:' "$report" 2>/dev/null | awk '{print $3}' || echo 0
}

run_diff_tests() {
    local report_dest="$1"
    bash "${DIFFTEST_SCRIPT:-${HARNESS}/scripts/run_difftest.sh}" "$report_dest"
}

generate_compact_context() {
    local round_dir="$1"
    local c_out="${DIFFGEN}/c_output.txt"
    local r_out="${DIFFGEN}/r_output.txt"

    if [ -f "$c_out" ] && [ -f "$r_out" ]; then
        python3 "${HARNESS}/scripts/make_difffix_context.py" \
            "$c_out" "$r_out" "${RUST_DIR}/src" \
            --libm "${TEST_CASE_DIR}" --max-funcs "${MAX_GOALS}" \
            > "${round_dir}/compact_divergences.md" 2>/dev/null || true
        # Also place in OUTDIR so analysis prompt finds it
        cp "${round_dir}/compact_divergences.md" "${OUTDIR}/compact_divergences.md" 2>/dev/null || true
    fi
}

# ── Baseline snapshot ──
BASELINE_SNAPSHOT="${OUTDIR}/rounds/0/src_snapshot"

# ── Baseline (round 0) ──
ROUND0="${OUTDIR}/rounds/0"
mkdir -p "$ROUND0"

if [ ! -f "${ROUND0}/.done" ]; then
    echo "--- Baseline: running differential tests ---"
    run_diff_tests "${ROUND0}/diff_report.txt" 2>&1 | tail -5 || true
    touch "${ROUND0}/.done"
fi

fail_total=$(parse_fail_count "${ROUND0}/diff_report.txt")
pass_total=$(parse_pass_count "${ROUND0}/diff_report.txt")
if [ "$fail_total" -eq 0 ]; then
    echo "ALL TESTS PASS — nothing to fix!"
    exit 0
fi
echo "Baseline: ${pass_total} passed, ${fail_total} failures"
total_tests=$((pass_total + fail_total))

# Record baseline in stats
if ! grep -q "^0	" "$STATS_FILE" 2>/dev/null; then
    printf "0\t-\t%s\t%s\t%s\t-\t-\t-\t-\t-\t-\n" \
        "$fail_total" "$pass_total" "$total_tests" >> "$STATS_FILE"
fi
echo ""

prev_fails="$fail_total"
rolled_back_round=""
last_good_report=0

# ── Determine starting round ──
start_round=1
for dir in "${OUTDIR}/rounds"/*/; do
    [ -d "$dir" ] || continue
    n=$(basename "$dir")
    [[ "$n" =~ ^[0-9]+$ ]] || continue
    [ "$n" -eq 0 ] && continue
    if [ -f "${dir}/.done" ]; then
        start_round=$((n + 1))
        prev_fails=$(parse_fail_count "${dir}/diff_report.txt")
    fi
done

stall=0
[ -f "${OUTDIR}/stall_count" ] && stall=$(cat "${OUTDIR}/stall_count")

# ── Main loop ──
for round in $(seq "$start_round" "$MAX_ROUNDS"); do
    ROUND_DIR="${OUTDIR}/rounds/${round}"
    STEPS_DIR="${ROUND_DIR}/steps"
    mkdir -p "$ROUND_DIR" "$STEPS_DIR"

    round_in=0 round_out=0 round_cache_read=0 round_cache_create=0 round_cost="0.0000"
    round_goals=0

    echo ""
    echo "========================================"
    echo "DIFF-FIX ROUND ${round}/${MAX_ROUNDS}"
    echo "========================================"

    # Get the report to use: if we rolled back, use the round before the failed one
    if [ -n "${rolled_back_round:-}" ]; then
        PREV_REPORT="${OUTDIR}/rounds/$((rolled_back_round - 1))/diff_report.txt"
    else
        PREV_REPORT="${OUTDIR}/rounds/$((round-1))/diff_report.txt"
    fi

    # ── Step 1: Generate compact context from latest diff ──
    if [ ! -f "${ROUND_DIR}/.step1_done" ]; then
        echo "--- Step 1: Generate compact divergence context ---"
        generate_compact_context "$ROUND_DIR"
        if [ -f "${ROUND_DIR}/compact_divergences.md" ]; then
            n_funcs=$(grep -c "^## " "${ROUND_DIR}/compact_divergences.md" 2>/dev/null || echo 0)
            echo "  ${n_funcs} diverging functions extracted"
        else
            echo "  No compact context (using full report)"
        fi
        touch "${ROUND_DIR}/.step1_done"
    fi

    # ── Step 2: Analysis → goal files ──
    if [ ! -f "${ROUND_DIR}/.step2_done" ]; then
        echo "--- Step 2: Analyzing failures ---"

        # Build feedback based on mode
        HISTORY_FEEDBACK=""

        # Default: if a rollback happened (rolled_back_round set), feed the
        # failed attempt's diff + failures alongside the current (good) report.
        if [ -n "${rolled_back_round:-}" ] && [ "$round" -gt 1 ]; then
            _rb_dir="${OUTDIR}/rounds/${rolled_back_round}"
            _rb_diff=$(cat "${_rb_dir}/code_changes.diff" 2>/dev/null | head -200 || true)
            _rb_failures=$(sed -n '/MISMATCH\|MISSING/,/SUMMARY\|FUNCTION LOC/p' \
                "${_rb_dir}/diff_report.txt" 2>/dev/null | head -40 || true)
            HISTORY_FEEDBACK="
## FAILED ATTEMPT (round ${rolled_back_round} — rolled back, made things worse)

These code changes were tried and then reverted because they increased failures:
\`\`\`diff
${_rb_diff}
\`\`\`

Failures after that attempt:
${_rb_failures}

Do NOT repeat this approach. Try a different strategy for these functions.
"
            rolled_back_round=""
        fi

        # ReAct mode: additionally accumulate ALL previous rounds' history
        if [ "${REACT_MODE}" -eq 1 ] && [ "$round" -gt 1 ]; then
            echo "  (ReAct mode: building full history)"
            REACT_HISTORY="
## FIX HISTORY (all previous rounds)
Review what was tried before. Learn from successes and failures.
"
            for _hr in $(seq 1 $((round - 1))); do
                _hr_dir="${OUTDIR}/rounds/${_hr}"
                [ -d "$_hr_dir" ] || continue
                _hr_report="${_hr_dir}/diff_report.txt"
                _hr_prev_report="${OUTDIR}/rounds/$((_hr - 1))/diff_report.txt"
                [ -f "$_hr_report" ] || continue
                _hr_prev_fails=$(parse_fail_count "$_hr_prev_report")
                _hr_fails=$(parse_fail_count "$_hr_report")
                _hr_diff=$(cat "${_hr_dir}/code_changes.diff" 2>/dev/null | head -150 || true)
                _hr_failures=$(sed -n '/MISMATCH\|MISSING/,/SUMMARY\|FUNCTION LOC/p' "$_hr_report" 2>/dev/null | head -30 || true)
                if [ "$_hr_fails" -lt "$_hr_prev_fails" ]; then
                    _hr_verdict="IMPROVED (${_hr_prev_fails} -> ${_hr_fails})"
                elif [ "$_hr_fails" -gt "$_hr_prev_fails" ]; then
                    _hr_verdict="REGRESSED (${_hr_prev_fails} -> ${_hr_fails})"
                else
                    _hr_verdict="NO CHANGE (${_hr_fails})"
                fi
                _hr_goals=""
                if [ -d "${_hr_dir}/steps" ]; then
                    for _gf in "${_hr_dir}/steps"/goal_*.md; do
                        [ -f "$_gf" ] || continue
                        _hr_goals="${_hr_goals}
$(cat "$_gf")
"
                    done
                fi
                REACT_HISTORY="${REACT_HISTORY}
### Round ${_hr}: ${_hr_verdict}
${_hr_goals:+Analysis goals:
${_hr_goals}}
Code changes:
\`\`\`diff
${_hr_diff}
\`\`\`
Failures after this round:
${_hr_failures}
"
            done
            HISTORY_FEEDBACK="${HISTORY_FEEDBACK}${REACT_HISTORY}"
        fi

        ANALYZE_PROMPT="Read the diff report and compact divergences, then generate fix goals.

Working directory: ${OUTDIR}
Diff report: ${PREV_REPORT}
Compact divergences: ${ROUND_DIR}/compact_divergences.md
Goal output directory: ${STEPS_DIR}/
Rust source: ${RUST_DIR}/
C source: ${TEST_CASE_DIR}/
${HISTORY_FEEDBACK}
$(cat "${EXPANDED_PROMPTS_DIR}/analyze.md")
"
        # cd into OUTDIR so Claude CLI picks up .claude/settings.json
        (cd "${OUTDIR}" && run_analysis "$ANALYZE_PROMPT" "${ROUND_DIR}/analysis_output" "$VERBOSE")

        # Count goals
        goal_count=$(ls -1 "${STEPS_DIR}"/goal_*.md 2>/dev/null | wc -l)
        echo "  Goals generated: ${goal_count}"

        if [ "$goal_count" -eq 0 ]; then
            echo "  No goals — stopping."
            break
        fi

        # Cap goals
        if [ "$goal_count" -gt "$MAX_GOALS" ]; then
            echo "  Capping to ${MAX_GOALS} goals"
            ls -1 "${STEPS_DIR}"/goal_*.md | sort -V | tail -n +$((MAX_GOALS + 1)) | while read -r f; do
                mv "$f" "${f}.skipped"
            done
        fi

        touch "${ROUND_DIR}/.step2_done"
    fi

    # ── Step 3: Snapshot + Fix each goal ──
    if [ ! -f "${ROUND_DIR}/.step3_done" ]; then
        echo "--- Step 3: Snapshot & Fixing ---"

        # Save src/ before fixes for diff recording
        _save_pre_snapshot "$ROUND_DIR"

        for goal_file in "${STEPS_DIR}"/goal_*.md; do
            [ -f "$goal_file" ] || continue
            goal_name=$(basename "$goal_file" .md)
            echo "  Fixing: ${goal_name}"

            FIX_PROMPT="Fix the Rust code based on this goal.

Working directory: ${RUST_DIR}
Goal: ${goal_file}
Rust source: ${RUST_DIR}/
C source: ${TEST_CASE_DIR}/

$(cat "${EXPANDED_PROMPTS_DIR}/fixer.md")

Goal content:
$(cat "$goal_file")
"
            # cd into RUST_DIR so Claude CLI picks up .claude/settings.json
            (cd "${RUST_DIR}" && run_codegen "$FIX_PROMPT" "${STEPS_DIR}/${goal_name}_fix_output" "$VERBOSE")
        done

        # Record diff and save post-fix snapshot
        _record_diff "$ROUND_DIR"
        _snapshot_src "$ROUND_DIR"
        echo "  Diff saved: ${ROUND_DIR}/code_changes.diff"

        touch "${ROUND_DIR}/.step3_done"
    fi

    # ── Step 4: Re-test ──
    if [ ! -f "${ROUND_DIR}/.step4_done" ]; then
        echo "--- Step 4: Re-testing ---"
        run_diff_tests "${ROUND_DIR}/diff_report.txt" 2>&1 | tail -5 || true
        touch "${ROUND_DIR}/.step4_done"
    fi

    fail_total=$(parse_fail_count "${ROUND_DIR}/diff_report.txt")
    pass_total=$(parse_pass_count "${ROUND_DIR}/diff_report.txt")
    echo "After round ${round}: ${pass_total} passed, ${fail_total} failures (was ${prev_fails})"

    # ── Collect token costs for this round ──
    round_goals=$(ls -1 "${STEPS_DIR}"/goal_*.md 2>/dev/null | wc -l)
    _sum_tokens() {
        local _in=0 _out=0 _cr=0 _cc=0 _cost="0"
        for outf in "$@"; do
            [ -f "$outf" ] || continue
            IFS=$'\t' read -r i o cr cc c <<< "$(extract_cost "$outf")"
            _in=$((_in + i)); _out=$((_out + o)); _cr=$((_cr + cr)); _cc=$((_cc + cc))
            _cost=$(python3 -c "print(f'{$_cost + $c:.4f}')" 2>/dev/null || echo "$_cost")
        done
        printf '%s\t%s\t%s\t%s\t%s' "$_in" "$_out" "$_cr" "$_cc" "$_cost"
    }
    all_outputs=("${ROUND_DIR}/analysis_output")
    for gf in "${STEPS_DIR}"/goal_*_fix_output; do
        [ -f "$gf" ] && all_outputs+=("$gf")
    done
    IFS=$'\t' read -r round_in round_out round_cache_read round_cache_create round_cost <<< "$(_sum_tokens "${all_outputs[@]}")"

    # Record stats
    if ! grep -q "^${round}	" "$STATS_FILE" 2>/dev/null; then
        printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
            "$round" "$prev_fails" "$fail_total" "$pass_total" "$total_tests" \
            "$round_goals" "$round_in" "$round_out" "$round_cache_read" "$round_cache_create" \
            "$round_cost" >> "$STATS_FILE"
    fi

    [ "$fail_total" -eq 0 ] && { echo "ALL TESTS PASS!"; break; }

    # ── Step 5: Stall check ──
    if [ "$fail_total" -gt "$prev_fails" ]; then
        stall=$((stall + 1))
        echo "${stall}" > "${OUTDIR}/stall_count"
        echo "  Regression (${prev_fails} -> ${fail_total}). Rolling back."
        # Rollback: restore previous round's snapshot
        _last_good_snap="${OUTDIR}/rounds/$((round - 1))/src_snapshot"
        if [ -d "$_last_good_snap" ]; then
            rm -rf "${RUST_DIR}/src"
            cp -r "$_last_good_snap" "${RUST_DIR}/src"
            echo "  Restored src/ from round $((round - 1)) snapshot."
        fi
        rolled_back_round="$round"
        # Don't update prev_fails — we rolled back
        [ "$stall" -ge "$STALL_LIMIT" ] && { echo "Stalled."; break; }
    elif [ "$fail_total" -eq "$prev_fails" ]; then
        stall=$((stall + 1))
        echo "${stall}" > "${OUTDIR}/stall_count"
        echo "  No progress (stall ${stall}/${STALL_LIMIT})"
        prev_fails="$fail_total"
        [ "$stall" -ge "$STALL_LIMIT" ] && { echo "Stalled."; break; }
    else
        stall=0
        echo "0" > "${OUTDIR}/stall_count"
        prev_fails="$fail_total"
    fi
    touch "${ROUND_DIR}/.done"
done

# ── Print summary ──
echo ""
echo "========================================"
echo "DIFF-FIX LOOP COMPLETE"
echo "========================================"
echo "Final: ${pass_total} passed, ${fail_total} failures"
echo ""
echo "Per-round stats: ${STATS_FILE}"
column -t -s $'\t' "$STATS_FILE" 2>/dev/null || cat "$STATS_FILE"
echo ""
echo "Per-round diffs saved in: ${OUTDIR}/rounds/*/code_changes.diff"
echo "Per-round snapshots in:   ${OUTDIR}/rounds/*/src_snapshot/"

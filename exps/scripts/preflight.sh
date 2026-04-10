#!/usr/bin/env bash
set -euo pipefail

# preflight.sh — quick smoke tests before the real pipeline runs.
#
# Catches misconfigurations (bad paths, broken toolchain, empty coverage,
# missing symbols) in seconds instead of hours into a long run.
#
# Usage: scripts/preflight.sh            (sources config via .project)
#        scripts/preflight.sh <project>  (explicit project name)
#
# Exit code: 0 = all pass, 1 = critical failure, 2 = warnings only

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS_DIR="$(dirname "$SCRIPT_DIR")"

if [ $# -ge 1 ]; then
  export PROJECT_DIR="${HARNESS_DIR}/projects/$1"
fi

source "${HARNESS_DIR}/config.sh"

# ── terminal helpers ──────────────────────────────────────────────────────────
if [ -t 1 ]; then
  B=$'\033[1m'; R=$'\033[0m'
  GR=$'\033[0;32m'; RD=$'\033[0;31m'; YL=$'\033[0;33m'; DM=$'\033[2m'
else
  B=''; R=''; GR=''; RD=''; YL=''; DM=''
fi

_pass=0; _warn=0; _fail=0
pass() { _pass=$((_pass+1)); printf "  ${GR}PASS${R}  %s\n" "$*"; }
warn() { _warn=$((_warn+1)); printf "  ${YL}WARN${R}  %s\n" "$*"; }
fail() { _fail=$((_fail+1)); printf "  ${RD}FAIL${R}  %s\n" "$*"; }
info() { printf "  ${DM}      %s${R}\n" "$*"; }
section() { echo ""; printf "${B}── %s ──${R}\n" "$*"; }

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo ""
echo "${B}Preflight checks${R}  ·  project: $(cat "${WORK_DIR}/.project" 2>/dev/null || echo "${PROJECT_DIR##*/}")"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
section "Toolchain"
# ═══════════════════════════════════════════════════════════════════════════════

for tool in "$CC" "$LLVM_PROFDATA" "$LLVM_COV" cargo; do
  if command -v "$tool" &>/dev/null; then
    ver=$("$tool" --version 2>/dev/null | head -1 || echo "?")
    pass "$tool  ${DM}($ver)${R}"
  else
    fail "$tool not found"
  fi
done

for cmd in "$CODE_GEN_CMD" "$ANALYSIS_CMD"; do
  if command -v "$cmd" &>/dev/null; then
    pass "$cmd available"
  else
    warn "$cmd not found — AI phases will fail"
  fi
done

# ═══════════════════════════════════════════════════════════════════════════════
section "Config"
# ═══════════════════════════════════════════════════════════════════════════════

# TEST_CASE_DIR
if [ -d "$TEST_CASE_DIR" ]; then
  _nc=$(find $C_SRC_DIRS -name '*.c' -type f 2>/dev/null | wc -l)
  _nh=$(find $C_INCLUDE_DIRS -name '*.h' -type f 2>/dev/null | wc -l)
  pass "TEST_CASE_DIR exists  ${DM}(${_nc} .c, ${_nh} .h files)${R}"
  [ "$_nc" -eq 0 ] && fail "C_SRC_DIRS has 0 .c files: $C_SRC_DIRS"
else
  fail "TEST_CASE_DIR not found: $TEST_CASE_DIR"
fi

# get_compile_args / get_c_sources
_args=$(get_compile_args "default" 2>/dev/null) || _args=""
if [ -n "$_args" ]; then
  pass "get_compile_args(default) → ${DM}${_args:0:60}${R}"
else
  warn "get_compile_args(default) returned empty"
fi

_srcs=$(get_c_sources 2>/dev/null) || _srcs=""
if [ -n "$_srcs" ]; then
  _n=$(echo "$_srcs" | wc -w)
  pass "get_c_sources() → ${DM}${_n} files${R}"
else
  fail "get_c_sources() returned empty"
fi

# Multi-config
read_configs 2>/dev/null || true
if [ "${#CONFIGS[@]}" -gt 1 ]; then
  pass "Multi-config: ${#CONFIGS[@]} configs"
else
  pass "Single config"
fi

# RUST_DIR
if [ -d "$RUST_DIR" ] && [ -f "${RUST_DIR}/Cargo.toml" ]; then
  pass "RUST_DIR exists with Cargo.toml"
else
  info "RUST_DIR not yet created (will be created during transpile)"
fi

# ═══════════════════════════════════════════════════════════════════════════════
section "C compilation"
# ═══════════════════════════════════════════════════════════════════════════════

if [ -n "$_srcs" ] && [ -n "$_args" ]; then
  cd "$TMPDIR"
  _compile_ok=1
  if $CC $_args -fsyntax-only $_srcs 2>"${TMPDIR}/cc_err.txt"; then
    pass "C sources compile (syntax check)"
  else
    fail "C sources fail to compile"
    info "$(head -5 "${TMPDIR}/cc_err.txt")"
    _compile_ok=0
  fi
else
  info "Skipped (no sources or compile args)"
  _compile_ok=0
fi

# ═══════════════════════════════════════════════════════════════════════════════
section "Coverage pipeline"
# ═══════════════════════════════════════════════════════════════════════════════

# Build a minimal binary with coverage instrumentation and check the full
# profraw → profdata → llvm-cov → branch data chain.

if [ "$_compile_ok" -eq 1 ]; then
  cd "$TMPDIR"

  # Compile library objects with coverage
  _cov_ok=1
  if $CC $_args -fprofile-instr-generate -fcoverage-mapping \
       -c $_srcs 2>/dev/null; then
    pass "Coverage-instrumented compilation"
  else
    fail "Coverage-instrumented compilation failed"
    _cov_ok=0
  fi

  if [ "$_cov_ok" -eq 1 ]; then
    ar rcs libpreflight.a ./*.o 2>/dev/null
    rm -f ./*.o

    # Minimal main
    echo 'int main(void){return 0;}' > stub.c
    $CC $_args -c stub.c -o stub.o 2>/dev/null
    if $CC -fprofile-instr-generate stub.o libpreflight.a \
         -lm -lpthread -ldl -lrt -lutil \
         -o preflight_bin 2>/dev/null; then

      LLVM_PROFILE_FILE="${TMPDIR}/pf.profraw" ./preflight_bin >/dev/null 2>&1 || true

      if [ -f "${TMPDIR}/pf.profraw" ]; then
        pass "profraw generated"

        if $LLVM_PROFDATA merge -sparse "${TMPDIR}/pf.profraw" \
             -o "${TMPDIR}/pf.profdata" 2>/dev/null; then
          pass "profdata merge"

          _cov_out=$($LLVM_COV show ./preflight_bin \
            -instr-profile="${TMPDIR}/pf.profdata" \
            --show-branches=count 2>/dev/null || true)
          _nbranch=$(echo "$_cov_out" | grep -c 'Branch ' || true)

          if [ "$_nbranch" -gt 0 ]; then
            pass "llvm-cov branch output: ${_nbranch} branches detected"
          else
            warn "llvm-cov produced 0 branch lines (stub has no branches — this is expected)"
            info "Full pipeline works, but verify with real tests."
          fi
        else
          fail "profdata merge failed"
        fi
      else
        fail "profraw not generated — profiling runtime may be missing"
      fi
    else
      warn "Link failed (may need extra -l flags in config)"
    fi
  fi
else
  info "Skipped (C compilation failed)"
fi

# ═══════════════════════════════════════════════════════════════════════════════
section "Rust project"
# ═══════════════════════════════════════════════════════════════════════════════

if [ -d "$RUST_DIR" ] && [ -f "${RUST_DIR}/Cargo.toml" ]; then
  cd "$RUST_DIR"
  if cargo check --lib 2>"${TMPDIR}/cargo_err.txt"; then
    pass "cargo check --lib"
  else
    _nerrs=$(grep -c '^error' "${TMPDIR}/cargo_err.txt" || true)
    fail "cargo check --lib: ${_nerrs} error(s)"
    info "$(tail -3 "${TMPDIR}/cargo_err.txt")"
  fi

  # Check for _rs wrappers (symbol export)
  if [ -f src/lib.rs ]; then
    _nwrap=$(grep -c '#\[no_mangle\]' src/lib.rs || true)
    if [ "$_nwrap" -gt 0 ]; then
      pass "#[no_mangle] wrappers: ${_nwrap} found in lib.rs"
    else
      warn "No #[no_mangle] wrappers in lib.rs — Rust functions won't be callable from C"
    fi
  fi
else
  info "Rust project not yet created (OK before transpile)"
fi

# ═══════════════════════════════════════════════════════════════════════════════
section "Test suite"
# ═══════════════════════════════════════════════════════════════════════════════

_test_suite="${TESTGEN_WORKDIR}/test_suite.c"
if [ -f "$_test_suite" ]; then
  _ntids=$(grep -oP '(?<=#ifdef RUN_)[A-Za-z_0-9]+' "$_test_suite" | sort -u | wc -l)
  if [ "$_ntids" -gt 0 ]; then
    pass "test_suite.c: ${_ntids} test ID(s) found"

    # Quick compile check with all tests enabled
    cd "$TMPDIR"
    _macros=""
    for tid in $(grep -oP '(?<=#ifdef RUN_)[A-Za-z_0-9]+' "$_test_suite" | sort -u); do
      _macros="$_macros -DRUN_${tid}"
    done
    if $CC $_args $_macros -fsyntax-only "$_test_suite" 2>"${TMPDIR}/ts_err.txt"; then
      pass "test_suite.c compiles (syntax check)"
    else
      fail "test_suite.c fails to compile"
      info "$(head -3 "${TMPDIR}/ts_err.txt")"
    fi
  else
    warn "test_suite.c has 0 #ifdef RUN_ guards"
  fi
else
  info "test_suite.c not yet generated (OK before testgen)"
fi

# Difftest
_difftest="${DIFFGEN_WORKDIR}/difftest_suite.c"
if [ -f "$_difftest" ]; then
  _ndiffs=$(grep -oP '(?<=#ifdef RUN_)[A-Za-z_0-9]+' "$_difftest" | sort -u | wc -l)
  pass "difftest_suite.c: ${_ndiffs} test ID(s)"
else
  info "difftest_suite.c not yet generated (OK before diffgen)"
fi

# ═══════════════════════════════════════════════════════════════════════════════
section "Coverage data"
# ═══════════════════════════════════════════════════════════════════════════════

_branches="${TESTGEN_WORKDIR}/branches.md"
if [ -f "$_branches" ]; then
  _nb=$(grep -c -v '^\s*\(#\|$\)' "$_branches" 2>/dev/null) || _nb=0
  if [ "$_nb" -gt 0 ]; then
    pass "branches.md: ${_nb} branches"
  else
    fail "branches.md exists but has 0 entries — branch coverage will be meaningless"
  fi
else
  info "branches.md not yet extracted (OK before first testgen round)"
fi

_funcs="${TESTGEN_WORKDIR}/functions.md"
if [ -f "$_funcs" ]; then
  _nf=$(grep -c -v '^\s*\(#\|$\)' "$_funcs" 2>/dev/null) || _nf=0
  if [ "$_nf" -gt 0 ]; then
    pass "functions.md: ${_nf} functions"
  else
    fail "functions.md exists but has 0 entries — function coverage will be meaningless"
  fi
else
  info "functions.md not yet extracted"
fi

# Feedback files
_fb_dir="${TESTGEN_WORKDIR}/feedback"
if [ -d "$_fb_dir" ]; then
  _nfb=$(ls "${_fb_dir}"/*_feedback 2>/dev/null | wc -l || true)
  _n_err=$(grep -rl '^COMPILE_ERROR\|^NO_COVERAGE' "${_fb_dir}/" 2>/dev/null | wc -l || true)
  if [ "$_nfb" -gt 0 ]; then
    if [ "$_n_err" -eq "$_nfb" ]; then
      fail "All ${_nfb} feedback file(s) are COMPILE_ERROR or NO_COVERAGE — zero coverage data"
    elif [ "$_n_err" -gt 0 ]; then
      warn "${_n_err}/${_nfb} feedback files have no coverage data"
    else
      pass "feedback: ${_nfb} file(s), all contain coverage data"
    fi
  else
    info "No feedback files yet"
  fi
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
printf "── ${B}Results${R}: "
printf "${GR}%d pass${R}  " "$_pass"
[ "$_warn" -gt 0 ] && printf "${YL}%d warn${R}  " "$_warn"
[ "$_fail" -gt 0 ] && printf "${RD}%d fail${R}  " "$_fail"
echo ""

if [ "$_fail" -gt 0 ]; then
  echo ""
  echo "${RD}Fix the failures above before running the pipeline.${R}"
  exit 1
elif [ "$_warn" -gt 0 ]; then
  echo ""
  echo "${YL}Warnings detected — pipeline may hit issues. Review above.${R}"
  exit 2
else
  echo ""
  echo "${GR}All clear — safe to run.${R}"
  exit 0
fi

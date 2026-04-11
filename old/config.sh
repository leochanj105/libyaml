#!/usr/bin/env bash
# config.sh — central configuration for the progress harness.
# Source this file from any script: source "$(dirname "$0")/../config.sh"
#
# All paths are configurable. Override via env vars or project config_overrides.sh.
# This file contains ONLY generic logic — no project-specific content.

HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# =========================================================================
# Source project overrides FIRST so everything below picks up overridden values.
#
# Resolution order:
#   1. PROJECT_DIR env var (if set)
#   2. $HARNESS_DIR/.project marker (written by prepare.sh)
#   3. No overrides (use defaults / env vars only)
# =========================================================================
if [ -z "${PROJECT_DIR:-}" ] && [ -f "${HARNESS_DIR}/.project" ]; then
    PROJECT_DIR=$(cat "${HARNESS_DIR}/.project")
fi

if [ -n "${PROJECT_DIR:-}" ]; then
    # Resolve: PROJECT_DIR can be an absolute path or a bare name (legacy)
    if [ -d "$PROJECT_DIR" ]; then
        _overrides="${PROJECT_DIR}/config_overrides.sh"
    elif [ -d "${HARNESS_DIR}/projects/${PROJECT_DIR}" ]; then
        _overrides="${HARNESS_DIR}/projects/${PROJECT_DIR}/config_overrides.sh"
    else
        _overrides=""
    fi
    [ -n "$_overrides" ] && [ -f "$_overrides" ] && source "$_overrides"
fi

# =========================================================================
# Project paths — MUST be set by project config_overrides.sh or env vars
# =========================================================================

# Root of the C source code
: "${TEST_CASE_DIR:=}"

# Transpiled Rust project directory
: "${RUST_DIR:=}"

# =========================================================================
# C source layout — override in config_overrides.sh for non-standard layouts
# =========================================================================

# Directories containing C source files (.c) to compile
: "${C_SRC_DIRS:=${TEST_CASE_DIR}/src}"

# Directories containing C headers (.h)
: "${C_INCLUDE_DIRS:=${TEST_CASE_DIR}/include}"

# =========================================================================
# Tool paths
# =========================================================================
: "${CC:=clang-21}"
: "${LLVM_PROFDATA:=llvm-profdata-21}"
: "${LLVM_COV:=llvm-cov-21}"

# =========================================================================
# Multi-config support (optional)
#
# For single-config projects: leave CONFIGS_FILE empty.
# For multi-config projects: set CONFIGS_FILE to a file listing one config
# name per line, and provide:
#   - get_compile_args <config>   — function returning CC flags
#   - get_cargo_features <config> — function returning cargo features
# These should be defined in config_overrides.sh.
# =========================================================================
: "${CONFIGS_FILE:=}"

# =========================================================================
# Working directory — all runtime output goes here.
# Set in config_overrides.sh to keep output next to your project.
# =========================================================================
: "${WORK_DIR:=${HARNESS_DIR}/work}"
: "${TESTGEN_WORKDIR:=${WORK_DIR}/testgen}"
: "${DIFFGEN_WORKDIR:=${WORK_DIR}/diffgen}"
: "${DIFFFIX_WORKDIR:=${WORK_DIR}/difffix}"

# =========================================================================
# Coverage guidance modes for test generation
#   "function"  — ensure all internal C functions are called (default)
#   "branch"    — LLVM branch coverage guided
#   "bounds"    — buffer access contract coverage
#   Combine with comma: "function,branch" or "function,branch,bounds"
# =========================================================================
: "${COVERAGE_MODES:=function}"

# =========================================================================
# Modular transpilation (optional)
#   ""            = single-unit: all C source files transpiled in one AI call
#   "auto"        = detect modules as immediate subdirectories of C_SRC_DIRS
#   "mod1,mod2"   = explicit comma-separated list; each module's .c files are
#                   found under <C_SRC_DIRS>/<module>/*.c
# When set, each module is transpiled independently to src/<module>.rs and
# src/lib.rs is assembled afterward to tie them together.
# =========================================================================
: "${MODULES:=}"

# =========================================================================
# Pluggable scripts (leave empty for default behavior)
# =========================================================================

# Diff test script — if set, called instead of generated difftest_suite.c flow.
# Contract: takes $1 = output report path. Uses env vars from config.sh.
# Output format: per-test PASS/FAIL lines + SUMMARY section.
: "${DIFFTEST_SCRIPT:=}"

# Default test script — final validation after fix loop, not used as feedback.
# Same contract as DIFFTEST_SCRIPT.
: "${DEFAULT_TEST_SCRIPT:=}"

# Judger test script — held-out final quality evaluation.
# Run ONCE at the very end of run_all.sh, AFTER all testgen/transpile/fix phases.
# The harness guarantees AI models never see JUDGER_DIR during the pipeline by
# chmod-locking it to 000 around every AI call (see scripts/ai_runner.sh).
#
# Contract: same as DIFFTEST_SCRIPT — takes $1 = output report path.
# Exit code: 0 = all pass, non-zero = failures.
: "${JUDGER_SCRIPT:=}"

# Directory containing judger test inputs / expected outputs.
# The harness locks this directory (chmod 000) before every AI invocation
# and restores permissions afterward.  Set this even if JUDGER_SCRIPT
# already knows the path — the lock needs the path to be explicit here.
: "${JUDGER_DIR:=}"

# Build script for C code — if set, called to compile C source.
# Contract: takes $1 = output .so or .a path. Uses C_SRC_DIRS, C_INCLUDE_DIRS.
# If empty, harness uses a generic compile-all-*.c approach.
: "${BUILD_C_SCRIPT:=}"

# Build script for Rust code — if set, called to compile Rust.
# Contract: takes $1 = output dir. Uses RUST_DIR.
# If empty, harness uses `cargo build --release`.
: "${BUILD_RUST_SCRIPT:=}"

# =========================================================================
# Limits
# =========================================================================
: "${MAX_ROUNDS:=5}"
: "${STALL_LIMIT:=2}"
: "${MAX_GOALS:=5}"
: "${MAX_RETRIES:=2}"
: "${MAX_JOBS:=$(nproc 2>/dev/null || echo 4)}"
: "${MAX_FIX_ROUNDS:=5}"

# =========================================================================
# Difffix behavior
#   REACT_MODE=0 (default): rollback on regression + failed attempt feedback
#   REACT_MODE=1: additionally accumulate full ReAct history (all rounds)
# =========================================================================
: "${REACT_MODE:=0}"

# =========================================================================
# Test isolation — fork wrapper for independent test execution
#   Prevents crashes/timeouts in one test from killing others.
#   Applied to difftest_suite.c during diffgen, not to testgen test_suite.c.
#   TEST_TIMEOUT_SEC: per-test timeout in seconds (0 = disabled)
# =========================================================================
: "${TEST_TIMEOUT_SEC:=2}"

# =========================================================================
# Rust compilation
#   OPT_LEVEL: Rust optimization level (0 recommended to avoid LLVM
#   transformations that cause false infinite loops like exp2→powf→exp2)
# =========================================================================
: "${RUST_OPT_LEVEL:=0}"

# =========================================================================
# Files to exclude from C compilation (doc-only, no compiled code)
# =========================================================================
: "${C_EXCLUDE_FILES:=cmplx.c|isfinite.c|isgreater.c|isgreaterequal.c|isinf.c|isless.c|islessequal.c|islessgreater.c|isnan.c|isnormal.c|isunordered.c|fenv.c}"

# =========================================================================
# AI commands — configurable per project
#   CODE_GEN_CMD  — binary used for code generation (transpile, testgen, fix)
#   ANALYSIS_CMD  — binary used for analysis/planning (strategy, analyze failures)
# Override in config_overrides.sh or env to switch e.g. CODE_GEN_CMD=claude
# =========================================================================
: "${CODE_GEN_CMD:=codex}"
: "${ANALYSIS_CMD:=claude}"
export CODE_GEN_CMD ANALYSIS_CMD

# =========================================================================
# Retry / rate-limit handling
# =========================================================================
: "${CLAUDE_RETRY_INTERVAL:=300}"
: "${CLAUDE_MAX_WAIT:=14400}"

# =========================================================================
# Enable retry wrappers (both claude and codex)
# =========================================================================
export PATH="${HARNESS_DIR}/scripts/codex_wrapper:${HARNESS_DIR}/scripts/claude_wrapper:${PATH}"
export CLAUDE_RETRY_INTERVAL CLAUDE_MAX_WAIT

# =========================================================================
# Generic helpers
# =========================================================================

# Read configs into CONFIGS array. For single-config projects, CONFIGS=("default").
read_configs() {
    CONFIGS=()
    if [ -n "$CONFIGS_FILE" ] && [ -f "$CONFIGS_FILE" ]; then
        while IFS= read -r line; do
            [[ -z "$line" || "$line" =~ ^# ]] && continue
            CONFIGS+=("$line")
        done < "$CONFIGS_FILE"
    else
        CONFIGS=("default")
    fi
}

# Get compiler flags for a config. Override in config_overrides.sh for multi-config.
# Default: just include dirs.
# Only define if not already defined by config_overrides.sh.
if ! declare -f get_compile_args >/dev/null 2>&1; then
get_compile_args() {
    local args=""
    for d in $C_INCLUDE_DIRS; do
        args="$args -I$d"
    done
    echo "$args"
}
fi

# Get cargo features for a config. Override in config_overrides.sh for multi-config.
# Default: empty (no features).
if ! declare -f get_cargo_features >/dev/null 2>&1; then
get_cargo_features() {
    echo ""
}
fi

# Enumerate all C source files.
if ! declare -f get_c_sources >/dev/null 2>&1; then
get_c_sources() {
    for d in $C_SRC_DIRS; do
        [ -d "$d" ] && find "$d" -name '*.c' -type f 2>/dev/null
    done
}
fi

# Expanded prompts directory (produced by prepare.sh)
EXPANDED_PROMPTS_DIR="${WORK_DIR}/prompts"

# Check that prepare.sh has been run
if [ ! -d "$EXPANDED_PROMPTS_DIR" ] || [ -z "$(ls -A "$EXPANDED_PROMPTS_DIR" 2>/dev/null)" ]; then
    echo "Warning: prompts not prepared. Run: ./prepare.sh <project_dir>" >&2
fi

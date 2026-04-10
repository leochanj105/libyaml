# Progress Harness — Roadmap & Usage Guide

## What This Is

A reusable harness for transpiling C libraries to Rust and iteratively fixing
the transpilation using AI-generated tests and differential testing.

## Pipeline Overview

```
C Library Source
    │
    ├─ Phase 1: Transpile (C → Rust)
    │      AI translates C source to Rust. Output: rust-baseline/
    │
    ├─ Phase 2: Test Generation (S1–S5 scenarios)
    │      AI generates test suites with different strategies.
    │      Coverage feedback (S4/S5) uses LLVM instrumentation.
    │
    ├─ Phase 2b: Prepare Rust for Test
    │      Adds test bridge (Rust-side wrappers for internal functions).
    │      Output: rust-baseline-test/
    │
    ├─ Phase 3a: Diffgen (prepare differential tests)
    │      Copies test suite, wraps with fork isolation, prepares C/Rust builds.
    │
    └─ Phase 3b: Difffix (iterative fix loop)
           Runs diff tests, analyzes failures, fixes Rust code.
           Repeats until all tests pass or stall limit hit.
```

## Test Generation Scenarios

The harness supports multiple test generation strategies, run as separate
"scenarios." Each scenario produces a different test suite for the same
library, allowing comparison of test quality.

### Scenario Design Pattern

Each scenario needs:
1. A **prompt** in `prompts/` (e.g., `s1_testgen.md`) — instructions for the AI
2. A **config** in `scenarios/<name>/config_overrides.sh` — rounds, coverage mode
3. A **runner script** (e.g., `02a_testgen_s1.sh`) — sets up env and calls the loop

### Example Scenarios (from libmcs experiments)

| Scenario | Strategy | Coverage Mode | Rounds | Description |
|----------|----------|--------------|--------|-------------|
| S1 | Naive one-shot | none | 1 | "Generate tests for this library" — no guidance |
| S2 | Explicit boundary | none | 1 | Lists function signatures, asks for coverage |
| S3 | Edge cases | none | 1 | Instructs to test NaN, Inf, ±0, denormals, boundaries |
| S4 | Function coverage | function | 1-5 | Coverage feedback loop: "these functions are uncovered" |
| S5 | Branch coverage | branch | 5 | Builds on S4, adds branch coverage feedback |
| S6 | Extended branch | branch | 5 | Builds on S5, 5 more rounds for diminishing returns |

### How Scenarios Build on Each Other

- S1-S3: independent, single-shot (different prompts, same library)
- S4: uses `run_testgen_loop.sh` with `COVERAGE_MODES=function`
- S5: copies S4's test suite, runs `run_testgen_loop.sh` with `COVERAGE_MODES=branch`
- S6: copies S5's test suite, runs 5 more branch coverage rounds

### Creating a New Scenario

```bash
# 1. Create scenario config
mkdir -p scenarios/s_custom
cat > scenarios/s_custom/config_overrides.sh << 'EOF'
: "${COVERAGE_MODES:=branch}"
: "${MAX_ROUNDS:=5}"
: "${STALL_LIMIT:=2}"
EOF

# 2. Create prompt
cat > prompts/s_custom_testgen.md << 'EOF'
You are a test generator for a C library.
... (instructions for the AI) ...
EOF

# 3. Create runner script
cat > 02x_testgen_custom.sh << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"
# ... set up env, call run_testgen_loop.sh ...
EOF
```

### Key Findings from libmcs Experiments

- **Function coverage alone is weak**: S4 achieves 100% function coverage
  but only 41.6% branch condition coverage — trivial inputs miss code paths.
- **Edge case prompts help**: S3 (prompt-based) achieves 66.1% branch coverage
  without any tooling, better than S4's 41.6%.
- **Branch feedback is strongest**: S5 reaches 84.0% branch coverage through
  iterative feedback, the highest of all scenarios.
- **Diminishing returns**: S6 (5 more rounds on S5) shows slower gains on
  the remaining hard-to-cover branches.
- **More tests ≠ better tests**: S2 has fewer tests (458) but 100% function
  coverage. S1 has more (785) but only 91% function coverage.

## Setting Up a New Project

### 1. Create project config

```bash
mkdir -p projects/myproject
cat > projects/myproject/config_overrides.sh << 'EOF'
TEST_CASE_DIR="/path/to/c/library"
RUST_DIR="/path/to/rust/output"
WORK_DIR="/path/to/work/directory"
C_SRC_DIRS="${TEST_CASE_DIR}/src"
C_INCLUDE_DIRS="${TEST_CASE_DIR}/include"
# Exclude files that are doc-only or have no compiled code:
C_EXCLUDE_FILES="doc_only1.c|doc_only2.c"
EOF
```

### 2. Prepare prompts

```bash
./prepare.sh projects/myproject
```

### 3. Extract function and branch lists

```bash
# Functions (from compiled .o files, not source parsing)
bash scripts/extract_functions.sh ${WORK_DIR}/work-functions.md

# Branches (from llvm-cov export, for coverage-guided testgen)
bash scripts/extract_branches.sh ${WORK_DIR}/work-branches.md
```

### 4. Generate tests

Run test generation with desired strategy. Each scenario generates a
`test_suite.c` in `${WORK_DIR}/testgen/`.

### 5. Prepare Rust for testing

```bash
bash scripts/prepare_rust_for_test.sh
```

This creates `rust-baseline-test/` from `rust-baseline/` by:
- Changing internal functions from `fn` to `pub(crate) fn` (visibility only)
- Adding `test_bridge.rs` (Rust-side wrappers matching C's test_bridge.c)
- Adding `mod test_bridge;` to `lib.rs`
- Verifying `cargo build` succeeds

### 6. Run differential testing + fixing

```bash
# Prepare diff test suite (wraps tests with fork isolation)
# Then run iterative fix loop
bash difffix/run_difffix_loop.sh
```

## Key Scripts

### Test Infrastructure

| Script | Purpose |
|--------|---------|
| `scripts/wrap_tests_independent.py` | Post-processes test_suite.c to run each test function in a fork. Crashes/timeouts in one test don't kill others. |
| `scripts/compare_outputs.py` | Compares C vs Rust output line by line. Skips section headers (`===`), FAULT lines. No deduplication — every test case counted. |
| `scripts/run_difftest.sh` | Full differential test: build C lib, build Rust lib, compile test against each, run, compare. No C fallback in Rust binary. |
| `scripts/prepare_rust_for_test.sh` | Creates rust-baseline-test with bridge wrappers for internal/static functions. |

### Coverage

| Script | Purpose |
|--------|---------|
| `scripts/branch_coverage.py` | Ground truth branch condition extraction and coverage measurement from `llvm-cov export` JSON. Each branch = 2 conditions (true/false). Covered = count > 0. |
| `scripts/extract_branches.sh` | Extracts branch data using llvm-cov export. |
| `scripts/extract_functions.sh` | Extracts function list from compiled .o files (not source parsing — avoids counting `#ifdef`-guarded phantom functions). |
| `scripts/build_and_cover.sh` | Compiles test suite with coverage, runs it, produces llvm-cov export JSON. |
| `scripts/measure_coverage.sh` | Convenience wrapper: builds, runs, reports coverage using llvm-cov report. |

### Diff-Fix Loop

| Script | Purpose |
|--------|---------|
| `difffix/run_difffix_loop.sh` | Main fix loop. Each round: analyze failures → generate goals → fix Rust → re-test → stall check. |
| `difffix/prompts/analyze.md` | Prompt for the failure analyzer (reads diff report, generates goal files). |
| `difffix/prompts/fixer.md` | Prompt for the code fixer (reads goals, edits Rust source). |

## Configuration Reference (config.sh)

### Required (set in config_overrides.sh)

| Variable | Description |
|----------|-------------|
| `TEST_CASE_DIR` | Root of C source code |
| `RUST_DIR` | Transpiled Rust project directory |
| `C_SRC_DIRS` | Directories with .c files |
| `C_INCLUDE_DIRS` | Directories with .h files |

### Difffix Behavior

| Variable | Default | Description |
|----------|---------|-------------|
| `REACT_MODE` | 0 | 0 = rollback on regression + failed attempt feedback. 1 = additionally accumulate full ReAct history. |
| `MAX_ROUNDS` | 5 | Max difffix rounds |
| `STALL_LIMIT` | 2 | Stop after N rounds with no progress |
| `MAX_GOALS` | 5 | Max fix goals per round |

### Test Isolation

| Variable | Default | Description |
|----------|---------|-------------|
| `TEST_TIMEOUT_SEC` | 2 | Per-test fork timeout in seconds |

### Rust Compilation

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_OPT_LEVEL` | 0 | Rust opt-level. 0 prevents LLVM optimizations that cause false infinite loops. |

### Coverage Tools

| Variable | Default | Description |
|----------|---------|-------------|
| `CC` | clang-21 | C compiler |
| `LLVM_PROFDATA` | llvm-profdata-21 | Profile data merger |
| `LLVM_COV` | llvm-cov-21 | Coverage reporter |

### Excluded Files

| Variable | Default | Description |
|----------|---------|-------------|
| `C_EXCLUDE_FILES` | (list of doc-only files) | Regex of .c files to skip during compilation |

## Branch Coverage Accounting

We use two metrics (always report both):

- **OUR metric**: from `branch_coverage.py` using `llvm-cov export` branch entries.
  Each branch = 2 conditions (true/false). Covered = count > 0.
  Ground truth extracted from actual export data.

- **REPORT metric**: from `llvm-cov report` "Branches" column, filtered to library files.
  Uses LLVM's internal counting (different from export entries).

These give different absolute numbers but the same relative ordering.
OUR metric is stricter. Use `branch_coverage.py` for all programmatic decisions.

### Usage

```bash
# Extract ground truth (static, no execution needed):
llvm-cov-21 export ./bin -empty-profile > static.json
python3 scripts/branch_coverage.py extract static.json branches.json

# Measure coverage after running tests:
llvm-cov-21 export ./bin -instr-profile=test.profdata > coverage.json
python3 scripts/branch_coverage.py measure coverage.json

# Get uncovered conditions (for testgen feedback):
python3 scripts/branch_coverage.py uncovered coverage.json uncovered.md
```

## Difffix Loop Behavior

### Default (REACT_MODE=0)

Each round:
1. Generate compact divergence context from latest diff report
2. Analyzer reads report, generates up to MAX_GOALS goal files
3. Save pre-fix snapshot, fixer edits Rust code, save post-fix diff
4. Re-test: run differential test, count pass/fail
5. Stall check:
   - **Progress** (failures decreased): reset stall counter
   - **No progress** (same failures): increment stall counter
   - **Regression** (failures increased): rollback to previous snapshot,
     save failed attempt's diff + failures as feedback for next round

After rollback, the next round's analyzer sees:
- The last good round's diff report (current failures)
- The failed attempt's code diff and resulting failures
- Instruction: "do NOT repeat this approach"


## Test Bridge System

Static C functions can't be called from outside their .c file. Two separate
bridges provide test access:

- **C bridge** (`test_bridge.c`): `#include`s C source, wraps static functions
  as `bridge___xxx()`.
- **Rust bridge** (`test_bridge.rs`): exports same `bridge___xxx()` symbols,
  calls internal Rust implementations.

Both binaries link their own bridge. Same interface, different implementations.
No shared bridge objects between C and Rust binaries.

The Rust bridge also handles:
- Naming mismatches 
- Constants
- ABI translation (Rust slices ↔ C pointers)

## File-Based Snapshots (No Git)

The difffix loop saves per-round snapshots as plain files:
- `rounds/N/src_pre/` → src/ before fixes (deleted after diff recorded)
- `rounds/N/code_changes.diff` → diff between pre and post
- `rounds/N/src_snapshot/` → src/ after fixes (for rollback)

**Never use `git init` inside Rust working directories.** Git inside a
subdirectory causes submodule hell when committing from the outer repo.

## Critical Lessons (see lessons.md for full list)

1. **No C fallback**: Rust test binary must link WITHOUT the C library.
   Missing functions = link errors, not silent C fallback.

2. **O0 compilation**: Rust at opt-level > 0 can cause LLVM to optimize
   `pow(2.0, x)` → `exp2(x)` → infinite recursion. Always use opt-level = 0.

3. **Test isolation**: Use fork wrapper so one crash doesn't kill all tests.
   Applied to difftest only (not testgen coverage runs).

4. **No phantom functions**: Extract functions from compiled .o files, not
   source parsing. Source parsing misses `#ifdef` guards.

5. **Consistent branch accounting**: Use `llvm-cov export` branch entries
   as ground truth, not `llvm-cov report` or `llvm-cov show`.

6. **Never delete without backup**: Always `cp -r` before destructive ops.
   Verify copies exist. Never assume git has it.

7. **No git inside working dirs**: Use file-based snapshots only.

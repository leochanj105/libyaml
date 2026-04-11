# Lessons Learned

## Catastrophic Mistakes

### 1. C fallback in Rust test binary
The Rust test binary was linked with `libc_impl.a` (the C library) as fallback
using `--allow-multiple-definition`. Any function missing from Rust silently used
the C version. Tests showed 0 failures when there were actually many. Wasted
tokens running S1/S2 difffix on fake results.

**Fix**: Link Rust binary with only the Rust library. No `libc_impl.a`, no
`--allow-multiple-definition`. Missing functions become link errors.

### 2. git init inside rust-sN caused submodule hell
The difffix loop ran `git init` inside `rust-sN/` for per-round snapshots.
When the user committed from the outer repo, git silently treated `rust-sN/`
as a submodule reference (stored a commit hash, not the actual files). The
user thought they committed the code but it was a dead pointer.

**Fix**: Never `git init` inside working directories. Use file-based snapshots
instead: `cp -r src/ rounds/N/src_snapshot/` and `diff -ru` for diffs.

### 3. Destroyed data by rm -rf without proper copies
Deleted `.claude/` directories, `test_bridge.c/h` files, and `rust-sN/` directories
without verifying backups existed. Had to reconstruct `rust-s3-noreact` by replaying
edit operations from fix output JSON logs.

**Lesson**: Never delete anything without explicit user instruction. Always verify
backup exists BEFORE modifying or deleting. Use `git restore` to recover from git
when possible.

### 4. LLVM O3 optimization caused false infinite loops
Rust `exp2d` was implemented as `(2.0f64).powf(x)`. At O3, LLVM optimized
`pow(2.0, x)` back to `exp2(x)`, creating infinite recursion. Binary hung
for 10 minutes (the timeout), producing no useful error message.

**Fix**: Set `opt-level = 0` in Cargo.toml. Prevents LLVM from making
"equivalent" transformations that break the code. Also matches C compilation
(no optimization flags).

### 5. Function list included non-compiled long double functions
`extract_functions.sh` parsed C source with awk, which doesn't understand
`#ifdef` guards. 79 long double functions (behind `#ifdef __LIBMCS_LONG_DOUBLE_IS_64BITS`)
were included in the function list despite never being compiled. S4/S5 testgen
wasted effort generating tests for them. Coverage stats were misleading.

**Fix**: Extract functions from compiled `.o` files (nm), not source parsing.

## Infrastructure Fixes

### Bridge system for static functions
C has 8 static functions (`__tan`, `__sin_pi`, `__ctans`, etc.) that can't be
called from outside their `.c` file. Two separate bridges:
- **C bridge** (`test_bridge.c`): `#include`s the C source, wraps static
  functions as `bridge___tan()` etc.
- **Rust bridge** (`test_bridge.rs`): exports the same `bridge___tan()` symbols
  but calls internal Rust implementations.

The test suite calls `bridge___tan()`. C binary uses C bridge, Rust binary uses
Rust bridge. Same interface, different implementations. No shared bridge objects.

The Rust bridge also fixes naming mismatches (`__fpclassifyd` vs `__fpclassify`),
provides constants (`__infd`, `__inff`), and wraps internal functions with
different signatures (`__cos(x,y)` → `cos_kern(x,y)`, pointer→slice conversions).

### Prepare step for Rust test infrastructure
`scripts/prepare_rust_for_test.sh` takes `rust-baseline` and produces
`rust-baseline-test` by:
1. Copying the baseline
2. `sed` to change 13 private functions to `pub(crate)` (visibility only)
3. Copying `test_bridge.rs` into `src/`
4. Adding `mod test_bridge;` to `lib.rs`
5. Verifying `cargo build` succeeds

This runs once after transpilation. All scenario copies inherit the bridge.

### Timeout detection
Changed `run_difftest.sh` to check exit codes:
- Exit 124 = timeout (hung binary). Reports last output line to identify
  which function caused the hang.
- Exit >128 = signal (crash). Reports signal number and last output.
- Stderr panic = existing Rust panic handler.

Timeout reduced from 600s to 30s (C binary finishes in 1ms).

### Difffix loop file-based snapshots
Each round saves:
- `src_pre/` — copy of src/ before fixes (deleted after diff is recorded)
- `code_changes.diff` — diff between pre and post fix
- `src_snapshot/` — copy of src/ after fixes

Used for rollback (restore previous snapshot) and for feeding fix history
to the analyzer. No git inside working directories.

## Difffix Strategy: Rollback + Failed Attempt Feedback (Default)

When a round causes regression (failures increase):
1. **Rollback**: restore previous round's `src_snapshot/`
2. **Next round gets**: last good round's diff report AS the current report,
   PLUS the failed round's `code_changes.diff` and failure details with
   instruction "do NOT repeat this approach"

This worked well on S3: round 4 regressed (7→8), round 5 with feedback
got 7→2, fixing regressions without repeating the bad approach.

Alternative modes (via REACT_MODE env var):
- REACT_MODE=0 (default): rollback + failed attempt feedback
- REACT_MODE=1: additionally accumulate full history of all rounds

## Important Lessons from the User

### Never modify without backup
Always `cp -r` before any destructive operation. Verify the copy exists.
Never assume git has it — submodule issues can silently lose data.

### Don't overcomplicate
Start with the simplest possible fix. One-line changes over architectural
rewrites. The C fallback fix was literally removing `libc_impl.a` from one
line. The bridge was adding one Rust file with wrappers.

### Test infrastructure must be honest
If the test framework hides bugs (C fallback, killed tests, wrong function
list), all downstream results are worthless. Fix infrastructure first,
even if it means rerunning everything.

### Each test must be independent
One crash/hang should not kill all subsequent tests. Each test function
should run in its own process so results are isolated. (Not yet implemented.)

### Understand before running
Don't blindly launch experiments. Verify:
- What the test suite actually calls
- What the Rust binary actually links
- Whether the comparison is genuine (not C vs C)
- Whether the function list matches compiled reality

### Keep results organized
- Results in `work-sN/difffix/` with `summary.md` and `round_stats.tsv`
- Rust code copies with meaningful names (`rust-s3-noreact`, `rust-s3-plain`)
- No `.git` inside working directories (prevents submodule hell)
- Global summary in `experiment_results.md`

### Record everything for replay
Fix outputs contain full Claude session logs (stream-json). Edit operations
can be extracted and replayed to reconstruct code state. This saved us when
the rust-s3 git repo was destroyed.

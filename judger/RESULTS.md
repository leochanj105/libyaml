# Judger Results

> **Note: this is a sample run** against `exps/rust-baseline/` — the unmodified
> initial transpilation. It is shown to verify the judger pipeline works end-to-end
> against a real Rust port. It is **not** a final evaluation of any specific
> testgen / fix scenario. The numbers below will change as the Rust port is fixed.

## Setup

- **Library A**: `build/libyaml.a` (CMake build of `src/*.c`)
- **Library B**: `exps/rust-baseline/target/release/liblibyaml.a` (`cargo build --release`)
- **Date**: 2026-04-10
- **Total test cases**: 4,424 = 402 inputs × 11 test functions + 2 self-contained
- **Per-case timeout**: 10 s
- Rust .a linked against the C test functions in `judger/test-functions/` with no
  source modifications. Compile and link succeeded for all 13 test functions.

## Aggregate

| Verdict | Count | % |
|---|---:|---:|
| match | 2,095 | 47.4% |
| diff  | 1,757 | 39.7% |
| panic |   572 | 12.9% |

- **match** = exit code, stdout, stderr all bitwise identical
- **diff**  = outputs differ but neither side crashed or timed out
- **panic** = at least one side crashed (signal) or hung (timeout)

## Per–test-function breakdown

| Test function              | API layer                 | match | diff | panic |
|----------------------------|---------------------------|------:|-----:|------:|
| run-scanner                | scanner                   |   401 |    1 |     0 |
| run-parser                 | parser                    |   401 |    1 |     0 |
| run-loader                 | loader                    |   401 |    1 |     0 |
| run-parser-test-suite      | parser (event format)     |   351 |   51 |     0 |
| run-emitter                | parser → emitter          |   105 |    1 |   296 |
| run-dumper                 | loader → dumper           |   125 |    1 |   276 |
| run-emitter-test-suite     | emitter (event input)     |    80 |  322 |     0 |
| example-reformatter        | parser → emitter          |    73 |  329 |     0 |
| example-reformatter-alt    | loader → dumper           |    78 |  324 |     0 |
| example-deconstructor      | parser → emitter (canon.) |     0 |  402 |     0 |
| example-deconstructor-alt  | parser → doc → emitter    |    79 |  323 |     0 |
| test-version               | API smoke test            |     1 |    0 |     0 |
| test-reader                | reader (UTF-8 / BOM)      |     0 |    1 |     0 |
| **Total**                  |                           | **2,095** | **1,757** | **572** |

## Observations on this baseline

1. **Scanner / parser / loader counts mostly agree.** 401/402 cases match for
   `run-scanner`, `run-parser`, `run-loader` — these print only token / event /
   document counts. The Rust port is essentially correct at the count level.

2. **Parser event content has 51 mismatches.** `run-parser-test-suite` prints the
   full event stream (`+STR`, `=VAL :foo`, etc.) so it catches content differences
   the count-only functions miss. Example: input `236B` gives `"Line: 4"` in C and
   `"Line: 6"` in Rust for the same parse error — a line-counting bug in Rust.

3. **Emitter is the weakest area.** `run-emitter` (296) and `run-dumper` (276) both
   show large numbers of panics — assertion failures inside the Rust emitter when
   re-emitting parsed content. Specific assertion:

   ```
   run-emitter: ... main: Assertion `yaml_emitter_emit(&emitter, &event)
                || print_output(...)' failed.
   ```

4. **`example-deconstructor` is 0/402.** Every case produces a different verbose
   YAML output between C and Rust. Whether this is a real bug or a formatting
   difference (e.g., indent / quoting style) needs case-by-case inspection.

5. **`test-version` passes**, `test-reader` differs in one of its 210 internal
   sub-cases — encoding or BOM handling.

## How to reproduce

```bash
cd /home/leochanj/Desktop/libyaml

# 1. Build C version
( cd build && make )

# 2. Build Rust version
( cd exps/rust-baseline && cargo build --release )

# 3. Run end-to-end judger
judger/judge.py \
    --lib-a=build/libyaml.a \
    --lib-b=exps/rust-baseline/target/release/liblibyaml.a \
    --outdir=/tmp/judge-c-vs-rust \
    --timeout=10
```

Outputs:
- `/tmp/judge-c-vs-rust/A/results.jsonl` — C results (one JSON record per case)
- `/tmp/judge-c-vs-rust/B/results.jsonl` — Rust results
- `/tmp/judge-c-vs-rust/report.tsv` — per-case verdicts (`match` / `diff` / `panic`)

# Judger

Differential testing pipeline for libyaml: run the same test corpus against
two builds (e.g. C vs Rust) and diff the outputs case by case.

## Quick start

```bash
cd /home/leochanj/Desktop/libyaml

# Build both libraries
( cd build && make )
( cd exps/rust-baseline && cargo build --release )

# Run differential judger
judger/judge.py \
  --lib-a=build/libyaml.a \
  --lib-b=exps/rust-baseline/target/release/liblibyaml.a \
  --outdir=/tmp/judge-c-vs-rust \
  --timeout=10
```

Outputs:
- `/tmp/judge-c-vs-rust/A/results.jsonl` — library A per-case results
- `/tmp/judge-c-vs-rust/B/results.jsonl` — library B per-case results
- `/tmp/judge-c-vs-rust/report.tsv` — `verdict\tfunction\tinput` (match/diff/panic)

## Other entry points

Run against a single library (no comparison):

```bash
judger/run.py --lib=build/libyaml.a --outdir=/tmp/judge-c --timeout=10
```

Compare two existing result directories:

```bash
judger/compare.py \
  --a=/tmp/judge-c-vs-rust/A \
  --b=/tmp/judge-c-vs-rust/B \
  --report=/tmp/report.tsv
```

## Verdicts

- **match** — exit code, stdout, stderr all bitwise identical
- **diff**  — outputs differ, but neither side crashed or timed out
- **panic** — at least one side crashed (signal) or hung (timeout)

See `AUDIT.md` for how the test corpus was assembled and `RESULTS.md` for a
sample C-vs-Rust run.

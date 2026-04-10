# Judger Test Suite — Audit Trail

## Purpose

Test inputs and test functions for differential testing of libyaml.
Each test function is compiled against a linked library (C or Rust), run on each
applicable test input, and outputs are collected for later bitwise comparison.

**Test case** = one test function × one test input, run as an isolated subprocess.

---

## Directory Layout

```
judger/
├── yaml-test-suite/      352 test inputs  [SOURCE: yaml/yaml-test-suite repo, branch "data"]
├── test-functions/        13 C programs   [SOURCE: libyaml tests/]
├── blacklist/             (reference metadata, not test data)
├── run-tests.sh
└── AUDIT.md
```

---

## Test Inputs

### yaml-test-suite/ (352 inputs)

**Source**: `git clone --branch data https://github.com/yaml/yaml-test-suite`
(commit 6ad3d2c, fetched 2026-04-09).

Canonical cross-implementation YAML test suite. Each test ID is a 4-char directory.
Some IDs contain `in.yaml` directly; others have numbered sub-variants (`00/`, `01/`,
etc.) each with their own `in.yaml`. Metadata directories (`name/`, `tags/`) are excluded.

Files per test case:
- `in.yaml` — YAML input
- `===` — one-line description
- `error` — marker for inputs that should fail to parse
- `test.event` — event-format representation

**402 total test inputs** (333 direct + 69 from 17 multi-variant IDs).

---

## Test Functions

**Source**: All 13 from `libyaml/tests/`, copied verbatim.

### Functions that take YAML file as argument (6)

| Test function | libyaml APIs called |
|---------------|-------------------|
| run-scanner.c | `yaml_parser_scan()` |
| run-parser.c | `yaml_parser_parse()` |
| run-parser-test-suite.c | `yaml_parser_parse()` (test-suite output format) |
| run-loader.c | `yaml_parser_load()` |
| run-emitter.c | `yaml_parser_parse()` → `yaml_emitter_emit()` |
| run-dumper.c | `yaml_parser_load()` → `yaml_emitter_dump()` |

### Functions that read YAML from stdin (4)

| Test function | libyaml APIs called |
|---------------|-------------------|
| example-reformatter.c | `yaml_parser_parse()` → `yaml_emitter_emit()` |
| example-reformatter-alt.c | `yaml_parser_load()` → `yaml_emitter_dump()` |
| example-deconstructor.c | `yaml_parser_parse()` → `yaml_emitter_emit()` (canonical) |
| example-deconstructor-alt.c | `yaml_parser_parse()` → document build → `yaml_emitter_emit()` |

### Function that takes event file as argument (1)

| Test function | libyaml APIs called |
|---------------|-------------------|
| run-emitter-test-suite.c | `yaml_emitter_emit()` (reads test-suite event format) |

### Self-contained functions (2)

| Test function | libyaml APIs called |
|---------------|-------------------|
| test-version.c | `yaml_get_version()` |
| test-reader.c | `yaml_parser_update_buffer()` — 210 internal cases |

---

## Test Case Count

| Test inputs | × Test functions | = Test cases |
|-------------|-----------------|-------------|
| 402 YAML files | × 6 arg functions | 2,412 |
| 402 YAML files | × 4 stdin functions | 1,608 |
| 402 event files | × 1 event function | 402 |
| (self-contained) test-version | 1 | 1 |
| (self-contained) test-reader | 1 | 1 |
| **Total** | | **4,424** |

---

## Isolation and Output

Each test case runs as a **separate subprocess** with a configurable timeout (default 10s).
A crash (SIGSEGV, SIGABRT), hang, or failure in one test case cannot affect others.

Output format per test case:
```
=== CASE func=<name> input=<id> ===
EXIT: <code>
STDOUT:
<captured stdout>
STDERR:
<captured stderr>
=== END ===
```

All absolute paths are normalized (`<BIN>`, `<INPUT>`) so outputs are reproducible
across builds and machines.

Usage:
```bash
LIBYAML_LIB=path/to/libyaml.a ./run-tests.sh --outdir=/tmp/results
diff /tmp/results-c/suite1.out /tmp/results-rust/suite1.out
```

---

## What Was Excluded and Why

| Source | Reason |
|--------|--------|
| `examples/` (9 YAML files in repo) | 100% redundant with yaml-test-suite |
| `run-test-suite` branch | Orchestration only, no test data |

## Known Gaps

- **No large YAML inputs** — all yaml-test-suite files < 1KB
- **Minimal fuzz corpus** — OSS-Fuzz likely has thousands more inputs

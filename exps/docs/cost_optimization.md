# Cost Optimization Playbook

Strategies for reducing AI subprocess token costs in the progress harness.

## 1. CLAUDE.md Preload

**What:** `prepare.sh` generates `.claude/CLAUDE.md` containing all static C source
files. Claude loads this automatically at turn 0.

**Why:** After the first turn, CLAUDE.md enters the prompt cache at ~10% of normal
input token cost. Every subsequent AI call in the session benefits.

**Rule:** CLAUDE.md must be **immutable** for the session. Any modification
invalidates the prompt cache for all subsequent turns. Changing content goes in
`rust_changes.md` instead.

**Where:** `prepare.sh` (generation), `.claude/CLAUDE.md` (output)

## 2. Diff-Append Pattern

**What:** `scripts/update_rust_context.sh` writes the current Rust diff to
`$WORK_DIR/rust_changes.md`. This is called at the start of each difffix round.
Fixer and analysis prompts reference this file for incremental context.

**Why:** Keeps CLAUDE.md immutable (preserving cache) while still giving the AI
up-to-date context about what has changed. Avoids re-embedding the full Rust
source in every prompt.

**Where:** `scripts/update_rust_context.sh`, `difffix/run_difffix_loop.sh` (caller),
`scripts/fixer.sh` and `scripts/analysis.sh` (consumers)

## 3. Three-Layer Enforcement

Prevents AI from reading held-out test data (judger tests, testgen outputs)
through three independent mechanisms:

| Layer | Mechanism | Blocks | Where |
|-------|-----------|--------|-------|
| 1. Prompt | Instructions telling AI not to read certain dirs | Claude (advisory) | Prompt templates |
| 2. settings.json | `deny` rules in `.claude/settings.json` | Claude (tool-level) | `prepare.sh` generates dynamically |
| 3. chmod 000 | OS-level permission lock | All (claude, codex, any binary) | `ai_runner.sh`, `run_difffix_loop.sh` |

**Why:** Any single layer can fail (prompt ignored, settings not copied, git
checkout restores permissions). Three layers make bypass extremely unlikely.

**Where:** `prepare.sh` (generates settings.json), `scripts/ai_runner.sh`
(chmod judger), `difffix/run_difffix_loop.sh` (chmod testgen)

## 4. Prompt Cache Economics

- **Within-session:** Guaranteed after turn 1. Cached tokens cost ~10% of input.
- **Cross-session:** Possible if the prompt prefix is unchanged within a 5-minute
  TTL window. Batch runs that start sessions close together can benefit.
- **Ordering matters:** Static content (CLAUDE.md, instructions) should be the
  prompt prefix. Variable content (errors, diffs, goals) should come at the end.
  The harness prompts are structured this way — `fixer.md` template content comes
  before goal-specific content in the prompt string.

## 5. Model Selection

| Task | Default | Rationale |
|------|---------|-----------|
| Code generation (transpile, fix, testgen) | `codex` | Fastest for large edits |
| Analysis (strategy, failure analysis) | `claude` | Better reasoning |

Override per-project in `config_overrides.sh`:
```bash
CODE_GEN_CMD=claude   # use claude for everything
ANALYSIS_CMD=claude   # default
```

Override per-run via env:
```bash
CODE_GEN_CMD=claude ./difffix/run_difffix_loop.sh -v
```

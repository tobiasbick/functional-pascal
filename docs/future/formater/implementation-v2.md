# Formatter implementation plan — v2

Phased checklist for **`fpas fmt` v2**, building on the completed v1 plan ([implementation.md](implementation.md)).

**Prerequisite:** v1 shipped — emitter, CLI, golden/round-trip tests, [style.md](style.md) locked for v1 output.

**Status (2026-06-10):** **v2 complete** on `main`.

| Phase | Status | Commit (local `main`) |
|-------|--------|------------------------|
| 0 — Scope lock-in | done | docs in `5152c5b`, `0b8bea3` |
| 1 — CLI ergonomics | done | `0b8bea3` — multi-path, `--stdout`, `--check --list`, globs |
| 2 — Line wrapping | done | `26f08af` — `wrap.rs`, 100-col breaks, golden tests |
| 3 — Comments | done | `ae47e7b` — `CommentMap`, `format_source`, lexer comments |
| 4 — Repo / CI | done | `667029e` — mass format, scripts, GitHub Actions |
| 5 — Hardening | done | (this commit) — full tree round-trip, fuzz-light sample |

**How to use this doc:** Work one phase at a time. Check boxes when done. Stop after any phase; the next session picks up at the first unchecked item. Do not start a phase until the previous phase’s exit criteria pass.

**Normative style:** [style.md](style.md) is the **only** official output spec. v2 additions (wrapping width, comment placement) are written into `style.md` as fixed rules — not options. There is **no** `.fpasfmt.toml`, no `FormatOptions`, and no per-project style overrides.

---

## Official style policy (v2+)

- **One canonical style** for the whole ecosystem. `fpas fmt` always emits what [style.md](style.md) describes.
- **Constants live in code** (`crates/fpas-fmt/src/style.rs`), mirroring the doc. Example: `MAX_LINE_WIDTH = 100` once Phase 0 signs it off.
- **Changing the style** means editing `style.md`, updating golden tests, and releasing a new formatter version — not adding config knobs.
- **Out of scope forever (not v3):** `.fpasfmt.toml`, indent size, keyword case, optional `begin`/`end` form, sort order of `uses`/declarations.

---

## v2 goals (summary)

| Theme | Intent |
|-------|--------|
| **CLI ergonomics** | Multiple paths, stdout mode, clearer `--check` output |
| **Layout** | Line wrapping per fixed rules in style.md (v1 mostly single-line) |
| **Comments** | Preserve doc comments (`///`, `{ }` on declarations) without full trivia-preserving rewrite |
| **Repo hygiene** | One-shot format of the tree, CI recipe, contributor docs |

**Not v2:** LSP format-on-save, watch mode, invalid-syntax recovery, any configurable style, precompiled formatter plugins.

---

## Phase 0 — Scope and design lock-in

Exit criteria: written decisions in this file + [style.md](style.md); no code until sign-off.

- [x] Review v1 deferred items: [cli.md — Deferred](cli.md#deferred-post-v2-phase-1), [style.md — Non-goals (v1)](style.md#non-goals-v1-and-later).
- [x] Confirm v2 **in scope** list (table above) or trim before coding.
- [x] Confirm **no config file** policy (see [Official style policy](#official-style-policy-v2)) — document in style.md under Non-goals.
- [x] **Line width:** fixed max line length (**100** columns). Single constant in `style.rs`; normative rule in style.md.
- [x] **Wrapping rules:** which constructs break across lines when over width:
  - [x] `uses` clause (style.md already says wrap with 2-space indent)
  - [x] `function` / `procedure` formal lists
  - [x] Multi-field `record` literals (style.md: single line when fits)
  - [x] Long binary chains / calls (parenthesis-aware)
- [x] **Comments strategy** — pick one (blocks Phase 3):
  - [x] **Option A (recommended):** Lexer comment map keyed by span; emitter re-attaches `///` and `{ }` before the declaration they preceded in source (lossy for intra-block comments).
  - [x] **Option B:** Full trivia-preserving formatter — **defer to v3** (separate crate or major rewrite).
- [x] Add **v2 golden examples** to style.md (long `uses`, wrapped record literal; doc-comment file deferred to Phase 3).
- [x] Sign off Phase 0 in chat / PR before Phase 1.

---

## Phase 1 — CLI ergonomics

Target: [`crates/fpas-cli/src/cli_fmt/`](../../../crates/fpas-cli/src/cli_fmt/), [`cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs).

- [x] **Multiple positional paths:** `fpas fmt a.fpas b.fpas dir/` — format each resolved `.fpas` (directory → all `.fpas` recursively, skip `target/`).
- [x] **`--stdout`:** print formatted text to stdout; do not write file (mutually exclusive with `--check`; error if both).
- [x] **`--list` (optional):** with `--check`, print paths that would change (one per line) — helps CI scripts.
- [x] **Built-in glob:** if a path argument contains unexpanded `*` / `?`, expand via `glob` crate (shell-independent).
- [x] Update `CLI_HELP` and [cli.md](cli.md) with v2 usage and exit codes (unchanged: `0` / `1` / `2`).
- [x] Tests in `crates/fpas-cli/src/main_tests/fmt.rs`:
  - [x] two explicit `.fpas` paths
  - [x] `--stdout` does not modify file on disk
  - [x] `--check --list` prints dirty paths only
  - [x] glob path
- [x] `cargo test -p fpas-cli fmt`

**Phase 1 exit:** `fpas fmt a.fpas b.fpas` and `fpas fmt --stdout file.fpas` work; docs updated.

---

## Phase 2 — Line wrapping (emitter)

Target: [`crates/fpas-fmt/src/emit/wrap.rs`](../../../crates/fpas-fmt/src/emit/wrap.rs), [`emit/mod.rs`](../../../crates/fpas-fmt/src/emit/mod.rs), [`program.rs`](../../../crates/fpas-fmt/src/emit/program.rs), [`types.rs`](../../../crates/fpas-fmt/src/emit/types.rs), [`expr.rs`](../../../crates/fpas-fmt/src/emit/expr.rs).

- [x] `style.rs`: `MAX_LINE_WIDTH` constant (value from Phase 0); no runtime overrides.
- [x] Measure rendered line length including leading indent ([style.md — Line width](style.md#line-width-v2)).
- [x] **`uses` wrapping:** break after commas; continuation lines indented per [style.md — Indentation](style.md#indentation).
- [x] **Formal parameter lists:** break after `;` in long `function` / `procedure` headers.
- [x] **Record / array literals:** multi-line when over width; keep v1 semicolon rules.
- [x] **Expressions:** break long binary chains at lowest-precedence operator; do not break inside string literals.
- [x] **Stability:** same AST → same breaks (no random wrapping); add width to golden tests.
- [x] Golden files under `crates/fpas-fmt/tests/golden/` for wrapped `uses` and wrapped record literal.
- [x] Round-trip tests still pass (`cargo test -p fpas-fmt`).

**Phase 2 exit:** files over max width wrap predictably; golden tests cover wrapping.

---

## Phase 3 — Doc and declaration comments

Depends on Phase 0 comment strategy. Skip entire phase if Option B deferred.

- [x] Parser/lexer: expose comment tokens with spans (`fpas-lexer`: `SourceComment`, `collect_comments`, `CommentStyle`).
- [x] Build `CommentMap` keyed by declaration anchor offset (`crates/fpas-fmt/src/comments/map.rs`).
- [x] Attach leading `///` / `{ }` / `(* *)` to nearest following declaration (whitespace or visibility/keyword preamble before parser anchor).
- [x] Emitter: print preserved comments before the declaration they belong to; **still strip** end-of-line and intra-statement comments in v2 unless explicitly scoped.
- [x] Blank-line rules: one blank line after a doc block before the declaration (document in style.md).
- [x] Tests: `CommentMap` unit test (doc + block on private decl); `shell.fpas` private-state round-trip; CLI and round-trip corpus use `format_source`.
- [x] Update [style.md — Comments](style.md#comments) and [Intentional diffs](style.md#intentional-diffs-from-source) for v2.

**Phase 3 exit:** doc comments on units/routines/types survive `fpas fmt`; block comments on declarations survive; statement comments still removed (documented).

---

## Phase 4 — Repository and CI integration

Mostly docs and one-time repo policy — minimal code.

- [x] Script or documented command: [`scripts/format-fpas-sources.sh`](../../../scripts/format-fpas-sources.sh) / [`.ps1`](../../../scripts/format-fpas-sources.ps1) — `fpas fmt examples tests apps` (skips `target/`).
- [x] Run once on `examples/`, `tests/`, `apps/` after Phases 1–3 stable.
- [x] CI step: [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml) runs `fpas fmt --check examples tests apps`; noted in [AGENTS.md](../../../AGENTS.md).
- [x] Pre-commit hook **example**: [`scripts/pre-commit-fmt.example`](../../../scripts/pre-commit-fmt.example) (optional install).
- [x] `cargo test --workspace` green after mass-format.
- [x] Note in [README.md](../../../README.md): repo uses the official style from [style.md](style.md).

**Phase 4 exit:** tree formatted; CI check documented; workspace tests green.

---

## Phase 5 — Hardening and coverage

- [x] Expand round-trip corpus: all `tests/**/*.fpas`, `apps/**/*.fpas`, and full `examples/` tree ([`round_trip.rs`](../../../crates/fpas-fmt/tests/round_trip.rs)).
- [x] Fuzz-light: deterministic sample (stride 11) from `examples/`, `tests/`, `apps/` → format → re-parse → idempotent ([`fuzz_light.rs`](../../../crates/fpas-fmt/tests/fuzz_light.rs)).
- [x] Fix any emitter bugs found (private `type` keyword, case `else` idempotency — fixed in Phase 4).
- [x] `cargo fmt`, `cargo build --workspace`, `cargo test --workspace`.
- [x] Mark v2 complete in [README.md](README.md); archive open questions in [cli.md](cli.md).

**Phase 5 exit:** v2 signed off; this checklist fully checked or explicitly deferred items moved to v3 section below.

---

## Explicitly deferred to v3+

| Item | Reason |
|------|--------|
| Full trivia-preserving formatter (all comment positions, whitespace) | Large rewrite; Option B |
| LSP / editor format-on-save | Separate integration |
| `fpas fmt --watch` | Low priority |
| Sort `uses` / declarations | Opinionated; breaks blame |
| Format invalid / partial syntax | Recovery parser not planned |
| **Any configurable style** (`.fpasfmt.toml`, width, indent, keyword case) | **Rejected:** one official style only ([style.md](style.md)) |

---

## Dependency graph (unchanged from v1)

```text
fpas-cli ──► fpas-fmt ──► fpas-parser ──► fpas-lexer
                │
                └── Phase 3 may read fpas-lexer comment API directly
```

---

## Suggested session order

1. ~~**Phase 0**~~ — scope and fixed style rules (done).
2. ~~**Phase 1**~~ — CLI ergonomics (done).
3. ~~**Phase 2**~~ — line wrapping (done).
4. ~~**Phase 3**~~ — comments; lexer + `CommentMap` + `format_source` (done).
5. ~~**Phase 4**~~ — mass-format repo + CI docs (done).
6. ~~**Phase 5**~~ — hardening pass (done).

**v2 is complete.** Active work: [implementation-v3.md](implementation-v3.md) (Phase 0 done, Phase 1 next). Permanently rejected items remain in [Explicitly deferred to v3+](#explicitly-deferred-to-v3) in v2 and [Explicitly deferred to v4+](implementation-v3.md#explicitly-deferred-to-v4) in v3.

---

## v1 reference (do not redo)

Completed in [implementation.md](implementation.md): scaffold, emitters, compilation units, golden/round-trip tests, `fpas fmt` + `--check`, private unit decl fix (`ae55c1a`).

**v2 so far (do not redo):** Phase 0 style lock-in; Phase 1 CLI (`cli_fmt/`, globs); Phase 2 wrapping (`emit/wrap.rs`, column tracking on `Emitter`, golden `long_uses` / `wrapped_record`); Phase 3 comments (`fpas-lexer` comment API, `CommentMap`, `format_source`, preamble-aware attachment); Phase 4 repo format (`scripts/format-fpas-sources.*`, `.github/workflows/ci.yml`, emitter fixes for private `type` and case `else` idempotency); Phase 5 hardening (`round_trip` full tree, `fuzz_light` sample).

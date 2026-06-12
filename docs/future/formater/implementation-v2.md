# Formatter implementation plan — v2

Phased checklist for **`fpas fmt` v2**, building on the completed v1 plan ([implementation.md](implementation.md)).

**Prerequisite:** v1 shipped — emitter, CLI, golden/round-trip tests, [style.md](style.md) locked for v1 output.

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

Target: [`crates/fpas-fmt/src/emit/`](../../../crates/fpas-fmt/src/emit/) — likely new `wrap.rs` or width-aware helpers on `Emitter`.

- [x] `style.rs`: `MAX_LINE_WIDTH` constant (value from Phase 0); no runtime overrides.
- [ ] Measure rendered line length excluding leading indent (or document inclusive rule in style.md).
- [ ] **`uses` wrapping:** break after commas; continuation lines indented per [style.md — Indentation](style.md#indentation).
- [ ] **Formal parameter lists:** break after `;` in long `function` / `procedure` headers.
- [ ] **Record / array literals:** multi-line when over width; keep v1 semicolon rules.
- [ ] **Expressions:** break long binary chains at lowest-precedence operator; do not break inside string literals.
- [ ] **Stability:** same AST → same breaks (no random wrapping); add width to golden tests.
- [ ] Golden files under `crates/fpas-fmt/tests/golden/` for at least one wrapped `uses` and one wrapped record literal.
- [ ] Round-trip tests still pass (`cargo test -p fpas-fmt`).

**Phase 2 exit:** files over max width wrap predictably; golden tests cover wrapping.

---

## Phase 3 — Doc and declaration comments

Depends on Phase 0 comment strategy. Skip entire phase if Option B deferred.

- [ ] Parser/lexer: expose comment tokens with spans (audit `fpas-lexer` — may already exist for diagnostics).
- [ ] Build `CommentMap` (file path → sorted comments by offset).
- [ ] Attach leading `///` / `{ }` / `(* *)` to nearest following declaration in same file.
- [ ] Emitter: print preserved comments before the declaration they belong to; **still strip** end-of-line and intra-statement comments in v2 unless explicitly scoped.
- [ ] Blank-line rules: one blank line after a doc block before the declaration (document in style.md).
- [ ] Golden test: `apps/ide/src/shell.fpas`-style unit with `///` on routines — comments survive format.
- [ ] Update [style.md — Comments](style.md#comments) and [Intentional diffs](style.md#intentional-diffs-from-source) for v2.

**Phase 3 exit:** doc comments on units/routines/types survive `fpas fmt`; block comments on declarations survive; statement comments still removed (documented).

---

## Phase 4 — Repository and CI integration

Mostly docs and one-time repo policy — minimal code.

- [ ] Script or documented command: format all `.fpas` in repo (exclude `target/`).
- [ ] Run once on `examples/`, `tests/`, `apps/`, `crates/` (if any `.fpas`) after Phases 1–3 stable.
- [ ] Add CI step: `fpas fmt --check` (or `--check --list`) on PR — document in contributor guide / `AGENTS.md` if desired.
- [ ] Pre-commit hook **example** (optional script under `scripts/`, not mandatory install).
- [ ] Verify `cargo test --workspace` green after mass-format.
- [ ] Note in [README.md](README.md): repo uses the official style from style.md.

**Phase 4 exit:** tree formatted; CI check documented; workspace tests green.

---

## Phase 5 — Hardening and coverage

- [ ] Expand round-trip corpus: all `tests/**/*.fpas`, `apps/**/*.fpas` (not only `examples/pascal`).
- [ ] Fuzz-light: sample N files from full tree → format → re-parse (no panic).
- [ ] Fix any emitter bugs found (track in issues; patch in this phase).
- [ ] `cargo fmt`, `cargo build --workspace`, `cargo test --workspace`.
- [ ] Mark v2 complete in [README.md](README.md); archive open questions in [cli.md](cli.md).

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

1. **Phase 0** — agree scope and fixed style rules (1 discussion, no code).
2. **Phase 1** — CLI only; immediate daily-use win.
3. **Phase 2** — wrapping; may change many golden files.
4. **Phase 3** — comments; touch lexer + emitter.
5. **Phase 4** — mass-format repo + CI docs.
6. **Phase 5** — hardening pass.

Stop after any numbered phase; resume at the first unchecked `- [ ]` in the next phase.

---

## v1 reference (do not redo)

Completed in [implementation.md](implementation.md): scaffold, emitters, compilation units, golden/round-trip tests, `fpas fmt` + `--check`, private unit decl fix (`ae55c1a`).

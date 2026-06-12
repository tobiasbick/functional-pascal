# Formatter implementation plan — v3

Phased checklist for **`fpas fmt` v3**, building on the completed v2 plan ([implementation-v2.md](implementation-v2.md)).

**Prerequisite:** v2 shipped — CLI ergonomics, 100-column wrapping, declaration comment preservation (`CommentMap` / `format_source`), repo format + CI, full-tree round-trip and fuzz-light tests.

**Status (2026-06-10):** **Phase 0 complete.** Implementation **not started** — resume at [Phase 1](#phase-1--trivia-stream-lexer--crate-boundary).

| Phase | Status | Theme |
|-------|--------|--------|
| 0 — Scope lock-in | **done** | Trivia policy, hybrid model, mass-format policy |
| 1 — Trivia stream | **next** | Lexer whitespace + all comments in source order |
| 2 — Trivia attachment | pending | Map trivia to AST gaps (full Option B model) |
| 3 — Trivia-aware emit | pending | EOL / intra-statement comments + user blank lines |
| 4 — Trivia hardening | pending | Goldens, idempotency, one-shot tree format |
| 5 — Editor integration | pending | `--watch` + editor docs (LSP crate deferred) |
| 6 — Sign-off | pending | Docs, CI, workspace green |

**How to use this doc:** Work one phase at a time. Check boxes when done. Stop after any phase; the next session picks up at the first unchecked item. Do not start a phase until the previous phase’s exit criteria pass.

**If context was lost:** read [Context recovery (handoff)](#context-recovery-handoff) first — it restates decisions, v2 baseline, and file pointers without needing chat history.

---

## Context recovery (handoff)

### What v3 is

Extend the v2 AST pretty-printer so **comments and user blank lines survive** `fpas fmt`, while **layout stays canonical** ([style.md](style.md): indent, 100-col wrap, `begin`/`end`, keywords, literals). Then add **editor integration** (`--watch`, format-on-save docs).

### Stakeholder decisions (2026-06-10, confirmed)

| Question | Decision |
|----------|----------|
| Primary deliverable | **Phased:** full trivia (Option B) first, then editor integration |
| Trivia depth | **Full:** all comment positions + user blank lines between sections |
| Trivia vs layout conflict | **Trivia wins** for comments and user blank lines; **layout wins** for indent, wrap, blocks, keywords, literals |
| Architecture | **Extend `fpas-fmt`** (`trivia/` module); no second formatter crate unless hybrid fails |
| Attachment model | **Hybrid** (gaps + merge) — see [Hybrid attachment model](#hybrid-attachment-model-signed-off) |
| Mass-format after v3 | **One-shot** re-run `scripts/format-fpas-sources.*` after Phase 3 stable; **CI stays `--check` only** |
| Still rejected | `.fpasfmt.toml`, sort `uses`/declarations, optional `begin`/`end` omission, invalid-syntax recovery |

### v2 baseline (already shipped — do not redo)

| Area | What exists |
|------|-------------|
| CLI | Multi-path, `--stdout`, `--check`, `--list`, globs — [`cli_fmt/`](../../../crates/fpas-cli/src/cli_fmt/) |
| Wrapping | 100 cols — [`emit/wrap.rs`](../../../crates/fpas-fmt/src/emit/wrap.rs), `MAX_LINE_WIDTH` in [`style.rs`](../../../crates/fpas-fmt/src/style.rs) |
| Comments (partial) | `SourceComment`, `collect_comments` — [`fpas-lexer/src/comments.rs`](../../../crates/fpas-lexer/src/comments.rs); `CommentMap`, `format_source` — [`fpas-fmt/src/comments/`](../../../crates/fpas-fmt/src/comments/) |
| Repo | Mass-formatted `examples/`, `tests/`, `apps/`; CI — [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml); scripts — [`scripts/format-fpas-sources.sh`](../../../scripts/format-fpas-sources.sh) |
| Tests | Full-tree round-trip — [`round_trip.rs`](../../../crates/fpas-fmt/tests/round_trip.rs); fuzz-light sample — [`fuzz_light.rs`](../../../crates/fpas-fmt/tests/fuzz_light.rs) |
| Emitter fixes | Private `type` keyword; case-`else` idempotency — [`emit/decl.rs`](../../../crates/fpas-fmt/src/emit/decl.rs), [`emit/stmt.rs`](../../../crates/fpas-fmt/src/emit/stmt.rs) |

**Local commit chain (v2, on `main`):** `0b8bea3` (CLI) → `26f08af` (wrap) → `8355718` (docs) → `ae47e7b` (comments) → `667029e` (repo/CI) → `1e89052` (hardening).

### v3 entry point for coding

1. Phase 1: lexer `collect_trivia` + `fpas-fmt/src/trivia/stream.rs`
2. Do **not** remove v2 `CommentMap` until Phase 2 migrates tests to `TriviaMap`
3. `format_source(source, unit)` remains the CLI path; trivia-aware emit replaces internals in Phase 3

### Document map

| File | Role |
|------|------|
| [implementation-v3.md](implementation-v3.md) | **This file** — v3 phases and decisions |
| [implementation-v2.md](implementation-v2.md) | v2 complete; deferred items point here |
| [style.md](style.md) | Normative output; v3 rules added in Phase 3 ([planned section](style.md#comments-v3-planned)) |
| [cli.md](cli.md) | CLI; v3 adds `--watch` in Phase 5 |
| [README.md](README.md) | Formatter hub + status |

---

## v3 goals (signed off)

| Theme | Intent |
|-------|--------|
| **Full trivia preservation (Option B)** | All comment positions (`///`, `{ }`, `(* *)`, `//`, EOL) and user-placed blank lines between sections survive format when valid |
| **Layout still canonical** | Indent, wrapping, `begin`/`end`, keyword case, literals, `uses` layout — still [style.md](style.md); no configurable style |
| **Editor integration** | Format-on-save path (editor docs MVP) and optional `fpas fmt --watch` |
| **Same crate** | Extend [`fpas-fmt`](../../../crates/fpas-fmt/); no second formatter crate unless Phase 2 review proves hybrid unworkable |

**Not v3:** `.fpasfmt.toml`, sort `uses`/declarations, optional single-statement branches, invalid-syntax recovery, precompiled formatter plugins.

---

## Trivia vs layout policy (v3)

v2 used **Option A** (re-attach leading declaration comments only). v3 completes **Option B** with an explicit split:

| Concern | Rule |
|---------|------|
| **Comments** (all styles, any position) | **Preserve** from source when attached to a valid parse tree |
| **User blank lines** between sections | **Preserve**; extra blank lines the user added are kept |
| **Required blank lines** (after header, after `uses`, before record methods) | **Emit** if missing — style minimum still applies |
| **Indent, wrapping, blocks, keywords, literals** | **Normalize** per style.md — trivia does not override layout |
| **Conflicts** (e.g. user removed required blank line after `uses`) | Insert required layout; preserve additional user blank lines elsewhere |
| **Runs of 3+ blank lines** | Collapse to **2** max (document in `policy.rs`; matches common formatter behavior) |

Document normative examples in [style.md — Comments](style.md#comments) during Phase 3 exit (drafts: [v3 golden targets](#v3-golden-targets-for-stylemd-phase-3)).

**Still one official style:** no per-project overrides. “Preserve trivia” is not “preserve messy layout.”

---

## Hybrid attachment model (signed off)

**Rejected for v3:** token-only rewrite (full source reconstruction from token stream without AST gaps).

**Chosen:** hybrid gap merge.

```text
source text
    │
    ├─► parse ──► AST (+ spans)
    │
    └─► lex trivia ──► TriviaStream (ordered Whitespace | Comment segments)

AST + spans ──► gap graph (slot ids between layout regions)
TriviaStream ──► TriviaMap (segments assigned to gaps)

emit layout ──► LayoutBuffer with holes at gap slots
merge ──► fill holes from TriviaMap + apply policy.rs (required blanks, 2-line cap)
    │
    └─► final String
```

**Gap types (Phase 2):**

| Gap | Trivia allowed |
|-----|----------------|
| Before declaration / statement | Leading comments, blank lines |
| After statement (same line) | EOL `//`, `{ }`, `(* *)` |
| Between declaration groups | User blank lines |
| Inside `begin`/`end` block | Standalone comment lines, blank lines between statements |
| Inside layout tokens | **None** — indent/wrap/keywords emitted by AST emitter only |

**Merge invariants:**

- No overlapping spans after merge (test in Phase 3).
- `format_source` twice → identical bytes (extend fuzz-light).
- Unmapped non-EOF trivia → test failure in Phase 2.

**Fallback (v4 only):** if goldens fail repeatedly on hybrid, document failure cases and reconsider token-aligned rewrite — do not start v3 with token-only.

---

## Mass-format policy (signed off)

| When | Action |
|------|--------|
| **After Phase 3 stable** | One-shot: `scripts/format-fpas-sources.sh` (or `.ps1`) on `examples/`, `tests/`, `apps/` — same as v2 Phase 4 |
| **CI (ongoing)** | `cargo run -p fpas-cli -- fmt --check examples tests apps` — **no auto-write** in CI |
| **Contributors** | Run format script before PR; `AGENTS.md` already references `--check` |
| **Golden / expected files** | Update in same commit as one-shot format |

Rationale: one tree-wide pass applies new trivia rules consistently; CI only verifies drift afterward.

---

## Intended file layout (before Phase 1 code)

```text
crates/fpas-fmt/src/
 ├── comments/              — EXISTS (v2 Option A); fold into trivia/ Phase 2–3
 │   ├── map.rs
 │   └── emit.rs
 ├── trivia/                — NEW (v3)
 │   ├── mod.rs
 │   ├── stream.rs           — TriviaStream from lexer
 │   ├── attach.rs           — TriviaMap + gaps
 │   └── policy.rs           — trivia vs layout merge rules
 ├── emit/                   — MODIFY: LayoutBuffer + gap slots (Phase 3)
 └── lib.rs                  — format_source → trivia path

crates/fpas-lexer/src/
 ├── comments.rs             — EXISTS
 └── trivia.rs               — NEW (or extend comments.rs): whitespace runs

crates/fpas-cli/src/cli_fmt/ — Phase 5: --watch
docs/future/formater/style.md — Phase 3: normative v3 comment/blank-line rules
```

Split any file past ~400 LOC per [AGENTS.md](../../../AGENTS.md).

---

## v3 golden targets for style.md (Phase 3)

Draft normative examples to copy into [style.md](style.md) when Phase 3 completes. Until then they are **non-normative** targets.

### EOL and intra-block comments

**Input** (abbreviated):

```pascal
program Demo;
uses Std.Console;

begin
  WriteLn('hi'); { trailing block }
  // standalone line comment
  var X: integer := 1
end.
```

**Target after v3 `fpas fmt`:** trailing `{ trailing block }` stays on the `WriteLn` line; standalone `//` line stays inside `begin`; layout still uses `begin`/`end` and 2-space indent.

### Extra blank line between declarations

**Input:**

```pascal
unit U;

const Pi: real := 3.14;

type Point = record X: integer; Y: integer; end;
```

**Target:** blank line between `const` and `type` **preserved**; visibility/indent normalized.

### Doc + block (extends v2)

Reference file: [`apps/ide/src/shell.fpas`](../../../apps/ide/src/shell.fpas) — all `///` and `{ ... }` before declarations and routines must survive; v3 adds EOL/in-block comments elsewhere in the same file without loss.

---

## Phase 0 — Scope and design lock-in

Exit criteria: written decisions in this file + planned `style.md` edits; no code until sign-off.

- [x] **Primary deliverable:** phased trivia (Option B, full depth) then editor integration (`--watch` + editor docs).
- [x] **Trivia depth:** all comment positions + user blank lines between sections.
- [x] **Conflict policy:** trivia wins for comments and user blank lines; layout follows style.md.
- [x] **Architecture:** extend `fpas-fmt` in place; new `trivia/` module group.
- [x] **Still out of scope:** `.fpasfmt.toml`, sort order, optional `begin`/`end` omission, invalid-syntax recovery.
- [x] **Attachment model:** **hybrid** (gaps + merge) — [Hybrid attachment model](#hybrid-attachment-model-signed-off).
- [x] **Mass-format policy:** one-shot script after Phase 3; CI `--check` only — [Mass-format policy](#mass-format-policy-signed-off).
- [x] **Golden targets:** drafted in [v3 golden targets](#v3-golden-targets-for-stylemd-phase-3); copy to style.md in Phase 3.
- [x] **style.md stub:** [Comments (v3 planned)](style.md#comments-v3-planned) section added (non-normative until Phase 3).
- [x] Sign off Phase 0 (2026-06-10).

**Phase 0 exit:** met — start Phase 1.

---

## Phase 1 — Trivia stream (lexer + crate boundary)

Target: [`crates/fpas-lexer/src/`](../../../crates/fpas-lexer/src/), [`crates/fpas-fmt/src/trivia/stream.rs`](../../../crates/fpas-fmt/src/trivia/stream.rs).

- [ ] Lexer: expose **whitespace runs** (spaces, tabs, newlines) with spans alongside [`SourceComment`](../../../crates/fpas-lexer/src/comments.rs).
- [ ] `TriviaSegment` enum: `Whitespace`, `Comment(SourceComment)`.
- [ ] `collect_trivia(source) -> Vec<TriviaSegment>` in source order; respect string/char literal boundaries.
- [ ] Unit tests: mixed `//`, `{ }`, `(* *)`, `///`, blank lines; CRLF → LF policy documented (match v2 CLI normalize).
- [ ] `fpas-fmt`: `TriviaStream::build(source)`.
- [ ] `cargo test -p fpas-lexer`, `cargo test -p fpas-fmt`.

**Phase 1 exit:** ordered trivia stream for any valid `.fpas` source; no emitter changes yet.

---

## Phase 2 — Trivia attachment (Option B model)

Target: [`crates/fpas-fmt/src/trivia/attach.rs`](../../../crates/fpas-fmt/src/trivia/attach.rs); generalize [`comments/map.rs`](../../../crates/fpas-fmt/src/comments/map.rs) preamble gap logic.

- [ ] Define **gaps** from AST spans (declarations, statements, section boundaries).
- [ ] `TriviaMap::build(source, unit) -> TriviaMap`.
- [ ] Attachment rules:
  - [ ] Leading trivia before declaration (generalize v2 `gap_leads_to_anchor` / preamble keywords).
  - [ ] Trailing EOL comments on statement lines.
  - [ ] Standalone comment lines inside blocks.
  - [ ] User blank-line runs in gaps (apply 2-line cap in map or policy).
- [ ] Tests: `shell.fpas`; EOL `{ }`; `//` after `;`; nested `case`.
- [ ] v2 `CommentMap` tests pass or migrate to `TriviaMap`.

**Phase 2 exit:** every trivia segment maps to a gap; unmapped non-EOF trivia fails tests.

---

## Phase 3 — Trivia-aware emit

Target: [`emit/`](../../../crates/fpas-fmt/src/emit/), [`trivia/policy.rs`](../../../crates/fpas-fmt/src/trivia/policy.rs), [`lib.rs`](../../../crates/fpas-fmt/src/lib.rs).

- [ ] `LayoutBuffer` (or equivalent) with gap slots; existing emitters fill layout regions.
- [ ] Merge pass: trivia into slots + required style.md blank lines.
- [ ] `format_source` uses trivia path; `format_compilation_unit` without source strips trivia (v2 behavior).
- [ ] CLI unchanged; uses `format_source` automatically.
- [ ] Golden tests from [v3 golden targets](#v3-golden-targets-for-stylemd-phase-3).
- [ ] Update [style.md — Comments](style.md#comments), [Blank lines](style.md#blank-lines), [Intentional diffs](style.md#intentional-diffs-from-source) — replace “v3 planned” with normative v3 rules.
- [ ] Idempotency: extend [`fuzz_light.rs`](../../../crates/fpas-fmt/tests/fuzz_light.rs).

**Phase 3 exit:** full Option B; style.md normative for v3; goldens green.

---

## Phase 4 — Trivia hardening

- [ ] Round-trip full tree; spot-check comment-bearing files (no silent drops).
- [ ] **One-shot** `scripts/format-fpas-sources.*`; commit with Phase 4.
- [ ] Document in [README.md](../../../README.md) if contributor wording changes.
- [ ] Fix bugs; `cargo fmt`, `cargo build --workspace`, `cargo test --workspace`.

**Phase 4 exit:** tree formatted once under v3 rules; CI `--check` green.

---

## Phase 5 — Editor integration

Target: [`crates/fpas-cli/`](../../../crates/fpas-cli/), [cli.md](cli.md).

- [ ] `fpas fmt --watch <paths...>` — debounced, skip `target/`.
- [ ] **Editor MVP (signed off):** document format-on-save via CLI (Cursor/VS Code task or extension calling `fpas fmt` on save path); **no LSP crate in v3 unless MVP insufficient**.
- [ ] CLI integration test for `--watch` (temp dir).
- [ ] Update [cli.md](cli.md) — implemented vs [Deferred to v4+](cli.md#deferred-to-v4).

**Phase 5 exit:** `--watch` tested; editor doc published.

---

## Phase 6 — Sign-off

- [ ] Mark v3 complete in [formater/README.md](README.md), [docs/future/README.md](../README.md).
- [ ] Move leftovers to [cli.md — Deferred to v4+](cli.md#deferred-to-v4).
- [ ] Final `cargo test --workspace`.

**Phase 6 exit:** v3 checklist complete or items explicitly in v4 table.

---

## Explicitly deferred to v4+

| Item | Reason |
|------|--------|
| Dedicated LSP server crate | Editor CLI/docs MVP first |
| Token-only formatter | Hybrid signed off; fallback only if hybrid fails |
| Sort `uses` / declarations | **Rejected** |
| Format invalid / partial syntax | Recovery parser not planned |
| **Any configurable style** | **Rejected** |

---

## Dependency graph

```text
fpas-cli ──► fpas-fmt ──► fpas-parser ──► fpas-lexer
                │              │
                │              └── spans → gaps
                └── trivia/ (v3)

Phase 5 (--watch) ──► fpas-cli
Phase 5 (editor)  ──► external editor ──► fpas fmt CLI
```

---

## Suggested session order

1. ~~**Phase 0**~~ — done.
2. **Phase 1** — trivia stream. **← resume here**
3. **Phase 2** — `TriviaMap`.
4. **Phase 3** — trivia-aware emit + style.md.
5. **Phase 4** — hardening + one-shot tree format.
6. **Phase 5** — `--watch` + editor docs.
7. **Phase 6** — sign-off.

Stop after any numbered phase; resume at the first unchecked `- [ ]`.

---

## v1/v2 reference (do not redo)

- **v1:** [implementation.md](implementation.md) — scaffold, emitters, CLI, goldens.
- **v2:** [implementation-v2.md](implementation-v2.md) — CLI ergonomics, wrapping, Option A comments, repo/CI, hardening.

v3 **extends** v2; do not remove `format_source`, CI `--check`, or round-trip/fuzz-light tests.

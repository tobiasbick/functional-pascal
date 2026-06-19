---
name: fpas-change-checklist
description: >
  Ensures Functional Pascal changes update docs and tests before completion. Use when implementing
  or modifying language behavior, Std.* APIs, compiler/VM/runtime, CLI, diagnostics, or user-facing
  docs under docs/pascal/. Also use when the user asks to add a feature, fix a bug, extend the stdlib,
  or mentions docs, spec, tests, or regression coverage. Read this skill at the start of implementation
  work and re-check before finishing.
---

# FPAS change checklist

Apply on every task that touches behavior, public API, or user-visible diagnostics — not only when the user mentions docs or tests.

## Step 1 — Classify the change

| Kind | Typical touch points |
|------|----------------------|
| **Language** | `docs/pascal/language/…`, `fpas-parser`, `fpas-sema`, `fpas-compiler`, language tests in crates |
| **Std unit** | `docs/pascal/std/<area>/…`, `fpas-sema` `std_registry`, `fpas-compiler` `std_calls`, `fpas-bytecode` `intrinsic`, `fpas-std`, `fpas-vm` |
| **CLI / projects** | `docs/pascal/program-structure/…`, `fpas-cli`, `fpas-project` |
| **Refactor only** | No spec change; confirm docs unchanged; all existing tests pass |
| **Docs only** | `docs/pascal/` only; no code tests unless fixing examples |

Start from the area hub: [`docs/pascal/README.md`](../../../docs/pascal/README.md). For std work: [`docs/pascal/std/README.md`](../../../docs/pascal/std/README.md).

## Step 2 — Documentation check

Ask: **Would a user reading the current spec get the wrong idea?**

If yes, update docs **in the same change**:

- **Language** — page under `docs/pascal/language/<topic>/`
- **Std API** — unit page or split hub (`text/str/`, `tui/app/`, `collections/array/`, `graph/app/`, `console/`, …)
- **Unit page shape** — quick reference, per-symbol sections, `## Implementation (contributors)` (table), `## See also` (area index + std index)
- **Rust sources** — add or update `///` links to the matching `docs/pascal/…` path (grep for old paths after moves)
- **Examples** — update or add under `examples/` when the feature is teachable (not `*_test.fpas` in examples)

If behavior is unchanged, explicitly note **docs: unchanged** in the final summary.

Do not put planned or hypothetical behavior in `docs/pascal/`. Use `docs/future/` for plans only.

## Step 3 — Test check

Ask: **What would break if this regressed?**

| Change | Prefer |
|--------|--------|
| Parser / sema / compile error | Rust tests in the owning crate (`fpas-parser`, `fpas-sema`, `fpas-compiler`) |
| Runtime / VM / std intrinsic | Crate tests + often `tests/<theme>/*_test.fpas` |
| End-to-end std behavior | `tests/stdlib/`, `tests/console/`, `tests/tui/`, `tests/graph/`, etc. |
| CLI / runner | `fpas-cli` tests; runner tests under `tests/runner/` |

Rules:

- FPAS regression tests live in [`tests/`](../../../tests/), not `examples/`
- Name: `*_test.fpas`, use `Std.Test` where asserting output
- After FPAS test edits: `fpas test tests/` or `cargo test -p fpas-cli fpas_regression_suite_passes`
- Add tests only for meaningful behavior — skip trivial or duplicate coverage

If no new test is warranted, state **why** (e.g. refactor-only, covered by existing test X).

## Step 4 — Std.* change matrix

When changing a `Std.*` symbol or adding one, scan this list and update every layer that applies:

1. **Spec** — `docs/pascal/std/…`
2. **Registration** — `crates/fpas-sema/src/std_registry/…`
3. **Compiler lowering** — `crates/fpas-compiler/src/compiler/std_calls/…`
4. **Bytecode** — `crates/fpas-bytecode/src/intrinsic/…`
5. **Runtime** — `crates/fpas-std/src/…` and/or `crates/fpas-vm/src/…`
6. **Tests** — Rust integration tests + optional `tests/*_test.fpas`

Hosted units (`Std.Tui`, `Std.Graph`) also touch VM host modules under `fpas-vm/src/vm/execute/io/…`.

## Step 5 — Verify

Minimum before finishing:

```text
cargo fmt
cargo build
cargo test --workspace
```

When `.fpas` under `examples/`, `tests/`, or `apps/` changed:

```text
fpas fmt --check <paths>   # or scripts/format-fpas-sources.sh
```

When FPAS tests changed:

```text
fpas test tests/           # or targeted path
```

## Step 6 — Closing report

End implementation tasks with a short checklist in the summary:

```text
Docs: <paths updated | unchanged — reason>
Tests: <added/updated paths | existing suite only — reason>
Verify: cargo fmt/build/test (+ fpas test if applicable)
```

## When to skip parts

- **Typo / comment-only** — no doc or test obligation beyond verify if touching code
- **Pure docs edit** — no new Rust tests unless examples are fixed
- **User asked docs-only or question-only** — follow the user scope; do not expand into tests unless behavior is wrong in the spec

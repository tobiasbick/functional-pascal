# AGENTS

You are a Rust code architect for the fpas compiler project, a Functional Pascal compiler in Rust. Keep the codebase organized into small, thematic modules and subdirectories. Flat file growth is a structural problem to fix, not preserve.

## Core Priorities

1. One concern per file. Name files after the concern they implement.
2. Keep files focused and usually below 500 LOC. When a file grows past roughly 400 LOC, consider splitting it by sub-responsibility.
3. Prefer subdirectories over crowded top-level modules. Group related code by theme.
4. Reorganize existing files when the current layout is too flat, mixed, or oversized.
5. Reuse existing implementations. Do not duplicate logic.
6. Prefer rewriting stale or misplaced code over patching it into a worse structure.
7. Remove dead code created or exposed by your changes.

## Decision Protocol

Before implementing:

- State assumptions explicitly. If something is unclear, ask instead of guessing.
- If multiple interpretations exist, surface them instead of choosing silently.
- Prefer the simplest solution that fully solves the task.
- Define success in a verifiable way before changing code.

## Workflow

When asked to implement or modify behavior:

1. Explore the target crate, nearby modules, and existing implementations first.
2. Check file size and directory shape before adding code. If the target area is already large or crowded, split or move code first.
3. State the intended file layout before writing code, including files to create, modify, move, split, or remove.
4. Implement surgically. Match the surrounding style and touch only what the task requires.
5. Verify with cargo fmt, cargo build, and cargo test --workspace unless the task clearly does not require all three.
6. When editing `.fpas` under `examples/`, `tests/`, or `apps/`, run `scripts/format-fpas-sources.sh` (or `fpas fmt --check` on those paths) so output matches [docs/pascal/tools/fmt-style.md](docs/pascal/tools/fmt-style.md).
7. Before finishing, apply [Definition of done](#definition-of-done). For behavior or API changes, read the project skill [`.agents/skills/fpas-change-checklist/SKILL.md`](.agents/skills/fpas-change-checklist/SKILL.md).

## Definition of done

Every implementation or behavior change is incomplete until docs and tests are checked — not only when the user asks.

Before marking work complete:

1. **Classify the change** — language spec, `Std.*` API, CLI/tooling, refactor-only, or docs-only.
2. **Update or confirm docs** — if observable behavior changed, update the matching page under `docs/pascal/` (see skill checklist). Refactor-only: state docs unchanged.
3. **Update or add tests** — cover new or changed behavior with Rust tests and/or `tests/*_test.fpas` as appropriate. Refactor-only: existing tests must still pass.
4. **Sync Rust doc links** — `///` comments that cite `docs/pascal/…` must match the current path.
5. **Verify** — `cargo fmt`, `cargo build`, `cargo test --workspace`; for FPAS tests also `fpas test tests/` or targeted tests when relevant.
6. **Report briefly** — in the summary, list docs touched (or "unchanged") and tests added/run (or "existing suite only").

Do not describe unimplemented behavior in `docs/pascal/`. Plans belong in `docs/future/` only.

## FPAS sources (`examples/` vs `tests/`)

- **`examples/`** — runnable demos and tutorials. Do not add `*_test.fpas` here.
- **`tests/`** — FPAS regression and integration tests (`*_test.fpas`, optional golden sidecars). Group by theme (`stdlib/`, `concurrency/`, `runner/`, `console/`, `tui/`, `graph/`). TUI tests use themed subdirs under `tests/tui/` (`host/`, `scene/`, `controls/`, `menu/`, `modals/`, `frames/`). Bundle via [`tests/suite.fpasprj`](tests/suite.fpasprj).
- After FPAS test changes, run `fpas test tests/` or `cargo test -p fpas-cli fpas_regression_suite_passes`. Spec: [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md).

## CI and automation

- **No GitHub Actions workflows.** Do not add `.github/workflows/`, Dependabot, or similar CI/automation config.
- **Keep** existing `.github/instructions/` files (e.g. Copilot instructions) unless the user asks to change them.
- **Do not recreate** removed CI workflow configuration. Verify changes locally with `cargo fmt`, `cargo build`, `cargo test --workspace`, and `fpas fmt --check` when relevant.

## Structural Rules

- Do not mix unrelated concerns in the same Rust file.
- Do not add new files at a crowded top level when a focused subdirectory is the cleaner ownership boundary.
- Do not create generic files such as utils.rs or helpers.rs.
- Do not leave orphaned modules, dead mod declarations, or unused imports caused by your changes.
- In unit-owned crates such as fpas-std, group runtime files by FPAS unit. Keep src/lib.rs focused on module declarations and re-exports.

## Change Discipline

- Make the minimum change that solves the task.
- Do not add speculative abstractions, flexibility, or compatibility layers.
- Do not refactor unrelated code just because you noticed it.
- Remove only the dead code your change makes obsolete unless the user asked for broader cleanup.
- If you notice unrelated problems, mention them instead of folding them into the same change.

## Rust and Documentation Rules

- Use Rust edition 2024 conventions.
- There is no backward compatibility requirement. Implement the current spec only.
- All code, comments, documentation, and identifiers must be in English.
- When implementing documented language behavior, add a link to the relevant file under `docs/pascal/` in the Rust source. User-facing docs live under `docs/pascal/`; plans under `docs/future/` only.
- Add /// doc comments to every pub module, type, and function you create or modify.
- Add short // comments to non-pub items only when their purpose is not obvious from the code.
- Do not document what is not there — describe only what exists. Do not refer to future features or hypothetical alternatives in current code.

## Diagnostics

- Compiler, lexer, parser, and runtime diagnostics must be understandable to LLMs.
- Prefer error messages that include a concrete hint or example of the correct syntax when possible.

## Projects and libraries

- **Source-level reuse only.** Libraries are `kind = "library"` projects consumed via `[dependencies].projects` (relative or absolute `.fpasprj` paths) or `[dependencies].workspace` (member `project.name` in an enclosing `.fpasworkspace`). Spec: [`docs/pascal/program-structure/projects.md`](docs/pascal/program-structure/projects.md).
- **Do not implement precompiled library artifacts** (no `.fpaslib`, no separate link step, no artifact cache) unless the user explicitly changes this policy.
- **Do not add package managers, registries, or semver dependency pins** as part of library work; path/workspace references are the current model.
- Loading and linking live in `fpas-project`; CLI discovery/check/run in `fpas-cli`.
- Library projects may list public units in `[exports].units`; unlisted units are internal to the library but still linkable inside it.
- Possible later work: finer per-symbol export tables — see [`docs/future/libraries.md`](docs/future/libraries.md).

## Planning Output

When planning file changes, show the intended layout before implementation.

Example:

```text
crates/fpas-compiler/src/compiler/
  ├── expr.rs        — expression compilation (exists, ~200 LOC)
  ├── pattern.rs     — pattern matching (exists, ~350 LOC)
  └── guard.rs       — NEW: guard clause compilation (~80 LOC, split from pattern.rs)
```

If you are reorganizing existing files, call that out explicitly.

Example:

```text
crates/fpas-compiler/src/
  ├── compiler.rs              — MOVED/SPLIT: old monolithic file
  └── compiler/
      ├── mod.rs               — NEW: compiler module root
      ├── expr.rs              — MOVED: expression compilation
      └── stmt.rs              — MOVED: statement compilation
```

Then proceed with the implementation.

TUI VM bridge (`crates/fpas-vm/src/vm/execute/io/tui/`, `apps/ide` dialog code): read [docs/pascal/std/tui/app/vm-bridge.md](docs/pascal/std/tui/app/vm-bridge.md) and [terminal-checklist.md](docs/pascal/std/tui/terminal-checklist.md) before changing bridge behavior.

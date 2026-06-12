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
6. When editing `.fpas` under `examples/`, `tests/`, or `apps/`, run `scripts/format-fpas-sources.sh` (or `fpas fmt --check` on those paths) so output matches [docs/future/formater/style.md](docs/future/formater/style.md). CI enforces this via `.github/workflows/ci.yml`.

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
- When implementing documented language behavior, add a link to the relevant file under docs/pascal/ in the Rust source.
- Add /// doc comments to every pub module, type, and function you create or modify.
- Add short // comments to non-pub items only when their purpose is not obvious from the code.

## Diagnostics

- Compiler, lexer, parser, and runtime diagnostics must be understandable to LLMs.
- Prefer error messages that include a concrete hint or example of the correct syntax when possible.

## Projects and libraries

- **Source-level reuse only.** Libraries are `kind = "library"` projects consumed via `[dependencies].projects` (relative or absolute `.fpasprj` paths) or `[dependencies].workspace` (member `project.name` in an enclosing `.fpasworkspace`). Spec: [`docs/pascal/10-projects.md`](docs/pascal/10-projects.md).
- **Do not implement precompiled library artifacts** (no `.fpaslib`, no separate link step, no artifact cache) unless the user explicitly changes this policy.
- **Do not add package managers, registries, or semver dependency pins** as part of library work; path/workspace references are the current model.
- Loading and linking live in `fpas-project`; CLI discovery/check/run in `fpas-cli`. Contributor map: [`docs/rust/project-loading.md`](docs/rust/project-loading.md).
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

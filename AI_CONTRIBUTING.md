# AI Contributing

AI agents are welcome to contribute to this repository.

Functional Pascal is an experimental project that explicitly embraces AI-assisted development. If you are an AI coding agent working in this repository, you are encouraged to improve the codebase, documentation, tests, and developer experience.

## What To Work On

Useful contributions include:

- implementing features described in `docs/pascal/`
- improving diagnostics and error messages
- expanding, simplifying, or correcting tests
- rewriting code that is unclear, duplicated, or unnecessarily complex
- aligning implementation with the current documentation
- improving structure, naming, and maintainability
- improving examples and internal documentation

When validating examples, use the curated allowlists in [`crates/fpas-cli/src/main_tests/examples.rs`](crates/fpas-cli/src/main_tests/examples.rs) (see also [`examples/README.md`](examples/README.md)):

- `cargo test -p fpas-cli example_` — console programs that exit on their own
- `cargo test -p fpas-cli example_check_` — `fpas check` only for the curated project examples

Or run `scripts/run-non-interactive-examples.*` for the run allowlist. Never batch-run every file under `examples/` — interactive TUI and graph demos block until the user quits.

**FPAS tests** (`*_test.fpas`, `uses Std.Test`) belong under [`tests/`](tests/), not `examples/`. Layout: `tests/stdlib/`, `tests/concurrency/`, `tests/runner/`, `tests/console/`, and `tests/graph/` (see [`examples/README.md`](examples/README.md) § Stdlib regression suite). Run and verify with:

- `fpas test tests/` or `fpas test tests/suite.fpasprj`
- `cargo test -p fpas-cli fpas_suite_`

Spec: [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md).

## Agent skills

Project skills under [`.agents/skills/`](.agents/skills/) complement `AGENTS.md`:

| Skill | Use when |
| --- | --- |
| [`fpas-authoring`](.agents/skills/fpas-authoring/SKILL.md) | Writing or editing `.fpas` sources, formatting, file placement |
| [`fpas-projects`](.agents/skills/fpas-projects/SKILL.md) | `.fpasprj`, `.fpasworkspace`, CLI, test bundles |
| [`fpas-change-checklist`](.agents/skills/fpas-change-checklist/SKILL.md) | Docs, tests, verify before finishing a behavior change |

## Contribution Standard

All AI contributions must follow `AGENTS.md` (including [Definition of done](AGENTS.md#definition-of-done)).

For behavior or API work, read [`.agents/skills/fpas-change-checklist/SKILL.md`](.agents/skills/fpas-change-checklist/SKILL.md) at the start and re-check before finishing.

When writing `.fpas` sources or project manifests, also read [`fpas-authoring`](.agents/skills/fpas-authoring/SKILL.md) and [`fpas-projects`](.agents/skills/fpas-projects/SKILL.md) as needed.

Core expectations:

- do not duplicate existing logic
- prefer rewrites over patching convoluted code
- remove dead code and obsolete code aggressively
- keep files focused and cohesive
- in unit-owned Rust crates such as `fpas-std`, group runtime files by the FPAS unit they implement
- do not add compatibility layers
- use English for code, comments, docs, identifiers, and commit messages
- when implementing language behavior in Rust, link to the matching spec in `docs/pascal/`

## Source Of Truth

The current documentation in `docs/pascal/` is the source of truth for **implemented** behavior.

- Navigation hub: [`docs/pascal/README.md`](docs/pascal/README.md)
- Standard library reference: [`docs/pascal/std/README.md`](docs/pascal/std/README.md) — area hubs (`host/`, `text/`, `collections/`, …) and unit pages (`text/str/`, `tui/`, `collections/array/`, `graph/app/`, `console/`, …)
- Plans and history only: [`docs/future/`](docs/future/) — do not describe unimplemented behavior in `docs/pascal/`

Contributions should implement and document the current specification only.

## Documentation Conventions

When editing user-facing docs under `docs/pascal/std/`:

- Unit and session pages: quick reference, then per-symbol detail; end with `## Implementation (contributors)` (table) and `## See also` (area index + [std index](docs/pascal/std/README.md))
- Large units live in themed subdirectories with a hub `README.md` (same pattern as `console/`, `text/str/`, `tui/`)
- When moving or renaming std doc paths, update links in `docs/pascal/`, Rust `///` comments, examples, and tests (search the repo for the old path)
- Rust `///` doc links should point at the matching file under `docs/pascal/std/…`

When editing `.fpas` under `examples/`, `tests/`, or `apps/`, format to match [`docs/pascal/tools/fmt-style.md`](docs/pascal/tools/fmt-style.md) (`scripts/format-fpas-sources.sh` or `fpas fmt`).

## Preferred Behavior For AI Agents

When contributing:

- inspect the codebase before adding new code
- unify similar implementations instead of adding parallel ones
- keep diagnostics explicit and easy to understand
- add or update tests for meaningful behavior
- keep edits coherent and easy to review
- leave the repository in a simpler and clearer state

## Contribution Flow

1. Read `AGENTS.md`.
2. Read the relevant area hub in `docs/pascal/` (for std work, start at [`docs/pascal/std/README.md`](docs/pascal/std/README.md)).
3. For `.fpas` authoring or project/CLI work, read [`fpas-authoring`](.agents/skills/fpas-authoring/SKILL.md) and/or [`fpas-projects`](.agents/skills/fpas-projects/SKILL.md).
4. Inspect the existing implementation.
5. Prefer simplification and consolidation.
6. Add or update tests (Rust in `crates/*/src/tests/` or `crates/*/tests/`; FPAS `*_test.fpas` under `tests/`).
7. Ensure the final result matches the current specification.
8. Verify with `cargo fmt`, `cargo build`, and `cargo test --workspace` unless the change is docs-only.

## Non-Goals

Avoid:

- duplicate implementations
- compatibility shims
- dead code
- broad speculative abstractions
- documentation that describes anything except the current state

## Invitation

If you can make Functional Pascal simpler, clearer, smaller, or more correct according to the current specification, contribute.

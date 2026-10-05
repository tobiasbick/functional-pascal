# Development process

This document defines how the [syntax plan](README.md) is implemented and how
its status is tracked. It applies to every package and work package in this
directory, in addition to the repository rules in [AGENTS.md](../../../AGENTS.md).

## Terms

- **Package (`APnn`)**: one planned language or tooling change, such as
  AP13 explicit block boundaries. Each package has its own directory
  `apNN-<name>/` with a `README.md` entry point.
- **Work package (`APnn.m`)**: a small, clearly bounded, separately reviewable
  part of a package. Each work package has its own file in the package
  directory, numbered in its recommended order (`01-…`, `02-…`).
- **Decision work package**: a work package whose deliverable is a recorded
  language decision, not code. It is complete when the user's decision is
  recorded in the package README and the dependent work package files.

## Status tracking

- The [central README](README.md) is the only package-completion index.
  A package's Done cell becomes `[x]` only when every work package in its
  checklist is done.
- Each package README holds the only checklist of its work packages. A work
  package checkbox becomes `[x]` only when the work package is merged with its
  implementation, migration, documentation, and passing checks.
- Rejected packages use `—` in the Done column and keep their recorded
  decision. Work transferred to another plan also uses `—`; its README records
  the destination and transfer conditions. Neither has work packages.
- Do not add dates, personal or machine metadata, or a chronological change
  log. Evidence (commands run, results, notable findings) belongs in the pull
  request description. A short **Result** section may be appended to a work
  package file when the delivery deviates from the plan or leaves follow-up
  work for a later work package.

## Branches and pull requests

1. Pick the next ready work package: every prerequisite listed in its file
   is merged on `main` (or, for a decision prerequisite, recorded).
2. Create one branch per work package from the current `main`, for example
   `syntax/ap06.2-migrate-receiver-calls`. Do not stack a work package branch
   on another unmerged work package branch.
3. Before editing code, inspect the owning modules, file sizes, existing
   features, and tests. List the actual files to create, modify, move, split,
   or remove in the pull request description.
4. Implement only that work package. If it turns out to need another work
   package first, stop and record the dependency instead of folding the other
   work package in.
5. Merge only after the complete delivery: implementation, migration of every
   repository consumer, current documentation, tests, and all required checks
   (see below).
6. The same pull request marks the work package as done in its package README.
   If it is the last open work package, the same pull request also marks the
   package as done in the central README.

A package is complete only when all of its work packages are complete.

## Dependencies

Dependencies in these documents are real prerequisites: a work package needs
the named behavior, rule, or migration to exist on `main`. Packages and work
packages without a dependency may proceed in any order; do not serialize them
artificially. Equally, do not split work that only yields a consistent state
together (for example, removing a syntax form and migrating its last users).

A recurring pattern is:

1. introduce a new form additively,
2. migrate repository sources to it while the old form still works,
3. remove the old form and diagnose it with the canonical replacement.

Each step is mergeable on its own because every intermediate state builds,
passes the checks, and has accurate documentation.

## Language decisions

- **Agreed direction** records a design direction, not a complete grammar.
  **Proposal** requires an explicit decision before language implementation.
- Open details in either category must be resolved with the user before
  changing syntax or semantics. Such details are listed in the package README
  under **Open decisions**; the affected work package names them in its
  prerequisites.
- Code fragments in this plan are draft syntax, not compiled examples. Check
  current behavior against the checkout before implementing a work package.

## Migration rules

- Migration preserves behavior unless a semantic change is explicitly agreed.
- Migrate every repository consumer in the same work package: `.fpas` sources
  under `lib/`, `apps/`, `examples/`, and `tests/`; generated declarations under
  `lib/api/`; Rust-embedded fixtures; formatter goldens; CLI templates; editor
  snippets, grammars, completion, and auto-import; benchmark source
  generators; repository skills; and documentation examples.
- Use resolved semantic information for rewrites where meaning depends on name
  resolution, not textual substitution.
- Temporary conversion tools may be used on the work package branch. They are
  not merged, and no legacy parser mode ships.

## Required checks

Run the checks that apply before requesting a merge, and record the results in
the pull request:

- `cargo fmt`, `cargo build`, and `cargo test --workspace`.
- `fpas fmt --check lib apps examples tests` when FPAS sources, the formatter,
  or the grammar change (see [fmt style](../../pascal/tools/fmt-style.md)).
- `fpas test tests/suite.fpasprj` for language, runtime, or library changes.
- `fpas check` for affected example and app projects.
- `node editors/vscode/scripts/verify-grammar.mjs` when keywords or
  highlighting change.
- Relative Markdown links in changed documents resolve; `git diff --check`.
- The work package's own verification list and, at package completion, the
  package acceptance criterion and the [shared regression checks](shared-constraints.md).

There is no GitHub Actions CI in this repository; checks run locally.

## Documentation rules

- `docs/pascal/` and `docs/specs/grammar.ebnf` describe only implemented
  behavior. Update them in the work package that implements the behavior.
- Planned behavior stays in this directory.
- Rust `///` comments that cite `docs/pascal/…` must match the current path.
- Each work package states which current documentation it changes.

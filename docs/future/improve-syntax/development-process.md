# Development process

This process applies to every package and work package in the
[AP plan](README.md), together with [AGENTS.md](../../../AGENTS.md).
Use the [example package](example-apNN-topic/README.md) when creating new work.

## Terms

- A package (`APnn`) groups one planned language, runtime or tooling change.
  Its directory is `apNN-<topic>/`, with a `README.md` entry point.
- A work package (`APnn.m`) is a bounded, separately reviewable part of that
  change. It has one numbered file, such as `01-decision.md` or
  `02-implementation.md`, in the package directory.
- A decision work package delivers a recorded user decision rather than code.
  Its acceptance criterion names the questions that must be resolved.

## Status tracking

The [central README](README.md#package-status) owns package completion and the
status legend. A package's Done cell becomes `[x]` only when every work package
in its README checklist is complete.

Each package README owns the only checklist of its work packages. A work-package
checkbox becomes `[x]` after its implementation, consumer migration,
documentation and applicable verification are complete in the working branch.
For a decision work package, completion means the user's decision is recorded
in the package README and reflected in dependent work-package files.

Known pre-existing check failures need evidence that they are independent of
the delivery. Failures caused by the work package keep its checkbox open.
Completion describes delivered work, not whether it has been committed,
merged or pushed. Git and pull requests track integration separately.

`Decision pending` means a required choice is unresolved. `Deferred` means the
user has parked the proposal with no scheduled implementation; reopening needs
a new decision. Rejected and transferred packages use `—` in the Done column
and retain the rationale or linked destination, without active implementation
work packages. Effort and priority do not replace a decision status.

Keep planning documents in English. Do not add dates, personal or machine
metadata, or chronological delivery logs. Commands and results belong in the
delivery report or pull request. Completed work-package files describe the
delivered contract, implementation owners, regression coverage and actual open
follow-ups. Remove intermediate states, superseded rules and stale test counts.

## Decision gates

Before implementation, state the current behavior, proposed replacement,
preserved behavior, exclusions and acceptance criterion. Record language
decisions in the owning package README, with their IDs and affected work
packages. An agreed direction is not a complete specification.

List unresolved details under Open decisions and name them as prerequisites of
the affected work packages. Resolve them with the user before changing syntax,
semantics or user-facing language specifications. Draft examples stay in this
plan and are identified as proposals. Check the current checkout before relying
on any earlier plan or result.

## Dependencies and branches

1. Pick a ready work package. Each prerequisite must be implemented on `main`,
   or its required decision must be recorded.
2. Create a branch from current `main`, for example
   `codex/ap47.2-implement-topic`. Do not base it on an unmerged work-package
   branch unless the user selected an integration branch for the campaign.
3. Inspect owning modules, file sizes, nearby implementations and tests.
   Before editing, list the actual paths to create, modify, move, split or
   remove and explain each purpose. Record the scope in the work-package file.
4. Implement that bounded scope. If another prerequisite is missing, record
   the dependency and resolve it before continuing dependent work.
5. Complete migration, documentation, verification and review. Then check off
   the work package in its package README. If it is the last open part, update
   the central package row as well.
6. Commit, push and merge only as authorized by the user. Completion checks
   still apply before requesting integration.

On a user-selected integration branch, completed prerequisites on that branch
may be used before they are merged into `main`. Independent packages need no
artificial ordering. Work that only produces a consistent result together
belongs in one work package.

## Migration

Preserve behavior unless the user explicitly agrees to a semantic change.
Implement the selected contract without a compatibility layer or a shipped
legacy parser mode. Choose intermediate steps only when each step builds,
passes its applicable checks and has accurate current documentation.

Inspect and migrate every affected consumer: `lib/`, `apps/`, `examples/`,
`tests/`, generated `lib/api/` declarations, Rust-embedded fixtures, formatter
goldens, CLI templates, editor snippets and grammars, completion and auto-import,
benchmark source generators, repository skills and documentation examples.
Use resolved semantic information when a rewrite depends on name resolution.
Temporary conversion helpers stay outside tracked deliverables.

## Review

Review the final change against both the approved contract (Spec) and repository
rules, module ownership and test quality (Standards). Record the selected work
packages and fixed baseline/target revisions. Confirm findings with concrete
evidence, expected versus actual behavior, an owning path and a bounded repair.
Keep unresolved suspicions out of the required repair list.

Repair confirmed findings at their owning boundary, run the relevant checks and
review the result again. Review completion does not authorize a language change,
commit, push or merge. Preserve any coverage limits in the delivery report.

## Required checks

Choose checks from the actual change and record what ran and its result:

- Rust changes: `cargo fmt`, `cargo build`, `cargo test --workspace`.
- FPAS source, formatter or grammar changes:
  `fpas fmt --check lib apps examples tests`; see
  [formatter style](../../pascal/tools/fmt-style.md).
- Language, runtime or library changes: `fpas test tests/suite.fpasprj`.
- Affected app and example projects: `fpas check <project.fpasprj>`.
- Editor keyword or highlighting changes:
  `node editors/vscode/scripts/verify-grammar.mjs`, plus affected editor tests.
- Documentation changes: relative links, stated commands and examples;
  `git diff --check`.
- Every work package: its own verification list. At package completion, also
  check the package acceptance criterion and
  [shared regression boundaries](shared-constraints.md).

Docs-only changes need document validation, not an unrelated build or test run.
There is no GitHub Actions CI; checks run locally. Existing tests may provide
sufficient coverage when their observable contract matches the change. Avoid
duplicate tests that only restate an implementation.

## Documentation

Planned behavior stays under `docs/future/`. Current behavior belongs under
`docs/pascal/` and `docs/specs/grammar.ebnf`; update it with the implementation.
Every work package names the documentation it affects. Rust doc comments that
cite handbook files must point to their current paths.

Language and authoring changes also update or confirm the
[agent reference](../../pascal/agents/README.md) under its
[maintenance guide](../../pascal/agents/maintenance.md).
Reuse its existing executable example test. For `Std.*` changes, check the
[shared implementation touchpoints](../../pascal/std/README.md#shared-implementation-touchpoints).

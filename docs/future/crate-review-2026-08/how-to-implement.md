# How to implement a crate-review task

Implement exactly one **open** numbered task from [`tasks/`](tasks/) unless the user explicitly
groups tasks. A task marked **decision required** is blocked until the user records the selected
behavior in that file. A task marked **coverage** changes tests only.

## Before editing

1. Read the workspace `AGENTS.md` and
   [`.agents/skills/fpas-change-checklist/SKILL.md`](../../../.agents/skills/fpas-change-checklist/SKILL.md).
2. Read [`README.md`](README.md), the selected task, and every specification page it cites.
3. Inspect the target crate, nearby modules, existing helpers, tests, file sizes, and directory
   shape. Do not rely on the review's source location without rechecking it.
4. State assumptions, a verifiable success condition, and the intended file layout before editing.
5. Change `Status: open` to `Status: in progress` and add a progress record:

```text
## Progress

- Base commit: <commit>
- Current step: <specific next action>
- Files changed: none
- Verification: not run
- Blockers: none
```

Update this record before stopping so another model can resume after context loss.

## Implementation rules

- Make the smallest complete change that fixes the verified cause.
- Reuse existing logic; do not create generic `utils.rs` or duplicate a parser, resolver, error
  path, or scheduler path.
- Follow the task's language gate. If current tests or docs contradict the task, stop, set the task
  back to `open`, and record the contradiction instead of changing semantics silently.
- Add the named regression tests. Do not weaken an existing assertion or use `#[ignore]`.
- For `.fpas` changes, follow the `fpas-authoring` and `fpas-projects` skills as applicable and run
  the FPAS formatter check required by `AGENTS.md`.
- Keep code, comments, diagnostics, tests, and repository documentation in English and free of host
  metadata.

## Verification

Run the task's focused commands first. Before completion, also run the workspace definition of
done unless the task documents a concrete reason why one command is inapplicable:

```text
cargo fmt
cargo build
cargo test --workspace
```

For changed FPAS tests, also run the relevant `fpas test` target and `fpas fmt --check` (or
`scripts/format-fpas-sources.sh`). Record exact commands and results in the task's progress section.

## Completion

1. Update current behavior documentation under `docs/pascal/` when observable behavior changed;
   otherwise record `Docs: unchanged — implementation now matches the existing contract`.
2. Set `Status: complete` only after code, tests, docs, and required verification pass.
3. Record touched docs, tests, commands, and the implementation commit in the task.
4. Remove the completed future task and its index entry once the implementation and current docs
   are committed. Do not retain a historical “used to behave differently” note.

## Test placement

| Kind | Location |
|---|---|
| Lexer / parser / sema / compiler | Existing thematic module under `crates/<crate>/src/tests/` |
| Std / VM runtime | Crate tests and a focused `tests/.../*_test.fpas` regression when appropriate |
| Formatter | `crates/fpas-fmt/tests/` using existing round-trip/golden helpers |
| CLI | Existing thematic tests in `crates/fpas-cli` |
| Debugger | `crates/fpas-debug/tests/` or the owning module's unit tests |

Never add FPAS tests under `examples/`, and never create a new public API solely as a test hook.

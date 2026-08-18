# How to implement these tasks (cheaper model)

Read this file once, then implement **exactly one** numbered task from [`tasks/`](tasks/). Do not start a second task in the same session unless the user names it.

These items are **bugfixes against the current spec**. They are not language-design work.

## Session protocol

1. Open [`README.md`](README.md) and pick the next **open** task in the recommended order, or the task the user named.
2. Read **only** that task file plus the spec pages it cites.
3. Read the listed source files before editing. Match surrounding style.
4. Implement the minimum change. Do not refactor unrelated code.
5. Add the tests the task names. Do not add extra fixtures, helpers, or abstractions.
6. Run the verify commands in the task. Then `cargo fmt`.
7. In the closing message, fill the checklist from [`.agents/skills/fpas-change-checklist/SKILL.md`](../../../.agents/skills/fpas-change-checklist/SKILL.md):
   - Docs: paths updated, or `unchanged — bugfix vs existing spec`
   - Tests: paths added/updated
   - Verify: commands run

Do **not** edit this plan directory unless the user asks you to mark a task done.

## Hard rules

- **Language gate:** Do not change FPAS syntax, semantics, or pages under `docs/pascal/language/` unless the task says the spec itself is wrong **and** the user already agreed. These tasks implement the spec as written.
- **English only** in code, comments, tests, and docs.
- **No new keywords** (`private`, `opaque`, …). Visibility stays `public` vs default.
- **No host metadata** (hostnames, usernames, home paths) in the repo.
- **No GitHub Actions / CI config.**
- Do not invent package managers, registries, or caches.
- Workspace forbids `unsafe_code`. Do not add `unsafe`.
- Prefer `Result` over `unwrap` / `expect` / `panic` in production code.
- One concern per file. If a file is already huge, add the fix next to the existing logic; do not start a drive-by split unless the task says to.

## Where tests go

| Kind | Put them here |
|------|----------------|
| Lexer / parser / sema / compiler unit tests | `crates/<crate>/src/tests/…` next to the existing module the task names |
| Std / VM runtime | crate tests **and** `tests/stdlib/…/*_test.fpas` when the task says so |
| Formatter | `crates/fpas-fmt/tests/` using `assert_round_trip` / `assert_golden` |
| CLI | `crates/fpas-cli` tests |
| Debugger | `crates/fpas-debug` tests |

FPAS tests: `*_test.fpas` under `tests/`, never under `examples/`. After adding `.fpas` files, run `fpas fmt --check` on those paths (or `scripts/format-fpas-sources.sh`).

Reuse helpers already in the crate (`check_ok` / `check_errors`, `assert_succeeds`, `lex_with_errors`). Do not create `utils.rs`.

## If you get stuck

- Difficulty **easy**: finish it. If a test already exists and contradicts the task, stop and report the contradiction; do not “fix” the spec.
- Difficulty **medium**: finish it, but do not expand scope when a nearby function looks messy.
- Difficulty **hard**: if the first approach would break existing generic / task / DAP tests, **stop**, describe what you observed, and do not land a partial guess. Leave the task for a stronger model.

Never silence a failing existing test with `#[ignore]` or by weakening the assertion.

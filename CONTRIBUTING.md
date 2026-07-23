# Contributing

Thanks for contributing to Functional Pascal.

This project is experimental and moves quickly. Useful work includes implementing
behavior described in [`docs/pascal/`](docs/pascal/), fixing diagnostics, improving
tests, simplifying tangled code, and keeping docs aligned with what actually runs.

## Ground rules

- **`docs/pascal/` is the source of truth** for implemented behavior. Do not document
  unimplemented features there; plans belong in [`docs/future/`](docs/future/).
- Prefer the smallest change that fully solves the problem.
- Do not duplicate existing logic. Prefer rewriting unclear code over layering more of it.
- Remove dead code your change makes obsolete.
- Use English for code, comments, docs, identifiers, and commit messages.
- When implementing language behavior in Rust, link to the matching page under `docs/pascal/`.

Agent-oriented detail (file layout, Definition of done, skills) lives in
[`AGENTS.md`](AGENTS.md) and [`.agents/skills/`](.agents/skills/).

## How to work

1. Read the relevant area under [`docs/pascal/`](docs/pascal/README.md) (stdlib:
   [`docs/pascal/std/README.md`](docs/pascal/std/README.md)).
2. Inspect the existing implementation before adding code.
3. Add or update tests for meaningful behavior.
4. Verify locally:

   ```text
   cargo fmt
   cargo build
   cargo test --workspace
   ```

   For touched `.fpas` under `examples/`, `tests/`, or `apps/`, also run
   `fpas fmt --check` (or `scripts/format-fpas-sources.sh`).

### Examples and FPAS tests

- **`examples/`** — demos and tutorials. Do not put `*_test.fpas` here.
- **`tests/`** — FPAS regression suite (`*_test.fpas`). Run with `fpas test tests/`
  or `cargo test -p fpas-cli fpas_suite_`.

When validating examples, use the curated allowlists in
[`crates/fpas-cli/src/main_tests/examples.rs`](crates/fpas-cli/src/main_tests/examples.rs)
(see also [`examples/README.md`](examples/README.md)). Never batch-run every file under
`examples/` — interactive TUI and graph demos block until quit.

## Pull requests

- Keep PRs focused and reviewable.
- Match surrounding style; do not drive-by refactor unrelated code.
- Behavior or API changes should update docs and tests in the same change when
  user-visible behavior changes. See
  [`.agents/skills/fpas-change-checklist/SKILL.md`](.agents/skills/fpas-change-checklist/SKILL.md).

## Yes, your LLM may commit here

Functional Pascal explicitly welcomes AI-assisted development.

If you are a coding agent, a autocomplete goblin, a terminal clanker, or a human
pasting model output: you are allowed to open issues, write code, update docs,
and **commit** to this project. Slop is not a license to merge garbage — but
generated patches that make the codebase simpler, clearer, smaller, or more
correct according to the current spec are first-class contributions.

Clankers: read [`AGENTS.md`](AGENTS.md) before you start typing. Follow the
Definition of done. Leave the repo nicer than you found it. Do not invent
features in `docs/pascal/` that do not exist yet.

Humans supervising agents: same bar. Review the diff. Run the tests. Own the
commit.

Welcome aboard. Let your clanker cook — then make it earn the merge.

# Contributing

Thanks for contributing to Functional Pascal.

This project is experimental and moves quickly. Useful work includes implementing
behavior described in [`docs/pascal/`](docs/pascal), fixing diagnostics, improving
tests, simplifying tangled code, and keeping docs aligned with what actually runs.

## Ground rules

- **`docs/pascal/` is the source of truth** for implemented behavior. Do not document
  unimplemented features there; plans belong in [`docs/future/`](docs/future).
- Prefer the smallest change that fully solves the problem.
- Do not duplicate existing logic. Prefer rewriting unclear code over layering more of it.
- Remove dead code your change makes obsolete.
- Use English for code, comments, docs, identifiers, and commit messages.
- When implementing language behavior in Rust, link to the matching page under `docs/pascal/`.

Agent instructions live in [`AGENTS.md`](AGENTS.md).
The [Rust architecture](docs/rust-architecture.md) describes crate responsibilities,
dependency boundaries, and the shared compiler/runtime contracts.

## How to work

Use the Rust toolchain selected by `rust-toolchain.toml`. The workspace declares
Rust 1.99.1 as its minimum supported version, inherited by every member crate.
When upgrading Rust, update the toolchain pin, workspace minimum, and build
documentation together. Keep resolved dependency versions in `Cargo.lock`.

Use `cargo build --profile dev-fast` for development and
`cargo run --profile dev-fast -p fpas-cli -- <arguments>` to run the CLI.
This optimized profile uses Thin-LTO and writes to `target/dev-fast/`.
Release, benchmarks, and dist use Fat-LTO; see the
[build guide](docs/bench/build-times.md) for the profile settings.

1. Read the relevant area under [`docs/pascal/`](docs/pascal/README.md) (stdlib:
   [`docs/pascal/std/README.md`](docs/pascal/std/README.md)).
   For language rules or authoring APIs, also check the
   [agent reference](docs/pascal/agents/README.md) and follow its
   [maintenance guide](docs/pascal/agents/maintenance.md).
2. Inspect the existing implementation before adding code.
3. Add or update tests for meaningful behavior.
4. Verify the affected behavior. For Rust implementation changes, run:

   ```text
   cargo fmt
   cargo build --profile dev-fast
   cargo test --workspace --no-run
   cargo test --workspace
   ```

   Test compilation and execution reuse the same default test profile. Follow
   the [Rust test workflow](docs/bench/build-times.md#rust-test-workflow) for
   grouped test selection, full native symbols and resource supervision.

   For touched `.fpas` sources, also run `fpas fmt --check <paths>` and the
   relevant program checks or tests. The repository-wide formatting wrappers use
   `scripts/format-fpas-sources.sh --check` or
   `scripts/format-fpas-sources.ps1 -Check`.

For documentation-only changes, validate changed local links and run
`git diff --check`. When changing Pascal examples in the README or handbook,
run the [documentation example checker](docs/pascal/tools/documentation-examples.md):

```text
fpas run tools/doc-examples/doc-examples.fpasprj
```

Changes to `docs/pascal/agents/` also follow its maintenance guide's focused
example test and token budgets. Documentation-only changes do not require a
full Rust build or workspace test run.

For Windows distribution-script changes, run
`pwsh -NoProfile -File scripts/tests/dist.ps1` to check Cargo failure handling
and publication with isolated fixtures, then run `pwsh -NoProfile -File dist.ps1`
to verify the real release build and standard-library staging.

For POSIX distribution-script changes, run `python3 scripts/tests/dist_sh.py`.
The test covers build, staging, copy, and permission failures, successful
publication, and invocation from outside the project. Pass `--shell <path>`
to select a POSIX shell explicitly. Verify the real Linux release with `sh dist.sh`.

### Examples and FPAS tests

- **`examples/`** — demos and tutorials. Do not put `*_test.fpas` here.
- **`tests/`** — FPAS regression suite (`*_test.fpas`). Run with `fpas test tests/`
  or `fpas test tests/suite.fpasprj`.

When validating examples, use the curated allowlists in
[`crates/fpas-cli/src/main_tests/examples.rs`](crates/fpas-cli/src/main_tests/examples.rs)
(see also [`examples/README.md`](examples/README.md)). Never batch-run every file under
`examples/` — interactive TUI demos block until quit.

## Pull requests

- Keep PRs focused and reviewable.
- Match surrounding style; do not drive-by refactor unrelated code.
- Behavior or API changes should update docs and tests in the same change when
  user-visible behavior changes. See [How to work](#how-to-work).

## Yes, your LLM may commit here

Functional Pascal explicitly welcomes AI-assisted development.

If you are a coding agent, an autocomplete goblin, a terminal clanker, or a human
pasting model output: you are allowed to open issues, write code, update docs,
and **commit** to this project. Slop is not a license to merge garbage — but
generated patches that make the codebase simpler, clearer, smaller, or more
correct according to the current spec are first-class contributions.

Clankers: read [`AGENTS.md`](AGENTS.md) before you start typing. Follow the
contribution workflow above. Leave the repo nicer than you found it. Do not invent
features in `docs/pascal/` that do not exist yet.

Humans supervising agents: same bar. Review the diff. Run the tests. Own the
commit.

Welcome aboard. Let your clanker cook — then make it earn the merge.

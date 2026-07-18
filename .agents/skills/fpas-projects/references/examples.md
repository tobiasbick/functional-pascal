# FPAS projects examples

Calibration for `fpas-projects`. User request → expected agent behavior.

## Example 1: New application project

User request:

```text
create a program project for my-cli app
```

Expected behavior:

- Add `my-cli.fpasprj` with `kind = "program"`, `main` pointing at the `program` file.
- `[sources].include` covers all `src/**/*.fpas`.
- `fpas check my-cli.fpasprj` before claiming success.
- If it depends on a library, add `[dependencies].projects` or `workspace` — not both unless needed.

## Example 2: Library consumed by an app

User request:

```text
add a shared greet library the hello app can use
```

Expected behavior:

- Library: `kind = "library"`, units under `src/`, optional `[exports].units`.
- Consumer program: `[dependencies].projects = ["../libs/greet/greet.fpasprj"]` or `workspace = ["greet"]` when inside a workspace.
- Copy layout from `examples/pascal/monorepo/`.
- Verify: `fpas check` on both manifests; `uses Demo.Greet` (or exported unit name) in the program.

## Example 3: Workspace for monorepo

User request:

```text
wire hello, greet, and tests into one workspace
```

Expected behavior:

- Create `.fpasworkspace` listing each member `.fpasprj` path.
- Do **not** move dependency declarations into the workspace file.
- `fpas check` from workspace root checks all members.
- `fpas test` with no path runs `kind = "test"` members only.

## Example 4: Run regression suite

User request:

```text
run all fpas tests
```

Expected behavior:

```text
fpas test tests/
fpas test tests/suite.fpasprj
cargo test -p fpas-cli fpas_suite_
```

Pick one full-suite command; use targeted `fpas test tests/<theme>/` when iterating.

## Example 5: Check without running interactive demo

User request:

```text
verify ide project compiles
```

Expected behavior:

```text
fpas check apps/ide/ide.fpasprj
```

Do not `fpas run` interactive TUI/graph demos in batch — they block until quit. Use check + targeted tests.

## Example 6: Extend test bundle glob

User request:

```text
add tests under tests/newarea/ to the suite
```

Expected behavior:

- Edit `tests/suite.fpasprj` `[sources].include` to add e.g. `"newarea/**/*_test.fpas"`.
- `fpas test tests/suite.fpasprj --list` to confirm discovery.
- Follow `fpas-change-checklist` if runner behavior changed.

## Example 7: fmt in CI-style verify

User request:

```text
make sure examples and tests are formatted
```

Expected behavior:

```text
fpas fmt --check examples/ tests/ apps/
```

Or `scripts/format-fpas-sources.sh` / `.ps1`. Style spec: [`fmt-style.md`](../../../../docs/pascal/tools/fmt-style.md).

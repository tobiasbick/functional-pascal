# Formatter CLI (`fpas fmt`)

**Status: deferred.** Input rules, globs, and discovery are discussed after the `fpas-fmt` crate plan is settled. See [implementation.md](implementation.md). Output shape is defined in [style.md](style.md) (blank lines, mandatory `begin`/`end`, etc.).

## Open questions

- [ ] Single path vs multiple `.fpas` paths vs directory argument.
- [ ] Shell glob `fpas fmt *.fpas` — rely on shell expansion + multi-path, or built-in glob (workspace already uses `glob` in `fpas-project`)?
- [ ] Parity with `fpas check`: optional path, `.fpas` / `.fpasprj` / `.fpasworkspace`, cwd discovery.
- [ ] `fpas fmt` with no args: format all project sources or error?
- [ ] `--check` only (no writes) for CI.
- [ ] Stdout mode (`fpas fmt --stdout file.fpas`) for piping.
- [ ] Exit codes: `0` ok, `1` parse/format error, `2` `--check` would have changed files (if adopted).

## Likely reuse

- [`crates/fpas-cli/src/cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs) — `CliMode::Fmt`, help text.
- [`crates/fpas-cli/src/cli_check.rs`](../../../crates/fpas-cli/src/cli_check.rs) — read sources, collect paths from `load_project` / `load_workspace`.
- Per file: read → `parse_compilation_unit` → `fpas_fmt::format_compilation_unit` → write if changed.

## Non-goals for CLI v1

- Formatting non-`.fpas` files.
- Watch mode / format on save.

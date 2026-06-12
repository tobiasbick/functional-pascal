# Formatter CLI (`fpas fmt`)

**Status: v1 + v2 Phase 4.** Output shape: [style.md](style.md). Emitter: [`crates/fpas-fmt/`](../../../crates/fpas-fmt/).

## Usage

```text
fpas fmt [<path>...]
fpas fmt [--check] [--list] [<path>...]
fpas fmt --stdout <file.fpas>
fpas fmt                                              # discover `.fpasworkspace` or `.fpasprj` in cwd
```

Each `<path>` may be:

- a `.fpas` file
- a directory (all `.fpas` files recursively; skips `target/`)
- a `.fpasprj` or `.fpasworkspace` (formats every listed source)
- a glob pattern containing `*`, `?`, or `[` (expanded by the CLI, e.g. `src/**/*.fpas`)

## Options

| Flag | Meaning |
|------|---------|
| `--check` | No writes; exit `2` if any file would change |
| `--list` | With `--check`, print paths that would change (one per line on stdout) |
| `--stdout` | Print formatted text to stdout for exactly one `.fpas`; do not write the file |

`--stdout` and `--check` are mutually exclusive.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success (all files formatted, or `--check` found no changes) |
| `1` | I/O, parse, project-load, or usage error |
| `2` | `--check` would modify one or more files |

## Implementation

- [`crates/fpas-cli/src/cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs) — `fpas fmt`, flags, discovery.
- [`crates/fpas-cli/src/cli_fmt/`](../../../crates/fpas-cli/src/cli_fmt/) — path resolution, read → `parse_compilation_unit` → `fpas_fmt::format_source` → write or stdout.
- [`scripts/format-fpas-sources.sh`](../../../scripts/format-fpas-sources.sh) — format `examples/`, `tests/`, `apps/` in one command.
- [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml) — `fpas fmt --check` on pull requests.

## Deferred (post-v2 Phase 1)

- Watch mode / format on save.
- LSP format-on-save integration.

## Non-goals

- Formatting non-`.fpas` files.
- Formatting invalid or partial syntax (recovery).
- Configurable style (`.fpasfmt.toml`, line width overrides) — one official style only ([style.md](style.md)).

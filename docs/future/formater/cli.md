# Formatter CLI (`fpas fmt`)

**Status: implemented (v1).** Output shape: [style.md](style.md). Emitter: [`crates/fpas-fmt/`](../../../crates/fpas-fmt/).

## Usage

```text
fpas fmt [<file.fpas | file.fpasprj | file.fpasworkspace>]
fpas fmt [--check] [<file.fpas | file.fpasprj | file.fpasworkspace>]
fpas fmt                                              # discover `.fpasworkspace` or `.fpasprj` in cwd
```

- **Single path** (same discovery rules as `fpas check`).
- **Shell globs** — rely on shell expansion (`fpas fmt src/*.fpas` passes multiple invocations or one path per run; v1 accepts one positional path like `check`).
- **Projects / workspaces** — formats every `.fpas` file listed in `project.sources` for the project or each workspace member.
- **Write in place** only when normalized content would change (LF output per [style.md](style.md)).
- **`--check`** — no writes; exit `2` if any file would change (CI).

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success (all files formatted, or `--check` found no changes) |
| `1` | I/O, parse, or project-load error |
| `2` | `--check` would modify one or more files |

## Implementation

- [`crates/fpas-cli/src/cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs) — `fpas fmt`, `--check`, discovery.
- [`crates/fpas-cli/src/cli_fmt.rs`](../../../crates/fpas-cli/src/cli_fmt.rs) — read → `parse_compilation_unit` → `fpas_fmt::format_compilation_unit` → write.

## Deferred (post-v1)

- Multiple positional paths in one invocation.
- Built-in glob (workspace already uses `glob` in `fpas-project`).
- `fpas fmt --stdout file.fpas` for piping.
- Watch mode / format on save.

## Non-goals (v1)

- Formatting non-`.fpas` files.
- Formatting invalid or partial syntax (recovery).

# AP02.4: JSON diagnostic output

Package: [AP02: Structured diagnostics](README.md)

## Scope

Add `--diagnostics json` to `fpas check`, `build`, `run`, and `test`. Emit one
JSON object per line with code, severity, file, line, column, end position,
message, expected, found, and hint (Q01).

## Prerequisites

- AP02.3 (structured project and build records).

## Implementation

- Add the option to the four commands and route diagnostics through one
  output module instead of growing the command files.
- Emit UTF-8 JSON Lines on stderr; program stdout is unchanged. In JSON mode,
  progress lines and test summaries must not enter the diagnostic stream.
- Pass the mode to the runner process without consuming FPAS application
  arguments. Wrap program-written stderr as a distinguishable program-output
  event so it cannot be mistaken for a compiler diagnostic.
- Keep exit codes unchanged in both modes.

## Affected areas

- `crates/fpas-cli/src/` (input parsing, `cli_check.rs`, `cli_build.rs`,
  `cli_run.rs`, `cli_test/`, runner binary, a focused diagnostics output module).
- `crates/fpas-diagnostics/src/render.rs` (JSON renderer).

## Migration

None.

## Documentation

- Document the option and envelope in the diagnostics page and in
  `docs/pascal/program-structure/cli.md`.

## Verification

- Real-process tests for all four commands in text and JSON mode: lexer,
  parser, semantic, project, runtime, multiple, imported, and no-location
  errors; Unicode columns; program stderr; runner startup failures; exit codes.

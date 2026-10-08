# AP02: Structured diagnostics

Status: complete (Q01). Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Tools and people can locate an error and determine its cause without parsing
explanatory prose. Superseded or commonly expected forms produce a hint with
the canonical spelling.

## Decisions (Q01)

- Use stable numeric diagnostic codes with ranges per phase: `FP1xxx` for
  lexer errors, `FP2xxx` for parser errors, `FP3xxx` for semantic checks,
  `FP4xxx` for compiler/project/build diagnostics, and `FP5xxx` for runtime diagnostics.
- Show the codes in both human-readable and machine-readable output.
- `--diagnostics json` is available on `check`, `build`, `run`, and `test`, emitting
  one JSON object per line with code, severity, phase, source, location,
  message, expected, found, and hint.
- The [diagnostics reference](../../../pascal/tools/diagnostics.md) lists
  every allocated code with its cause, incorrect use and correction.

## Cross-package rule

For every syntax change in this plan, the owning work package recognizes the
superseded or commonly expected form (classic Pascal, Delphi, or other
mainstream habits) and names the canonical replacement in the diagnostic. That
diagnostic ships with the owning work package, not with AP02.

## Resolved decisions

- **Code identity:** the shared catalog uses stable `FPnxxx` identifiers
  in producers, tools, tests and current documentation.
- **FP4xxx allocation:** compiler codes use FP4000–FP4099; project, build,
  linker, CLI and test-runner codes use FP4100–FP4999. Internal invariants
  use FP9xxx. These subranges retain distinct compiler/project phase values.
- **Transport:** the shared `Diagnostic` holds the code, severity, message,
  optional byte span and expected/found/hint. `FileDiagnostic` carries the
  authoritative path. JSON exposes that path as `source`, and one-based scalar
  coordinates as `location.start` and exclusive `location.end`.

There are no remaining open decisions for AP02.

## Dependencies

None. AP03, AP04, AP05, AP07, AP09, AP13, AP22, and AP27 depend on this package.

## Work packages

- [x] [AP02.1: Diagnostic audit and code scheme](01-audit-and-code-scheme.md)
- [x] [AP02.2: Shared diagnostic record](02-shared-diagnostic-record.md)
- [x] [AP02.3: Project and build diagnostics](03-project-and-build-diagnostics.md)
- [x] [AP02.4: JSON diagnostic output](04-json-output.md)
- [x] [AP02.5: Diagnostics reference](05-diagnostics-reference.md)
- [x] [AP02.6: Parameter declaration diagnostics](06-parameter-declaration-diagnostics.md)

## Acceptance

Tools can locate a representative error and determine its cause without
parsing explanatory prose, and superseded forms produce a hint with the
canonical spelling.

## Result

All six work packages are complete. The [implementation map](implementation-audit.md)
identifies the shared model, current producers/consumers, transport rules and
regression coverage.

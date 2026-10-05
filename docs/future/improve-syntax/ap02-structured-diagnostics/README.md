# AP02: Structured diagnostics

Status: agreed direction (Q01). Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Tools and people can locate an error and determine its cause without parsing
explanatory prose. Superseded or commonly expected forms produce a hint with
the canonical spelling.

## Decisions (Q01)

- Use stable numeric diagnostic codes with ranges per phase: `FP1xxx` for
  lexer errors, `FP2xxx` for parser errors, `FP3xxx` for semantic checks,
  `FP4xxx` for project/build diagnostics, and `FP5xxx` for runtime diagnostics.
- Show the codes in both human-readable and machine-readable output.
- Add `--diagnostics json` to `check`, `build`, `run`, and `test`, emitting
  one JSON object per line with code, severity, file, line, column, end position,
  message, expected, found, and hint.
- Add a diagnostics reference under `docs/pascal/tools/` during implementation,
  listing every code with a wrong and a corrected example.

## Cross-package rule

For every syntax change in this plan, the owning work package recognizes the
superseded or commonly expected form (classic Pascal, Delphi, or other
mainstream habits) and names the canonical replacement in the diagnostic. That
diagnostic ships with the owning work package, not with AP02.

## Open decisions

- **Code numbering versus existing codes.** The checkout already has stable
  `Fxxxx` codes in `crates/fpas-diagnostics/src/codes.rs` with different phase
  ranges (`F0xxx` lexer, `F1xxx` parser, `F2xxx` semantic, `F3xxx` compiler,
  `F4xxx` runtime, `F9xxx` internal). Q01 implies renumbering them. The
  reference branch `codex/syntax-changes` kept the existing `Fxxxx` identities
  and added a project/build range instead. Confirm before AP02.2 whether Q01's
  `FPnxxx` scheme replaces the existing codes.

## Dependencies

None. AP03, AP04, AP05, AP07, AP09, AP13, AP22, and AP27 depend on this package.

## Order

AP02.1 audits and fixes the code scheme. AP02.2 extends the shared record;
AP02.3 carries it through project and build; AP02.4 adds JSON output. AP02.5
documents the catalog. AP02.6 is independent of AP02.3–AP02.5.

## Work packages

- [ ] [AP02.1: Diagnostic audit and code scheme](01-audit-and-code-scheme.md)
- [ ] [AP02.2: Shared diagnostic record](02-shared-diagnostic-record.md)
- [ ] [AP02.3: Project and build diagnostics](03-project-and-build-diagnostics.md)
- [ ] [AP02.4: JSON diagnostic output](04-json-output.md)
- [ ] [AP02.5: Diagnostics reference](05-diagnostics-reference.md)
- [ ] [AP02.6: Parameter declaration diagnostics](06-parameter-declaration-diagnostics.md)

## Acceptance

Tools can locate a representative error and determine its cause without
parsing explanatory prose, and superseded forms produce a hint with the
canonical spelling.

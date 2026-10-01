# Stage 2: Diagnostics

Prerequisite: the [language contract](../language-contract.md).
Completion is tracked in the [steering plan](../README.md).

## Target

Extend `fpas-diagnostics`; do not create a parallel error system. Keep existing
`Fxxxx` codes and their meanings, including current phase assignments. Add unused
codes for new errors and a project/build range after auditing the registry.
The former proposal to rename all codes to `FPxxxx` is removed.

Every diagnostic has a code, severity, phase, source location where available,
message, and optional expected/found/hint information. An unavailable location
is null, not a fabricated line zero. Spans include start and end positions.
Text and JSON render the same diagnostic record and code.

Add `--diagnostics json` to `check`, `build`, `run`, and `test`. Emit UTF-8 JSON
Lines on stderr, one complete diagnostic object per line; program stdout is
unchanged. In this mode, tool progress and test summaries must not contaminate the
diagnostic stream. Program-written stderr remains a separate concern: capture and
forward it as a distinguishable program-output JSON event so it cannot masquerade
as a compiler diagnostic. Document this envelope and preserve text-mode behavior.
Locations use one-based lines/columns, exclusive end positions, and a documented
Unicode column unit; use Unicode scalar positions consistently across renderers.

No heuristic should invent an application-level decision. A superseded syntax
form receives the canonical replacement; a semantic ambiguity explains what
information is missing. Diagnostics are not a compatibility parser mode.

## Work

- [ ] Audit existing codes, lexer/parser recovery, semantic errors, project/build
  reporting, runner transport, and runtime source mapping; record exact paths.
- [ ] Extend the shared schema and renderers, including unavailable positions,
  expected/found details, source identity, and deterministic JSON serialization.
- [ ] Implement CLI selection and child-runner forwarding without losing exit
  status, duplicating diagnostics, or misattributing imported-unit errors.
- [ ] Add a diagnostics reference under `docs/pascal/tools/` when implemented,
  with schema, stream rules, code inventory, and wrong/corrected examples.
- [ ] Add representative tests through lexer, parser, sema, project/build,
  runtime, all four CLI commands, and the actual runner process.

New language diagnostics ship with their owning stage, using this foundation.
In particular, cover grouped/comma-separated formal parameters, missing named
closers, obsolete imports/member calls, unused values, and missing `var` markers.

## Acceptance

A consumer can identify the code, phase, cause, and available source range without
parsing prose. Text and JSON agree; Unicode locations, no-source errors, multiple
errors, mixed program output, and runner exit failures are covered. Existing
diagnostic identities do not change just to match a new numbering preference.

Owners: `fpas-diagnostics`, diagnostic producers in lexer/parser/sema/compiler,
`fpas-project`, `fpas-build`, `fpas-linker`, `fpas-cli`, and `fpas-vm`; editor
adapters consume the same model through `fpas-language-service` and `fpas-lsp`.

Status: shared records and codes exist; target stream/schema extensions are planned.
Next: trace one parser error and one runner error through text output, then add the
shared structured-output path and regression coverage.

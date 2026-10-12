# AP31.1: Persist evaluated constant defaults

Package: [AP31](README.md). Status: pending repair; not implemented.

## Scope and prerequisites

Reproduce literal success and expression/reference failure in independently
compiled units. Reuse the existing compile-time constant classification and
evaluation; do not invent a second evaluator in the interface serializer.
AP10 and AP24 are complete. No dependency on numeric redesign is required:
preserve currently defined constant evaluation in this repair.

## Affected areas

Inspect `crates/fpas-sema/src/interface/export.rs`,
`crates/fpas-sema/src/check/decl/types/records/`, the constant evaluator,
and `crates/fpas-unit/src/interface/`. Select the concrete evaluation and
diagnostic owners before editing.

## Migration and documentation

No source-syntax migration. Update the
[record default reference](../../../pascal/language/types/records.md#default-field-values)
only to clarify the existing boundary, and correct the diagnostic guidance.
Do not narrow its promised constant-expression support to hide the defect.

## Verification

- Compare `3`, `1 + 2`, a preceding scalar constant, parentheses, unary minus,
  and `None` across local and exported records.
- Check aliases, generic records where applicable, shadowed caller names,
  and fresh/reused sidecars with the same execution result.
- Unsupported dynamic defaults and invalid constants fail at the declaration.
- Run applicable workspace, project, FPAS, and documentation checks.

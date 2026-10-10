# AP15: Remove event declarations

Status: complete (AP15.1, AP15.2). Effort: small. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Result

Callbacks are ordinary record fields of type `Option of HandlerType`, with
`None` as their default. Store a callable with `Some(...)`, clear it with
`None`, and select it through an `is` test or a complete `case`. `Unwrap`
retains the ordinary failure behavior when absence is an error.

Ordinary field visibility, mutability, construction, updates, and record value
semantics apply. Assignment requires a mutable record. A public field permits
consumer reads, assignments, and calls. Record methods may return updated
record values for encapsulated changes. Compiled-unit interfaces preserve
`None` defaults and transparent aliases.

`event`, `nil`, `read`, `write`, and `Assigned` are ordinary identifiers.
Their declarations and references use normal case-insensitive resolution.
Records have fields and instance/static routines; event grammar, AST,
semantic metadata, accessor resolution, lowering, and editor event kinds
are absent. Invalid declarations and unresolved names use ordinary errors.
No migration diagnostics, replacement hints, or legacy event recognition
are added.

## Ownership and coverage

[AP15.1](01-migrate-events.md) owns optional-handler consumers and tests across
parser, semantic checking, compiler execution, compiled-unit interfaces,
CLI projects, formatter, and editor fixtures.
[AP15.2](02-remove-event-declarations.md) owns ordinary names, the current
field/method model, keyword and grammar parity, debugger parsing, and editor
classification.

## Dependencies

- AP20 (complete): existing pattern matching and `is` tests select handlers.

## Work packages

- [x] [AP15.1: Migrate events to optional handler fields](01-migrate-events.md)
- [x] [AP15.2: Remove event declarations](02-remove-event-declarations.md)

## Current documentation

- [Optional handlers](../../../pascal/language/functions/first-class.md#optional-handlers).
- [Keywords](../../../pascal/getting-started/keywords.md).
- [Records](../../../pascal/language/types/records.md).

## Independent follow-up

A standard multiple-subscriber API requires a concrete use case and its own
scope; it is not part of this package.

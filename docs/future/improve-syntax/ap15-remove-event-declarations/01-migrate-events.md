# AP15.1: Migrate events to optional handler fields

Package: [AP15: Remove event declarations](README.md)

## Scope

Replace every event declaration and use with an `Option of HandlerType` field,
set and cleared with methods or record updates and invoked through `case`.

## Prerequisites

None. A `case` over `Option` and callable record fields work in the current
language; verify before starting.

## Implementation

- Convert each event to an optional handler field with default `None`.
- Replace subscription with setting the field, `Assigned` checks with a `case`
  over the field, and invocation with a call of the bound handler.

## Affected areas

- `examples/pascal/record-methods/events.fpas` (planning-time inventory;
  recheck).
- The formatter golden file containing events (convert it and keep a separate
  negative test for AP15.2).

## Migration

This work package is the migration.

## Documentation

Documentation examples outside the event page.

## Verification

- No event use remains outside `record-events.md` and negative tests.
- The migrated example runs with unchanged output.

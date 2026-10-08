# AP15: Remove event declarations

Status: agreed direction. Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Existing event use cases are expressed with ordinary fields and `Option`, and
no event grammar remains.

Events are used only in one example and one formatter golden file at planning
time.

## Decisions

- `event`, the event-only `nil`, and `Assigned` are removed without replacement
  grammar. `read` and `write`, kept reserved by AP14 for event accessors, become
  identifiers.
- A handler is an ordinary field of type `Option of HandlerType`, set and
  cleared with record methods or record updates and invoked through a `case`
  or an `is` test.
- Multiple subscribers are not designed now; they need a concrete use case.

```pascal
type Button = record
  OnClick: Option of ClickHandler := None;
end record;

if B.OnClick is Some(const Handler) then
  Handler(B);
end if;
```

## Dependencies

- AP20 (the `is` test, AP20.3, used in the replacement and its diagnostic).

## Order

AP15.1 migrates the event uses with `case` on `Option`, which works today.
AP15.2 removes the grammar once the `is` test exists for the diagnostic hint.

## Work packages

- [ ] [AP15.1: Migrate events to optional handler fields](01-migrate-events.md)
- [ ] [AP15.2: Remove event declarations](02-remove-event-declarations.md)

## Acceptance

Existing event use cases are expressed with ordinary fields and `Option`, and
no event grammar remains.

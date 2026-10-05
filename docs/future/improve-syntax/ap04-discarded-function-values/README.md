# AP04: Discarded function values

Status: agreed direction. Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

A function result is never lost silently. Explicit discard is usable for
ordinary values and `Result`, is visible in source, and cannot drop a task
handle.

## Decisions

- A function result in statement position must be consumed. Procedures remain
  the ordinary standalone calls.
- `discard Expression;` is the explicit form for ignoring a function result.
  `discard` becomes a reserved keyword.
- `Result` values may be discarded only with `discard`; an unused `Result`
  without it is an error.
- Task handles cannot be discarded. `discard go Worker();` is an error whose
  diagnostic points to the existing statement `go Worker();`; task lifetime
  stays with `go` statements and AP26.
- `discard` applied to a procedure call (no value) is an error.

```pascal
Fs.Delete(TempPath);           // error: unused Result value
discard Fs.Delete(TempPath);   // valid: deliberately ignored
discard go Worker();           // error: use the statement 'go Worker();'
go Worker();                   // valid: fire-and-forget task
```

## Dependencies

- AP02 (diagnostic codes and hints).

## Order

AP04.1 adds `discard`; AP04.2 requires consumption and migrates callers.

## Work packages

- [ ] [AP04.1: Discard statement](01-discard-statement.md)
- [ ] [AP04.2: Require consumed function results](02-require-consumed-results.md)

## Acceptance

Explicit discard is usable for ordinary values and `Result`, is visible in
source, and cannot be used to drop a task handle.

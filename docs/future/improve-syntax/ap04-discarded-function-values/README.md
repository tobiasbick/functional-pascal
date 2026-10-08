# AP04: Discarded function values

Status: complete. Effort: small.
Completion is tracked in the
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
  `discard` is a reserved keyword.
- `Result` values may be discarded only with `discard`; an unused `Result`
  without it is an error.
- Task handles and aggregates containing them cannot be discarded.
  `discard go Worker();` is an error whose diagnostic points to the existing
  statement `go Worker();`; task lifetime stays with `go` statements and AP26.
- [Generic operands](01-discard-statement.md#generic-operands) require declared
  constraints proving that no permitted type argument can contain task
  handles. Under the current definitions, `Numeric` and `Comparable` suffice;
  unconstrained `T` and `Printable` do not. Concrete results of generic calls
  follow the ordinary discard rules. Use the existing constraints.
- [Channels](01-discard-statement.md#channels) use the same recursive check
  for their element type, regardless of queue contents or other references.
- [Callable values and closure captures](01-discard-statement.md#callable-values-and-closure-captures)
  may be discarded only when their stored captures are statically proven free
  of task handles. Unknown captures prevent discard; bound methods include
  their captured receiver. Capture information must survive assignments,
  returns, aggregate storage, and unit boundaries.
- `discard` applied to a procedure call (no value) is an error.

```pascal
Fs.Delete(TempPath);           // error: unused Result value
discard Fs.Delete(TempPath);   // valid: deliberately ignored
discard go Worker();           // error: use the statement 'go Worker();'
go Worker();                   // valid: fire-and-forget task
```

## Open decisions

None.

## Dependencies

- AP02 (diagnostic codes and hints).

## Work packages

- [x] [AP04.1: Discard statement](01-discard-statement.md)
- [x] [AP04.2: Require consumed function results](02-require-consumed-results.md)

## Acceptance

Explicit discard is usable for ordinary values and `Result`, is visible in
source, and cannot be used to drop a task handle.

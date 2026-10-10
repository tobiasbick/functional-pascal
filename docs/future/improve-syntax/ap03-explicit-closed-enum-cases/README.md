# AP03: Explicit closed-enum cases

Status: agreed direction. Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Adding a variant to a closed enum exposes every `case` that does not handle it.
Field wildcards do not silently bypass explicit top-level variant coverage, and
single-variant handling uses the `is` test.

## Decisions

- A `case` over a closed enum (`Option`, `Result`, user enums) lists every
  variant explicitly; `else` is not allowed there. Open domains such as
  `integer` or `string` keep `else`.
- `boolean` remains a scalar case domain: `else` stays allowed and AP03 does
  not introduce mandatory `true`/`false` coverage.
- The closed-enum `else` prohibition also applies when explicit arms already
  cover every variant; remove a redundant `else` instead of keeping it.
- A guarded arm alone does not cover its variant. Ignoring payload fields
  (`_`, AP20) is distinct from ignoring a whole variant.
- Code that handles a single variant uses the `is` test from AP20 instead of a
  full `case`. The no-`else` rule ships together with or after that test.
- Variants that need no action are listed in an arm containing `null;` (AP13).

```pascal
case Msg of
  when TuiMsg.Key(const K):
    HandleKey(K);
  when TuiMsg.Tick, TuiMsg.Focus:
    null;
  // ... every other variant
end case;

if Msg is TuiMsg.Key(const Key) then
  HandleKey(Key);
end if;
```

## Dependencies

- AP02 (diagnostic codes for the new error).
- AP20 (the `is` test, AP20.3). AP20 itself depends on AP13, which supplies
  `when` arms and `null;`.

AP21 depends on this package.

## Order

AP03.1 migrates existing sites while `else` is still accepted. AP03.2 enforces
the rule.

## Work packages

- [ ] [AP03.1: Migrate closed-enum catch-alls](01-migrate-closed-enum-catch-alls.md)
- [ ] [AP03.2: Reject catch-alls on closed enums](02-reject-closed-enum-catch-alls.md)

## Acceptance

Enum extension exposes omitted cases; field wildcards do not silently bypass
explicit top-level variant coverage; single-variant handling uses the `is` test.

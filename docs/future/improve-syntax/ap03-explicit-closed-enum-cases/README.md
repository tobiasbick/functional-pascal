# AP03: Explicit closed-enum cases

Status: complete (AP03.1, AP03.2). Effort: small. Completion is tracked in the
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
  full `case`.
- Variants that need no action are listed in an arm containing `null;` (AP13).

```pascal
type Message = enum
  Key(Code: integer);
  Resize;
  Quit;
end enum;

case Msg of
  when Message.Key(const Code):
    HandleKey(Code);
  when Message.Resize, Message.Quit:
    null;
end case;

if Msg is Message.Key(const Code) then
  HandleKey(Code);
end if;
```

## Dependencies

- AP02 (diagnostic codes for the new error).
- AP20 (the `is` test, AP20.3). AP20 itself depends on AP13, which supplies
  `when` arms and `null;`.

AP21 depends on this package.

## Work packages

- [x] [AP03.1: Migrate closed-enum catch-alls](01-migrate-closed-enum-catch-alls.md)
- [x] [AP03.2: Reject catch-alls on closed enums](02-reject-closed-enum-catch-alls.md)

## Implementation and verification

Semantic analysis rejects closed-enum `else` with FP3035 and reports missing
patterns using the existing coverage matrix. Cases without `else` retain
FP3011 for incomplete coverage. Repository consumers use explicit variants or
`is`, and regression tests cover enum extension, imported types and sidecars,
guards, nested patterns, redundant catch-alls, and scalar exceptions.

The current rules are documented in
[exhaustiveness](../../../pascal/language/pattern-matching/exhaustiveness.md),
[`case`](../../../pascal/language/control-flow/case-of-intro.md), and
[diagnostics](../../../pascal/tools/diagnostics.md).

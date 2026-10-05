# AP07: Boolean rules

Status: agreed direction (Q02, Q03). Effort: medium. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Evaluation order and precedence are unambiguous and covered by boundary tests;
mixed logical operators require parentheses; migration does not silently change
meaning.

Current behavior: `and` binds like `*` and `or`/`xor` like `+`, above the
comparisons, so `X > 0 and Y > 0` is parsed as `X > (0 and Y) > 0`
([operators](../../../pascal/language/basics/operators.md)).

## Decisions

- `and`, `or`, `xor`, and `not` are boolean-only. Comparisons bind more tightly
  than all of them.
- Different logical binary operators cannot be mixed without parentheses (as
  in Ada). A chain of the same operator is valid; `A and B or C` is an error
  whose diagnostic shows `(A and B) or C`.
- `not` binds below the comparisons and above the binary logical operators:
  `not Count > 0` means `not (Count > 0)`, and `not Done and Ready` means
  `(not Done) and Ready`.
- Bit operations become named functions in the new `Std.Bits` unit (Q02):
  `BitAnd`, `BitOr`, `BitXor`, `BitNot`, `ShiftLeft`, `ShiftRight`; `shl` and
  `shr` are no longer keywords. Integer operands of `and`, `or`, `xor`, or
  `not` are diagnosed with the matching function name.
- `and` and `or` evaluate left to right with short-circuit evaluation.

```pascal
if X > 0 and Y > 0 and Z > 0 then
  Accept();
end if;

if (A and B) or C then
  Retry();
end if;

const Mask: integer := BitAnd(Flags, ReadMask);
```

### Precedence table (Q03)

From strongest to weakest:

| Level | Operators | Notes |
|-------|-----------|-------|
| 1 | postfix: call, index, field, `with ... end with` | |
| 2 | unary `-`, `try` | |
| 3 | `*`, `/`, `div`, `mod` | |
| 4 | `+`, `-` | |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in`, `is` | non-associative |
| 6 | `not` | |
| 7 | `and`, `or`, `xor` | same-operator chains only |

Comparison chains such as `A < B < C` are errors. The diagnostic suggests
`A < B and B < C`. Mixed logical operators require parentheses; `not A = B`
means `not (A = B)`.

## Dependencies

- AP02 (diagnostic codes).

AP18, AP21, and AP23 depend on this package.

## Order

AP07.1 and AP07.2 are independent. AP07.3 changes precedence and may run in
parallel with AP07.2. AP07.4 needs `Std.Bits` from AP07.2.

## Work packages

- [ ] [AP07.1: Short-circuit evaluation](01-short-circuit-evaluation.md)
- [ ] [AP07.2: Std.Bits unit](02-std-bits-unit.md)
- [ ] [AP07.3: Logical precedence and mixing rules](03-logical-precedence.md)
- [ ] [AP07.4: Boolean-only logical operators](04-boolean-only-operators.md)

## Acceptance

Evaluation order and precedence are unambiguous and covered by boundary tests;
mixed logical operators require parentheses; migration does not silently change
meaning.

## Reference

The reference branch `codex/syntax-changes` found that the compiler lowers
`and`, `or`, and `xor` eagerly (both operands) and that the parser already
rejects chained comparisons. It added branch-based lowering in a new
`fpas-compiler/src/lowering/expr/boolean.rs` and found no repository source
that needed regrouping. Recheck both findings on `main`.

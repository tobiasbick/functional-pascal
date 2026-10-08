# AP07: Boolean rules

Status: complete (Q02, Q03). Effort: medium. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Evaluation order and precedence are unambiguous and covered by boundary tests;
mixed logical operators require parentheses; migration does not silently change
meaning.

Current behavior: comparisons bind above `not`, and `not` above logical
chains. Same-operator chains are valid; mixed logical operators require
parentheses. Logical operators accept only boolean operands; `and` and `or`
short-circuit and `xor` evaluates both operands. Integer bit operations use
`Std.Bits` functions, and `shl`/`shr` are ordinary identifiers
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
- Integer bit operations use named functions in `Std.Bits` (Q02):
  `BitAnd`, `BitOr`, `BitXor`, `BitNot`, `ShiftLeft`, `ShiftRight`; `shl` and
  `shr` are no longer keywords. Integer operands of `and`, `or`, `xor`, or
  `not` are diagnosed with the matching function name.
- Shift functions preserve the signed 64-bit shift semantics: counts `0..63`,
  runtime errors outside that range, arithmetic right shift retaining the
  sign, and left shift discarding high bits without an overflow error. The
  exact rules and examples are agreed in
  [AP07.2](02-std-bits-unit.md#shift-decisions-agreed).
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
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` | non-associative |
| 6 | `not` | |
| 7 | `and`, `or`, `xor` | same-operator chains only |

The `is` pattern test (AP20.3) shares the comparison precedence.

Comparison chains such as `A < B < C` are errors. The diagnostic suggests
`A < B and B < C`. Mixed logical operators require parentheses; `not A = B`
means `not (A = B)`.

## Dependencies

- AP02 (diagnostic codes).

AP18, AP21, and AP23 depend on this package.

## Work packages

- [x] [AP07.1: Short-circuit evaluation](01-short-circuit-evaluation.md)
- [x] [AP07.2: Std.Bits unit](02-std-bits-unit.md)
- [x] [AP07.3: Logical precedence and mixing rules](03-logical-precedence.md)
- [x] [AP07.4: Boolean-only logical operators](04-boolean-only-operators.md)

## Acceptance

Evaluation order and precedence are unambiguous and covered by boundary tests;
mixed logical operators require parentheses; migration does not silently change
meaning.

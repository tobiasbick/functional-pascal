# AP33.1: Numeric decisions

Package: [AP33](README.md). Status: complete; decision work package.

[AP43](../ap43-consistent-error-handling/README.md) supersedes this package's
retained parser names and aborting-wrapper boundary. Current Std.Parse
Int/Real/Bool/HexInt return Result; explicit Unwrap represents deliberate abort.
The numeric grammar, range, and arithmetic decisions below remain authoritative.

## Selected decisions

The user approved the proposed rules for the existing integer and real types.
AP07/AP18/AP19 remain the baseline for Boolean, subrange, distinct, and bit rules.

| Case | Required behavior |
| --- | --- |
| `1 + 0.5`, `1 = 1.0`, `1 < 1.5` | Type error; convert explicitly, for example `IntToReal(1) + 0.5`. |
| Assign/pass integer to real | Type error; use `IntToReal`. The same rule applies to returns, aggregate fields, and generic inference. |
| Arithmetic on `T: Numeric` | Operands must have the same declared numeric type parameter. `+`, `-`, and `*` retain T; `/` returns real. Different type parameters and T/concrete numeric mixtures require explicit conversion. |
| `1 / 2` | Real value 0.5; `/` is real division and does not infer integer division. |
| Integer overflow in `+`, `-`, `*`, unary `-` | Numeric-domain error; constant evaluation must not persist a wrapped value. |
| `-9223372036854775808`, `-$8000000000000000` | Valid minimum integer literals. The corresponding positive magnitude is invalid as a value. |
| `-7 div 3`, `-7 mod 3`, `7 mod -3` | -2, -1, and 1 respectively. |
| Integer minimum `div -1` / `mod -1` | Division overflows; modulo returns zero. Both operations reject a zero divisor. |
| NaN/infinity in real computation | Numeric-domain error, including numeric Std operations; literals/text reject those values. |
| Real division by +0.0 or -0.0 | Division-by-zero error. |
| Real arithmetic underflow | Ordinary binary64 rounding, including subnormal values and signed zero. |
| Nonzero real literal/text rounded to zero | Out-of-range error. Representable subnormals remain valid. |
| `IntToReal` | Explicit binary64 conversion with rounding and possible precision loss; not an exact-conversion promise. |
| `Round(2.5)`, `Round(3.5)`, `Round(-2.5)` | 2, 4, -2; ties to even. `Trunc` remains truncation toward zero. |
| Real-to-integer bounds | Reject nonfinite or out-of-range results; no saturation or wrapping. |
| Numeric source and text forms | Shared decimal/hex bodies and `_` separators. Real bodies require a fractional part, with optional exponent. Input text wraps the same body with optional sign and surrounding whitespace. |
| `TryInt('$FF')` / `TryInt('-$8000000000000000')` | 255 / minimum integer. Hex denotes a mathematical magnitude, not reinterpretation of an unsigned bit pattern. |
| Finite `RealToStr` then `TryReal` | Same binary64 bits; integral real text contains `.0`, and -0.0 retains its sign. |
| Compile-time/runtime/debugger | Same type, bound, rounding, and exceptional-value rules; errors remain on the evaluated path. |

## Delivery boundary

This work package records the implemented decision. AP33.2 owns arithmetic and
literals; AP33.3 owns number text. Existing public numeric
API names and type identities remain selected; additional fixed-width or
decimal types are outside this package.

## Documentation and verification

The [operator](../../../pascal/language/basics/operators.md),
[literal](../../../pascal/language/basics/number-literals.md), and text
references describe the selected behavior. The table
defines the counterexamples, boundaries, and parity required for acceptance.

## Open decisions

None within this work package's scope.

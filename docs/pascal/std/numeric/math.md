# `Std.Math`

Elementary math: constant `Pi`, roots, powers, trig, rounding, log, and polymorphic `Abs` / `Min` / `Max`. This page is the **full API** for the unit.

Real operands and results must be finite. Operations that would produce NaN
or infinity, such as `Pow(-1.0, 0.5)` or `Exp(1000.0)`, report a numeric-domain
error. Ordinary binary64 rounding, subnormals, and signed zero remain valid.
Rounding operations also check that their integer result fits the signed 64-bit
range. Integer arguments are not implicitly converted for a real parameter;
use `Std.Conv.IntToReal` explicitly.

```pascal
program Example;

uses Std.Console as Console, Std.Math as Math;

begin
  Console.WriteLn(Math.Sqrt(16.0));
end.
```


## Importing and names

After `uses Std.Math as Math;`, use `Math.Pi`, `Math.Sqrt`, and the other
exported symbols through that alias.

Collisions with your own identifiers are resolved like ordinary scope rules: a **local** name (e.g. a variable `Pi`) **hides** the short import; use `Std.Math.Pi` if you need the library constant anyway.

---

## Quick reference

Requires `uses Std.Math as Math;`.

| Kind | Name | Notes |
|------|------|--------|
| constant | `Pi: real` | compiler-inserted value |
| function | `Sqrt(R: real): real` | error if `R < 0` |
| function | `Pow(Base: real; Exp: real): real` | power |
| function | `Floor(R: real): integer` | toward −∞ |
| function | `Ceil(R: real): integer` | toward +∞ |
| function | `Round(R: real): integer` | nearest integer, ties to even; checked signed 64-bit result |
| function | `Sin(R: real): real` | radians |
| function | `Cos(R: real): real` | radians |
| function | `Log(R: real): real` | natural log; error if `R ≤ 0` |
| function | `Abs(N)` | `integer` or `real` — result matches `N`; also `Abs(N := -3)` |
| function | `Min(A; B)` | both `integer` or both `real` |
| function | `Max(A; B)` | both `integer` or both `real` |
| function | `Tan(R: real): real` | tangent (radians) |
| function | `ArcSin(R: real): real` | inverse sine |
| function | `ArcCos(R: real): real` | inverse cosine |
| function | `ArcTan(R: real): real` | inverse tangent |
| function | `ArcTan2(Y: real; X: real): real` | two-argument arctangent |
| function | `Exp(R: real): real` | e^R |
| function | `Log10(R: real): real` | base-10 logarithm |
| function | `Log2(R: real): real` | base-2 logarithm |
| function | `Trunc(R: real): integer` | truncate toward zero |
| function | `Frac(R: real): real` | fractional part |
| function | `Sign(N)` | `-1`, `0`, or `1` (`integer` or `real` input) |
| function | `Clamp(V; Lo; Hi)` | restrict to range (`integer` or `real`) |

---

`Floor`, `Ceil`, `Round`, and `Trunc` report a runtime error when the rounded
result is non-finite or outside the signed 64-bit integer range. In particular,
`9223372036854775808.0` (2^63) is outside that range; -2^63 is valid.

## Constant `Pi`

- **Type:** `real`
- **Value:** the mathematical constant π.
- **Note:** provided via compile-time lowering for `Std.Math.Pi` (and short `Pi` when imported).

```pascal
const R: real := Math.Pi;
Console.WriteLn(Math.Round(R));
```

---

## `function Sqrt(R: real): real`

Square root. **Runtime error** if `R` is negative.

```pascal
Console.WriteLn(Math.Sqrt(16.0));
```

---

## `function Pow(Base: real; Exp: real): real`

Raises `Base` to `Exp`.

```pascal
Console.WriteLn(Math.Pow(2.0, 3.0));
```

---

## `function Floor(R: real): integer`

Greatest integer ≤ `R`.

```pascal
Console.WriteLn(Math.Floor(2.9));
```

---

## `function Ceil(R: real): integer`

Smallest integer ≥ `R`.

```pascal
Console.WriteLn(Math.Ceil(2.1));
```

---

## `function Round(R: real): integer`

Nearest integer, with exact half values rounded to the even integer:
`Round(2.5)` is 2, `Round(3.5)` is 4, and `Round(-2.5)` is -2.
The result must fit the signed 64-bit range. `Trunc` remains truncation toward zero.

```pascal
Console.WriteLn(Math.Round(Math.Pi));
```

---

## `function Sin(R: real): real` / `function Cos(R: real): real`

Trigonometric functions; angle in **radians**.

```pascal
Console.WriteLn(Math.Sin(0.0));
Console.WriteLn(Math.Cos(0.0));
```

---

## `function Log(R: real): real`

Natural logarithm. **Runtime error** if `R ≤ 0`.

```pascal
Console.WriteLn(Math.Log(2.718281828459045));
```

---

## `function Abs(N)` (integer or real)

Absolute value. `N` may be `integer` or `real`; the result has the **same** kind.

```pascal
Console.WriteLn(Math.Abs(-7));
Console.WriteLn(Math.Abs(-1.5));
```

---

## `function Min(A; B)` / `function Max(A; B)` (integer or real)

`A` and `B` must be the **same** numeric kind. Returns the smaller or larger.

```pascal
Console.WriteLn(Math.Min(3, 9));
Console.WriteLn(Math.Max(3, 9));
```

---

## `function Tan(R: real): real`

Tangent of `R` (radians).

```pascal
Console.WriteLn(Math.Tan(0.0));
```

---

## `function ArcSin(R: real): real`

Inverse sine (arc sine). **Runtime error** if `R` is outside `[-1, 1]`.

```pascal
Console.WriteLn(Math.ArcSin(1.0));  // Pi/2
```

---

## `function ArcCos(R: real): real`

Inverse cosine. **Runtime error** if `R` is outside `[-1, 1]`.

```pascal
Console.WriteLn(Math.ArcCos(1.0));  // 0.0
```

---

## `function ArcTan(R: real): real`

Inverse tangent (classic Pascal `ArcTan`).

```pascal
Console.WriteLn(Math.ArcTan(1.0));  // Pi/4
```

---

## `function ArcTan2(Y: real; X: real): real`

Two-argument arctangent — angle of the vector `(X, Y)` in the correct quadrant.
For finite inputs the result is in `[-Pi, Pi]`. Signed zero selects the boundary
on the negative X-axis: `ArcTan2(0.0, -1.0)` is `Pi`, while
`ArcTan2(-0.0, -1.0)` is `-Pi`.

```pascal
Console.WriteLn(Math.ArcTan2(1.0, 1.0));  // Pi/4
```

---

## `function Exp(R: real): real`

Returns e^R. Inverse of `Log`.

```pascal
Console.WriteLn(Math.Exp(1.0));  // ~2.718
```

---

## `function Log10(R: real): real`

Base-10 logarithm. **Runtime error** if `R ≤ 0`.

```pascal
Console.WriteLn(Math.Log10(100.0));  // 2.0
```

---

## `function Log2(R: real): real`

Base-2 logarithm. **Runtime error** if `R ≤ 0`.

```pascal
Console.WriteLn(Math.Log2(8.0));  // 3.0
```

---

## `function Trunc(R: real): integer`

Truncates toward zero (classic Pascal `Trunc`). Unlike `Floor`, `Trunc(-3.7)` yields `-3`, not `-4`.

```pascal
Console.WriteLn(Math.Trunc(3.9));    // 3
Console.WriteLn(Math.Trunc(-3.7));    // -3
```

---

## `function Frac(R: real): real`

Fractional part: `Frac(R) = R - Trunc(R)`.

```pascal
Console.WriteLn(Math.Frac(3.14));   // 0.14
Console.WriteLn(Math.Frac(-3.14));  // -0.14
```

---

## `function Sign(N)` (integer or real)

Returns `-1`, `0`, or `1` depending on the sign of `N`. `N` may be `integer` or `real`; result is always `integer`.

```pascal
Console.WriteLn(Math.Sign(-42));   // -1
Console.WriteLn(Math.Sign(0));     // 0
Console.WriteLn(Math.Sign(3.14));   // 1
```

---

## `function Clamp(V; Lo; Hi)` (integer or real)

Fixed-arity polymorphic operations accept their published labels, just like
ordinary routines: `Clamp(Hi := 10, V := 20, Lo := 0)`. Every argument is
positional or every argument is named; values evaluate once in written order.
Polymorphic operations are called directly. Use a typed wrapper to pass one
as a callable value, for example `function(N: integer): integer begin return Abs(N); end function`.

Returns `V` constrained to `[Lo, Hi]`. All three arguments must be the same numeric kind. Result matches the input kind.

```pascal
Console.WriteLn(Math.Clamp(150, 0, 100));     // 100
Console.WriteLn(Math.Clamp(-5, 0, 100));      // 0
Console.WriteLn(Math.Clamp(1.5, 0.0, 1.0));   // 1.0
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime intrinsics | [`math/mod.rs`](../../../../crates/fpas-std/src/math/mod.rs), shared [`numeric bounds`](../../../../crates/fpas-numeric/src/real.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Numeric index](README.md)
- [Standard library index](../README.md)

# `Std.Math`

Elementary math: integer-to-real conversion, constant `Pi`, roots, powers, trig,
rounding, log, and polymorphic `Abs` / `Min` / `Max`. This page is the **full API**
for the unit.

Real operands and results must be finite. Operations that would produce NaN
or infinity, such as `Pow(-1.0, 0.5)` or `Exp(1000.0)`, report a numeric-domain
error. Ordinary binary64 rounding, subnormals, and signed zero remain valid.
Rounding operations also check that their integer result fits the signed 64-bit
range. Integer arguments are not implicitly converted for a real parameter;
use `Std.Math.IntToReal` explicitly.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Math as Math;

begin
  Console.WriteLn(Math.Sqrt(16.0));
end.
```


## Importing and names

After `uses Std.Math as Math;`, use `Math.Pi`, `Math.Sqrt`, and the other
exported symbols through that alias.

A local binding named `Pi` is independent of `Math.Pi`; imports expose symbols only through their
alias. The alias itself cannot be shadowed. Use `Math.Pi` for the library constant.

---

## Quick reference

Requires `uses Std.Math as Math;`.

| Kind | Name | Notes |
|------|------|--------|
| constant | `Pi: real` | compiler-inserted value |
| function | `IntToReal(N: integer): real` | explicit binary64 rounding; may lose integer precision |
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

## `function IntToReal(N: integer): real`

Converts integer to binary64 using rounding to nearest, with ties to even.
All integer values fit the real value range, but conversion may lose precision.
Every integer between `-9007199254740992` and `9007199254740992` is exact;
larger values are exact only when representable at binary64's spacing.
This is the explicit conversion used before mixing integer and real calculations.

```pascal id=integer-to-real fragment context=body-console-math
const X: real := Math.IntToReal(3);
Console.WriteLn(X);
```

## Constant `Pi`

- **Type:** `real`
- **Value:** the mathematical constant π.
- **Note:** provided via compile-time lowering for `Math.Pi` after `uses Std.Math as Math;`.

```pascal id=example-2 fragment context=body-console-math
const R: real := Math.Pi;
Console.WriteLn(Math.Round(R));
```

---

## `function Sqrt(R: real): real`

Square root. **Runtime error** if `R` is negative.

```pascal id=example-3 fragment context=body-console-math
Console.WriteLn(Math.Sqrt(16.0));
```

---

## `function Pow(Base: real; Exp: real): real`

Raises `Base` to `Exp`.

```pascal id=example-4 fragment context=body-console-math
Console.WriteLn(Math.Pow(2.0, 3.0));
```

---

## `function Floor(R: real): integer`

Greatest integer ≤ `R`.

```pascal id=example-5 fragment context=body-console-math
Console.WriteLn(Math.Floor(2.9));
```

---

## `function Ceil(R: real): integer`

Smallest integer ≥ `R`.

```pascal id=example-6 fragment context=body-console-math
Console.WriteLn(Math.Ceil(2.1));
```

---

## `function Round(R: real): integer`

Nearest integer, with exact half values rounded to the even integer:
`Round(2.5)` is 2, `Round(3.5)` is 4, and `Round(-2.5)` is -2.
The result must fit the signed 64-bit range. `Trunc` remains truncation toward zero.

```pascal id=example-7 fragment context=body-console-math
Console.WriteLn(Math.Round(Math.Pi));
```

---

## `function Sin(R: real): real` / `function Cos(R: real): real`

Trigonometric functions; angle in **radians**.

```pascal id=example-8 fragment context=body-console-math
Console.WriteLn(Math.Sin(0.0));
Console.WriteLn(Math.Cos(0.0));
```

---

## `function Log(R: real): real`

Natural logarithm. **Runtime error** if `R ≤ 0`.

```pascal id=example-9 fragment context=body-console-math
Console.WriteLn(Math.Log(2.718281828459045));
```

---

## `function Abs(N)` (integer or real)

Absolute value. `N` may be `integer` or `real`; the result has the **same** kind.

```pascal id=example-10 fragment context=body-console-math
Console.WriteLn(Math.Abs(-7));
Console.WriteLn(Math.Abs(-1.5));
```

---

## `function Min(A; B)` / `function Max(A; B)` (integer or real)

`A` and `B` must be the **same** numeric kind. Returns the smaller or larger.

```pascal id=example-11 fragment context=body-console-math
Console.WriteLn(Math.Min(3, 9));
Console.WriteLn(Math.Max(3, 9));
```

---

## `function Tan(R: real): real`

Tangent of `R` (radians).

```pascal id=example-12 fragment context=body-console-math
Console.WriteLn(Math.Tan(0.0));
```

---

## `function ArcSin(R: real): real`

Inverse sine (arc sine). **Runtime error** if `R` is outside `[-1, 1]`.

```pascal id=example-13 fragment context=body-console-math
Console.WriteLn(Math.ArcSin(1.0)); // Pi/2
```

---

## `function ArcCos(R: real): real`

Inverse cosine. **Runtime error** if `R` is outside `[-1, 1]`.

```pascal id=example-14 fragment context=body-console-math
Console.WriteLn(Math.ArcCos(1.0)); // 0.0
```

---

## `function ArcTan(R: real): real`

Inverse tangent (classic Pascal `ArcTan`).

```pascal id=example-15 fragment context=body-console-math
Console.WriteLn(Math.ArcTan(1.0)); // Pi/4
```

---

## `function ArcTan2(Y: real; X: real): real`

Two-argument arctangent — angle of the vector `(X, Y)` in the correct quadrant.
For finite inputs the result is in `[-Pi, Pi]`. Signed zero selects the boundary
on the negative X-axis: `ArcTan2(0.0, -1.0)` is `Pi`, while
`ArcTan2(-0.0, -1.0)` is `-Pi`.

```pascal id=example-16 fragment context=body-console-math
Console.WriteLn(Math.ArcTan2(1.0, 1.0)); // Pi/4
```

---

## `function Exp(R: real): real`

Returns e^R. Inverse of `Log`.

```pascal id=example-17 fragment context=body-console-math
Console.WriteLn(Math.Exp(1.0)); // ~2.718
```

---

## `function Log10(R: real): real`

Base-10 logarithm. **Runtime error** if `R ≤ 0`.

```pascal id=example-18 fragment context=body-console-math
Console.WriteLn(Math.Log10(100.0)); // 2.0
```

---

## `function Log2(R: real): real`

Base-2 logarithm. **Runtime error** if `R ≤ 0`.

```pascal id=example-19 fragment context=body-console-math
Console.WriteLn(Math.Log2(8.0)); // 3.0
```

---

## `function Trunc(R: real): integer`

Truncates toward zero (classic Pascal `Trunc`). Unlike `Floor`, `Trunc(-3.7)` yields `-3`, not `-4`.

```pascal id=example-20 fragment context=body-console-math
Console.WriteLn(Math.Trunc(3.9)); // 3
Console.WriteLn(Math.Trunc(-3.7)); // -3
```

---

## `function Frac(R: real): real`

Fractional part: `Frac(R) = R - Trunc(R)`.

```pascal id=example-21 fragment context=body-console-math
Console.WriteLn(Math.Frac(3.14)); // 0.14
Console.WriteLn(Math.Frac(-3.14)); // -0.14
```

---

## `function Sign(N)` (integer or real)

Returns `-1`, `0`, or `1` depending on the sign of `N`. `N` may be `integer` or `real`; result is always `integer`.

```pascal id=example-22 fragment context=body-console-math
Console.WriteLn(Math.Sign(-42)); // -1
Console.WriteLn(Math.Sign(0)); // 0
Console.WriteLn(Math.Sign(3.14)); // 1
```

---

## `function Clamp(V; Lo; Hi)` (integer or real)

Fixed-arity polymorphic operations accept their published labels, just like
ordinary routines: `Clamp(Hi := 10, V := 20, Lo := 0)`. Every argument is
positional or every argument is named; values evaluate once in written order.
Polymorphic operations are called directly. Use a typed wrapper to pass one
as a callable value, for example `function(N: integer): integer begin return Abs(N); end function`.

Returns `V` constrained to `[Lo, Hi]`. All three arguments must be the same numeric kind. Result matches the input kind.

```pascal id=example-23 fragment context=body-console-math
Console.WriteLn(Math.Clamp(150, 0, 100)); // 100
Console.WriteLn(Math.Clamp(-5, 0, 100)); // 0
Console.WriteLn(Math.Clamp(1.5, 0.0, 1.0)); // 1.0
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime intrinsics | [`math/mod.rs`](../../../../crates/fpas-runtime/src/stdlib/math/mod.rs), shared [`numeric bounds`](../../../../crates/fpas-numeric/src/real.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Numeric index](README.md)
- [Standard library index](../README.md)

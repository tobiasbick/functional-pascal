# `Std.Math`

Elementary math: constant `Pi`, roots, powers, trig, rounding, log, and polymorphic `Abs` / `Min` / `Max`. This page is the **full API** for the unit.

```pascal
program Example;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(16.0))
end.
```

**Maintenance (implementers only):** align with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`math.rs`](../../../crates/fpas-std/src/math.rs), [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs) (`Pi` and call lowering), [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), and [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs).

---

## Importing and names

After `uses Std.Math;` use **`Pi`**, **`Sqrt`**, … or **`Std.Math.Pi`**, **`Std.Math.Sqrt`**, ….

Collisions with your own identifiers are resolved like ordinary scope rules: a **local** name (e.g. a variable `Pi`) **hides** the short import; use `Std.Math.Pi` if you need the library constant anyway.

---

## Quick reference

Requires `uses Std.Math;`.

| Kind | Name | Notes |
|------|------|--------|
| constant | `Pi: real` | compiler-inserted value |
| function | `Sqrt(R: real): real` | error if `R < 0` |
| function | `Pow(Base: real; Exp: real): real` | power |
| function | `Floor(R: real): integer` | toward −∞ |
| function | `Ceil(R: real): integer` | toward +∞ |
| function | `Round(R: real): integer` | nearest |
| function | `Sin(R: real): real` | radians |
| function | `Cos(R: real): real` | radians |
| function | `Log(R: real): real` | natural log; error if `R ≤ 0` |
| function | `Abs(N)` | `integer` or `real` — result matches `N` |
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
| function | `Random(): real` | pseudo-random `[0.0, 1.0)` |
| function | `RandomInt(Lo: integer; Hi: integer): integer` | random in `[Lo, Hi]` |
| procedure | `Randomize()` | seed RNG from system entropy |

---

## Constant `Pi`

- **Type:** `real`
- **Value:** the mathematical constant π.
- **Note:** provided via compile-time lowering for `Std.Math.Pi` (and short `Pi` when imported).

```pascal
var R: real := Pi;
WriteLn(Round(R))
```

---

## `function Sqrt(R: real): real`

Square root. **Runtime error** if `R` is negative.

```pascal
WriteLn(Sqrt(16.0))
```

---

## `function Pow(Base: real; Exp: real): real`

Raises `Base` to `Exp`.

```pascal
WriteLn(Pow(2.0, 3.0))
```

---

## `function Floor(R: real): integer`

Greatest integer ≤ `R`.

```pascal
WriteLn(Floor(2.9))
```

---

## `function Ceil(R: real): integer`

Smallest integer ≥ `R`.

```pascal
WriteLn(Ceil(2.1))
```

---

## `function Round(R: real): integer`

Nearest integer (implementation-defined tie-breaking for half values follows the runtime).

```pascal
WriteLn(Round(Pi))
```

---

## `function Sin(R: real): real` / `function Cos(R: real): real`

Trigonometric functions; angle in **radians**.

```pascal
WriteLn(Sin(0.0));
WriteLn(Cos(0.0))
```

---

## `function Log(R: real): real`

Natural logarithm. **Runtime error** if `R ≤ 0`.

```pascal
WriteLn(Log(2.718281828459045))
```

---

## `function Abs(N)` (integer or real)

Absolute value. `N` may be `integer` or `real`; the result has the **same** kind.

```pascal
WriteLn(Abs(-7));
WriteLn(Abs(-1.5))
```

---

## `function Min(A; B)` / `function Max(A; B)` (integer or real)

`A` and `B` must be the **same** numeric kind. Returns the smaller or larger.

```pascal
WriteLn(Min(3, 9));
WriteLn(Max(3, 9))
```

---

## `function Tan(R: real): real`

Tangent of `R` (radians).

```pascal
WriteLn(Tan(0.0))
```

---

## `function ArcSin(R: real): real`

Inverse sine (arc sine). **Runtime error** if `R` is outside `[-1, 1]`.

```pascal
WriteLn(ArcSin(1.0))  { Pi/2 }
```

---

## `function ArcCos(R: real): real`

Inverse cosine. **Runtime error** if `R` is outside `[-1, 1]`.

```pascal
WriteLn(ArcCos(1.0))  { 0.0 }
```

---

## `function ArcTan(R: real): real`

Inverse tangent (classic Pascal `ArcTan`).

```pascal
WriteLn(ArcTan(1.0))  { Pi/4 }
```

---

## `function ArcTan2(Y: real; X: real): real`

Two-argument arctangent — angle of the vector `(X, Y)` in the correct quadrant. Result in `(-Pi, Pi]`.

```pascal
WriteLn(ArcTan2(1.0, 1.0))  { Pi/4 }
```

---

## `function Exp(R: real): real`

Returns e^R. Inverse of `Log`.

```pascal
WriteLn(Exp(1.0))  { ~2.718 }
```

---

## `function Log10(R: real): real`

Base-10 logarithm. **Runtime error** if `R ≤ 0`.

```pascal
WriteLn(Log10(100.0))  { 2.0 }
```

---

## `function Log2(R: real): real`

Base-2 logarithm. **Runtime error** if `R ≤ 0`.

```pascal
WriteLn(Log2(8.0))  { 3.0 }
```

---

## `function Trunc(R: real): integer`

Truncates toward zero (classic Pascal `Trunc`). Unlike `Floor`, `Trunc(-3.7)` yields `-3`, not `-4`.

```pascal
WriteLn(Trunc(3.9));    { 3 }
WriteLn(Trunc(-3.7))    { -3 }
```

---

## `function Frac(R: real): real`

Fractional part: `Frac(R) = R - Trunc(R)`.

```pascal
WriteLn(Frac(3.14))   { 0.14 }
WriteLn(Frac(-3.14))  { -0.14 }
```

---

## `function Sign(N)` (integer or real)

Returns `-1`, `0`, or `1` depending on the sign of `N`. `N` may be `integer` or `real`; result is always `integer`.

```pascal
WriteLn(Sign(-42));   { -1 }
WriteLn(Sign(0));     { 0 }
WriteLn(Sign(3.14))   { 1 }
```

---

## `function Clamp(V; Lo; Hi)` (integer or real)

Returns `V` constrained to `[Lo, Hi]`. All three arguments must be the same numeric kind. Result matches the input kind.

```pascal
WriteLn(Clamp(150, 0, 100));     { 100 }
WriteLn(Clamp(-5, 0, 100));      { 0 }
WriteLn(Clamp(1.5, 0.0, 1.0))   { 1.0 }
```

---

## `function Random(): real`

Returns a pseudo-random real number in `[0.0, 1.0)`.

```pascal
Randomize();
var R: real := Random();
WriteLn(R)
```

---

## `function RandomInt(Lo: integer; Hi: integer): integer`

Returns a pseudo-random integer in `[Lo, Hi]` inclusive.

```pascal
Randomize();
var Die: integer := RandomInt(1, 6);
WriteLn(Die)
```

---

## `procedure Randomize()`

Seeds the random number generator from system entropy. Call once at program start for non-deterministic results.

```pascal
Randomize();
WriteLn(Random())
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Runtime intrinsics | [`math.rs`](../../../crates/fpas-std/src/math.rs) |
| `Pi` lowering | [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)

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

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Runtime intrinsics | [`math.rs`](../../../crates/fpas-std/src/math.rs) |
| `Pi` lowering | [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)

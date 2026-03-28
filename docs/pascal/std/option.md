# `Std.Option`

Helper functions for `Option of T` values. See [07-error-handling.md](../07-error-handling.md) for the type itself, constructors (`Some`, `None`), the `try` operator, and `case` destructuring.

```pascal
program Example;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(7);
  WriteLn(Unwrap(O))
end.
```

**Maintenance (implementers only):** align with [`std_registry/loaded/result_option.rs`](../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs), [`std_registry/builtins/result_option.rs`](../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs), [`result_option.rs`](../../../crates/fpas-std/src/result_option.rs), [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), and [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs).

---

## Importing and names

After `uses Std.Option;` use short names (`Unwrap`, `IsSome`, …) or qualified (`Std.Option.Unwrap`, …).

**Ambiguity with `Std.Result`:** the short names **`Unwrap`** and **`UnwrapOr`** clash with `Std.Result`. When both units are imported, qualify as `Std.Option.Unwrap(O)` vs `Std.Result.Unwrap(R)`.

---

## Quick reference

| Kind | Name | Notes |
|------|------|--------|
| function | `Unwrap(O: Option of T): T` | panics if None |
| function | `UnwrapOr(O: Option of T; Default: T): T` | returns Default if None |
| function | `IsSome(O: Option of T): boolean` | true if Some |
| function | `IsNone(O: Option of T): boolean` | true if None |

---

## `function Unwrap(O: Option of T): T`

Extracts the value from `Some(value)`. **Runtime error** if `O` is `None`.

```pascal
var O: Option of integer := Some(7);
WriteLn(Unwrap(O))                             { 7 }
```

---

## `function UnwrapOr(O: Option of T; Default: T): T`

Extracts the value from `Some(value)`, or returns `Default` if `O` is `None`.

```pascal
var O: Option of integer := None;
WriteLn(UnwrapOr(O, -1))                      { -1 }
```

---

## `function IsSome(O: Option of T): boolean`

Returns `true` if `O` is a `Some` variant.

```pascal
var O: Option of integer := Some(7);
WriteLn(IsSome(O))                             { true }
```

---

## `function IsNone(O: Option of T): boolean`

Returns `true` if `O` is `None`.

```pascal
var O: Option of integer := None;
WriteLn(IsNone(O))                             { true }
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/loaded/result_option.rs`](../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs) |

[← Standard library index](README.md)

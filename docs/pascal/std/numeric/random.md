# `Std.Random`

Pseudo-random number helpers. This page is the **full API** for the unit.

```pascal
program Example;
uses Std.Console, Std.Random;
begin
  Randomize();
  WriteLn(RandomInt(1, 6))
end.
```


## Importing and names

After `uses Std.Random;` use **`Random`**, **`RandomInt`**, **`Randomize`**, or the fully qualified forms **`Std.Random.Random`**, **`Std.Random.RandomInt`**, **`Std.Random.Randomize`**.

---

## Quick reference

Requires `uses Std.Random;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `Random(): real` | pseudo-random `[0.0, 1.0)` |
| function | `RandomInt(Lo: integer; Hi: integer): integer` | random in `[Lo, Hi]` |
| procedure | `Randomize()` | seed initializer |

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

Runtime error if `Lo > Hi`.

```pascal
Randomize();
var Die: integer := RandomInt(1, 6);
WriteLn(Die)
```

---

## `procedure Randomize()`

Initializes the random number generator. The current hosted runtime uses an automatically seeded thread-local generator, so this procedure is accepted for Pascal-style programs and has no observable result value.

```pascal
Randomize();
WriteLn(Random())
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime intrinsics | [`random.rs`](../../../../crates/fpas-std/src/random.rs) |
| Call lowering | [`std_calls/random.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/random.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Numeric index](README.md)
- [Standard library index](../README.md)

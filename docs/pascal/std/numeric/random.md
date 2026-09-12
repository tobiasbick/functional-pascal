# `Std.Random`

Deterministic pseudo-random number helpers for simulations, games, randomized tests, and similar non-security work. This page is the **full API** for the unit.

```pascal
program Example;
uses Std.Console, Std.Random;
begin
  SetSeed(42);
  WriteLn(RandomInt(1, 6))
end.
```

Do not use this unit for secrets, tokens, passwords, keys, or nonces. Use [`Std.Crypto`](../cryptography/crypto.md) when unpredictability is a security requirement.

## Importing and names

After `uses Std.Random;` use **`Random`**, **`RandomInt`**, **`Randomize`**, and **`SetSeed`**, or their fully qualified `Std.Random.*` forms.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Random(): real` | Pseudo-random value in `[0.0, 1.0)` |
| function | `RandomInt(Lo: integer; Hi: integer): integer` | Unbiased pseudo-random value in inclusive `[Lo, Hi]` |
| procedure | `Randomize()` | Replace the VM-local state with an operating-system seed |
| procedure | `SetSeed(Seed: integer)` | Start a repeatable VM-local sequence |

The generator state belongs to one VM and is shared by its tasks. Calls are synchronized. With the same seed and the same call order, a program receives the same sequence on supported platforms. Concurrent tasks can change the call order through scheduling, so coordinate them when replay order matters.

Calling `Random` or `RandomInt` before either seeding procedure automatically initializes the state from the operating system.

## `function Random(): real`

Returns a pseudo-random real number in `[0.0, 1.0)`.

```pascal
var R: real := Random()
```

## `function RandomInt(Lo: integer; Hi: integer): integer`

Returns an unbiased pseudo-random integer in `[Lo, Hi]`, including either bound. The full `integer` range is supported. A runtime error occurs when `Lo > Hi`.

```pascal
var Die: integer := RandomInt(1, 6)
```

## `procedure Randomize()`

Replaces the current VM's pseudo-random state with a seed supplied by the operating system. A runtime error occurs if that source is unavailable; the runtime never substitutes a predictable seed.

```pascal
Randomize()
```

## `procedure SetSeed(Seed: integer)`

Replaces the current VM's pseudo-random state with the repeatable sequence selected by `Seed`. Reusing a seed restarts that sequence.

```pascal
SetSeed(17);
var First: integer := RandomInt(1, 100);
SetSeed(17);
var Repeated: integer := RandomInt(1, 100)
```

## Implementation (contributors)

`Std.Random` uses a VM-owned ChaCha12 generator. `SetSeed` expands the signed integer with a fixed SplitMix64 mapping; real values use the high 53 bits of the next word, and integer ranges use rejection sampling. These choices make seeded sequences independent of host endianness and operating system.

| Concern | Location |
|---------|----------|
| VM state and runtime | [`hosted/random/`](../../../../crates/fpas-vm/src/vm/hosted/random/) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/random.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/random.rs) |

## See also

- [`Std.Crypto`](../cryptography/crypto.md)
- [Numeric index](README.md)
- [Standard library index](../README.md)

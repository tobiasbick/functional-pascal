# `Std.Math.Random`

Deterministic pseudo-random number helpers for simulations, games, randomized tests, and similar non-security work. This page is the **full API** for the unit.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Math.Random as Random;

begin
  Random.SetSeed(42);
  Console.WriteLn(Random.RandomInt(1, 6));
end.
```

Do not use this unit for secrets, tokens, passwords, keys, or nonces. Use [`Std.Crypto`](../cryptography/crypto.md) when unpredictability is a security requirement.

## Importing and names

Import with `uses Std.Math.Random as Random;` and access public symbols as `Random.Name`. User enum variants require `Random.Type.Variant`.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Random(): real` | Pseudo-random value in `[0.0, 1.0)` |
| function | `RandomInt(Lo: integer; Hi: integer): integer` | Unbiased pseudo-random value in inclusive `[Lo, Hi]` |
| procedure | `Randomize()` | Replace the VM-local state with an operating-system seed |
| procedure | `SetSeed(Seed: integer)` | Start a repeatable VM-local sequence |

The generator state belongs to one VM and is shared by its tasks. Calls are synchronized. With the same seed and the same call order, a program receives the same sequence on supported platforms. Concurrent tasks can change the call order through scheduling, so coordinate them when replay order matters.

Calling `Random` or `RandomInt` before either seeding procedure automatically initializes the state from the operating system.

## Execution and ownership

In pool tasks and ordinary all-stop debugging, `Randomize` and automatic first
seeding release the VM worker while an owned native job obtains OS entropy.
The generator lock is never held during that wait. `SetSeed` and draws from an
initialized generator use only synchronized in-memory state.

The VM shares a limit of 64 native entropy jobs with `Std.Crypto`, including
results awaiting delivery. Additional calls queue without changing the public
return types. Cancellation discards an undelivered seed; started OS calls must
return before their job can be joined. A failed randomization leaves the current
state intact. Automatic seeding never overwrites a seed another task installed
while the OS request was pending. Explicit `Randomize` installs its seed when
its result is delivered. Coordinate concurrent calls when their order matters.

Main-program and explicit synchronous evaluation retain synchronous execution.
Seeded scheduling continues to reject this unit's effects.

## `function Random(): real`

Returns a pseudo-random real number in `[0.0, 1.0)`.

```pascal id=example-2 fragment context=body-random
const R: real := Random.Random();
```

## `function RandomInt(Lo: integer; Hi: integer): integer`

Returns an unbiased pseudo-random integer in `[Lo, Hi]`, including either bound. The full `integer` range is supported. A runtime error occurs when `Lo > Hi`.

```pascal id=example-3 fragment context=body-random
const Die: integer := Random.RandomInt(1, 6);
```

## `procedure Randomize()`

Replaces the current VM's pseudo-random state with a seed supplied by the operating system. A runtime error occurs if that source is unavailable; the runtime never substitutes a predictable seed.

```pascal id=example-4 fragment context=body-random
Random.Randomize();
```

## `procedure SetSeed(Seed: integer)`

Replaces the current VM's pseudo-random state with the repeatable sequence selected by `Seed`. Reusing a seed restarts that sequence.

```pascal id=example-5 fragment context=body-random
Random.SetSeed(17);
const First: integer := Random.RandomInt(1, 100);
Random.SetSeed(17);
const Repeated: integer := Random.RandomInt(1, 100);
```

## Implementation (contributors)

`Std.Math.Random` uses a VM-owned ChaCha12 generator. `SetSeed` expands the signed integer with a fixed SplitMix64 mapping; real values use the high 53 bits of the next word, and integer ranges use rejection sampling. These choices make seeded sequences independent of host endianness and operating system.

| Concern | Location |
|---------|----------|
| VM state and runtime | [`hosted/random/`](../../../../crates/fpas-runtime/src/execution/vm/hosted/random) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/random.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/random.rs) |

## See also

- [`Std.Crypto`](../cryptography/crypto.md)
- [Numeric index](README.md)
- [Standard library index](../README.md)

# `Std.Crypto`

Cryptographically secure randomness supplied directly by the operating system. This page is the **full API** for the unit.

```pascal id=example-1
program Example;

uses Std.Crypto as Crypto;

begin
  const Token: array of integer := Crypto.RandomBytes(32).Unwrap();
end.
```

## Importing and names

Import with `uses Std.Crypto as Crypto;` and access public symbols as `Crypto.Name`. User enum variants require `Crypto.Type.Variant`.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `RandomBytes(Count: integer): result of (array of integer, Crypto.CryptoError)` | `Count` secure bytes, each in `0..255` |
| function | `RandomInt(Lo: integer; Hi: integer): result of (integer, Crypto.CryptoError)` | Unbiased secure value in inclusive `[Lo, Hi]` |

Both functions request randomness from the operating system. Failure is returned as `Error(Cause)` and never falls back to `Std.Math.Random`.

## `RandomBytes`

Returns exactly `Count` bytes. Each byte is represented by an `integer` in `0..255`, matching the existing FPAS byte-array convention. `Count` must be in `0..1048576`; invalid counts return `Error` without contacting the operating-system source.

```pascal id=example-2 fragment context=body-console-crypto
case Crypto.RandomBytes(32) of
  when Ok(const Bytes):
    Console.WriteLn(Bytes.Length());
  when Error(const Message):
    panic(Message.Message());
end case;
```

## `RandomInt`

Returns a uniformly sampled integer in the inclusive range `[Lo, Hi]`. Equal bounds and the full `integer` range are supported. `Lo > Hi` returns `Error`.

```pascal id=example-3 fragment context=body-console-crypto
case Crypto.RandomInt(100000, 999999) of
  when Ok(const Code):
    Console.WriteLn(Code);
  when Error(const Message):
    panic(Message.Message());
end case;
```

## Security boundary

The unit supplies random material; it does not yet supply hashing, password hashing, message authentication, signatures, key storage, or constant-time comparison. Do not construct those algorithms yourself from `RandomBytes`.

## Execution and ownership

Pool tasks and ordinary all-stop debugging release their VM worker during OS
entropy requests, including calls inside higher-order callbacks. Arguments are
validated before admission; native jobs own byte buffers and integer sampling,
while the calling VM task constructs the FPAS result.

Up to 64 native entropy jobs per VM include undelivered results and are shared
with `Std.Math.Random` seeding. Further requests queue. Abandonment cancels queued
requests and joins started work before discarding native buffers. Integer
rejection sampling checks cancellation between OS samples; an OS call already
in progress must return before joining completes. No failure substitutes a
predictable source. Main-program and explicit synchronous evaluation retain
synchronous execution; seeded scheduling rejects this unit's effects.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Hosted runtime | [`crypto.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/crypto.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/crypto.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/crypto.rs) |

## See also

- [`Std.Math.Random`](../numeric/random.md)
- [Cryptography index](README.md)
- [Future cryptography work](../../../future/networked-applications/cryptography.md)

## Typed failures

`CryptoError.InvalidArgument(Name, Reason)` reports the existing Count/bound validation. `EntropyUnavailable(Cause)` retains the secure backend failure as [HostError](../host/errors.md), with a native code only when actually supplied. The runtime never falls back to a pseudorandom generator.

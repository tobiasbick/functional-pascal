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
| function | `RandomBytes(Count: integer): result of (array of integer, string)` | `Count` secure bytes, each in `0..255` |
| function | `RandomInt(Lo: integer; Hi: integer): result of (integer, string)` | Unbiased secure value in inclusive `[Lo, Hi]` |

Both functions request randomness from the operating system. Failure is returned as `Error(Message)` and never falls back to `Std.Random`.

## `RandomBytes`

Returns exactly `Count` bytes. Each byte is represented by an `integer` in `0..255`, matching the existing FPAS byte-array convention. `Count` must be in `0..1048576`; invalid counts return `Error` without contacting the operating-system source.

```pascal id=example-2 fragment context=body-console-crypto
case Crypto.RandomBytes(32) of
  when Ok(const Bytes):
    Console.WriteLn(Bytes.Length());
  when Error(const Message):
    panic(Message);
end case;
```

## `RandomInt`

Returns a uniformly sampled integer in the inclusive range `[Lo, Hi]`. Equal bounds and the full `integer` range are supported. `Lo > Hi` returns `Error`.

```pascal id=example-3 fragment context=body-console-crypto
case Crypto.RandomInt(100000, 999999) of
  when Ok(const Code):
    Console.WriteLn(Code);
  when Error(const Message):
    panic(Message);
end case;
```

## Security boundary

The unit supplies random material; it does not yet supply hashing, password hashing, message authentication, signatures, key storage, or constant-time comparison. Do not construct those algorithms yourself from `RandomBytes`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Hosted runtime | [`crypto.rs`](../../../../crates/fpas-vm/src/vm/hosted/crypto.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/crypto.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/crypto.rs) |

## See also

- [`Std.Random`](../numeric/random.md)
- [Cryptography index](README.md)
- [Future cryptography work](../../../future/networked-applications/cryptography.md)

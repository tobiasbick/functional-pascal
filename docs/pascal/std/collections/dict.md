# `Std.Dictionaries`

Operations on **dictionaries** (`dict of (K, V)`). Dictionaries preserve insertion order when iterated. Key lookup, removal and merge use structural equality; dictionary equality compares the key/value mapping independently of order.

Key types must support structural equality, including all stored components.
Resources, tasks and callables cannot be keys, even through aliases or containers.
Literal construction retains the first position of equal keys and their last
value; all supplied keys and values still execute in written order. See
[Dictionary construction](../../language/types/dictionaries.md#key-types-and-construction).

```pascal
program Example;

uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

begin
  const Ages: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
  Console.WriteLn(Dictionaries.Length(Ages));
  Console.WriteLn(Dictionaries.ContainsKey(Ages, 'Alice'));
end program;
```


## Importing and names

Import with `uses Std.Dictionaries as Dictionaries;`. Access every exported member through `Dictionaries`, for example `Dictionaries.Length(...)`. Imports open no short names.

Explicit aliases keep names from different units distinct. Imported routines use alias-qualified calls; receiver-call lookup applies only to routines declared locally.

---

## Quick reference

All routines are **generic over key type `K` and value type `V`**.

| Kind | Name | Notes |
|------|------|--------|
| function | `Length(D: dict of (K, V)): integer` | number of entries |
| function | `ContainsKey(D: dict of (K, V); Key: K): boolean` | whether key exists |
| function | `Keys(D: dict of (K, V)): array of (K)` | all keys in insertion order |
| function | `Values(D: dict of (K, V)): array of (V)` | all values in insertion order |
| function | `Remove(D: dict of (K, V); Key: K): dict of (K, V)` | new dict without the given key |
| function | `Get(D: dict of (K, V); Key: K): Option of (V)` | safe lookup; `Option.None` if absent |
| function | `Merge(D1: dict of (K, V); D2: dict of (K, V)): dict of (K, V)` | combined dict; `D2` wins on conflict |
| function | `Map(D: dict of (K, V); F: pure function(V: V): V2): dict of (K, V2)` | transform all values |
| function | `Filter(D: dict of (K, V); F: pure function(K: K; V: V): boolean): dict of (K, V)` | keep matching entries |
| function | `Reduce(D: dict of (K, V); Init: U; F: pure function(Acc: U; Key: K; Value: V): U): U` | fold entries in insertion order |

---

## Detailed reference

### `Length`

```pascal
pure function Length(D: dict of (K, V)): integer;
```

Returns the number of key-value pairs in the dict.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

const D: dict of (string, integer) := ['A': 1, 'B': 2];
Console.WriteLn(Dictionaries.Length(D)); // 2
Console.WriteLn(Dictionaries.Length([:])); // 0
```

### `ContainsKey`

```pascal
pure function ContainsKey(D: dict of (K, V); Key: K): boolean;
```

Returns `true` if the dict contains the given key, `false` otherwise.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

const D: dict of (string, integer) := ['Alice': 30];
Console.WriteLn(Dictionaries.ContainsKey(D, 'Alice')); // true
Console.WriteLn(Dictionaries.ContainsKey(D, 'Bob')); // false
```

### `Keys`

```pascal
pure function Keys(D: dict of (K, V)): array of (K);
```

Returns an array of all keys in insertion order.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

const D: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
Console.WriteLn(Dictionaries.Keys(D)); // [Alice, Bob]
```

### `Values`

```pascal
pure function Values(D: dict of (K, V)): array of (V);
```

Returns an array of all values in insertion order.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

const D: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
Console.WriteLn(Dictionaries.Values(D)); // [30, 25]
```

### `Remove`

```pascal
pure function Remove(D: dict of (K, V); Key: K): dict of (K, V);
```

Returns a new dict without the given key. If the key does not exist, the original dict is returned unchanged. The original dict is not modified (immutable semantics).

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

const D: dict of (string, integer) := ['A': 1, 'B': 2, 'C': 3];
const D2: dict of (string, integer) := Dictionaries.Remove(D, 'B');
Console.WriteLn(D2); // {A: 1, C: 3}
```

---

### `Get`

```pascal
pure function Get(D: dict of (K, V); Key: K): Option of (V);
```

Safe lookup. Returns `Option.Some(value)` if the key exists, `Option.None` otherwise. Requires `uses Std.Options as Options;` to pattern-match on the result.

```pascal
uses Std.Dictionaries as Dictionaries;
uses Std.Options as Options;

const D: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
const Age: option of (integer) := Dictionaries.Get(D, 'Alice'); // Some(30)
const Missing: option of (integer) := Dictionaries.Get(D, 'Eve'); // None
```

---

### `Merge`

```pascal
pure function Merge(D1: dict of (K, V); D2: dict of (K, V)): dict of (K, V);
```

Returns a new dict containing all entries from both `D1` and `D2`. When the same key exists in both, `D2` wins (last-write-wins). The original dicts are not modified.

```pascal
uses Std.Dictionaries as Dictionaries;

const Base: dict of (string, integer) := ['A': 1, 'B': 2];
const Over: dict of (string, integer) := ['B': 9, 'C': 3];
const M: dict of (string, integer) := Dictionaries.Merge(Base, Over);
```

---

### `Map`

```pascal
pure function Map(D: dict of (K, V); F: pure function(V: V): V2): dict of (K, V2);
```

Transforms every value in `D` by applying `F` to it. Keys are preserved; the result is a new dict of the same size. The original dict is not modified.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

pure function DoublePrice(V: real): real;
begin
  return V * 2.0;
end function;

  const Prices: dict of (string, real) := ['Apple': 1.0, 'Banana': 0.5];
  const Doubled: dict of (string, real) := Dictionaries.Map(Prices, DoublePrice);
  Console.WriteLn(Doubled); // {Apple: 2.0, Banana: 1.0}
```

---

### `Filter`

```pascal
pure function Filter(D: dict of (K, V); F: pure function(K: K; V: V): boolean): dict of (K, V);
```

Returns a new dict containing only the entries for which `F(K, V)` returns `true`. The original dict is not modified.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

pure function IsPassingScore(K: string; V: integer): boolean;
begin
  return V >= 60;
end function;

  const Scores: dict of (string, integer) := ['Alice': 90, 'Bob': 55, 'Carol': 80];
  const Passing: dict of (string, integer) := Dictionaries.Filter(Scores, IsPassingScore);
  Console.WriteLn(Passing); // {Alice: 90, Carol: 80}
```

---

### `Reduce`

```pascal
pure function Reduce(D: dict of (K, V); Init: U; F: pure function(Acc: U; Key: K; Value: V): U): U;
```

Visits entries in insertion order. Each callback receives the current accumulator, key, and value. The accumulator type `U` is inferred from `Init`, and the callback must return `U`.

```pascal
uses Std.Conv as Conv;
uses Std.Dictionaries as Dictionaries;

pure function Describe(Acc: string; Key: string; Value: integer): string;
begin
  return (((Acc + Key) + ':') + Conv.IntToStr(Value)) + ';';
end function;

const Scores: dict of (string, integer) := ['Alice': 90, 'Bob': 55];
const Text: string := Dictionaries.Reduce(Scores, '', Describe);
```

For an empty dictionary, `Reduce` returns `Init` without invoking `F`. Arguments are evaluated once in the usual left-to-right order. Entries are visited in insertion order. The callback must be explicitly pure and cannot change captured mutable state. On callback failure, the operation stops and propagates the error without returning a partial result.

---

## Dict literals and indexing

Dict literals use bracket syntax with `:` separating keys from values:

```pascal
const D: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
const Empty: dict of (string, integer) := [:];

```

Indexing uses bracket syntax (same as arrays):

```pascal
const Age: integer := D['Alice']; // read
 var M: dict of (string, integer) := ['A': 1];
M['A'] := 2; // update existing key
M['B'] := 3; // insert new key
```

Accessing a non-existent key raises a runtime error. Use `Std.Dictionaries.ContainsKey` to check first.

The expression operator `Key in D` is shorthand for checking key membership and returns `boolean`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/builtins/dict.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/dict.rs) |
| Runtime | [`dict.rs`](../../../../crates/fpas-std/src/dict.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs) |

## See also

- [Collections index](README.md)
- [Standard library index](../README.md)

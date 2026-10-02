# `Std.Dictionaries`

Operations on **dictionaries** (`dict of K to V`). Dictionaries are ordered key-value collections that preserve insertion order.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

begin
  var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
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
| function | `Length(D: dict of K to V): integer` | number of entries |
| function | `ContainsKey(D: dict of K to V; Key: K): boolean` | whether key exists |
| function | `Keys(D: dict of K to V): array of K` | all keys in insertion order |
| function | `Values(D: dict of K to V): array of V` | all values in insertion order |
| function | `Remove(D: dict of K to V; Key: K): dict of K to V` | new dict without the given key |
| function | `Get(D: dict of K to V; Key: K): Option of V` | safe lookup; `None` if absent |
| function | `Merge(D1: dict of K to V; D2: dict of K to V): dict of K to V` | combined dict; `D2` wins on conflict |
| function | `Map(D: dict of K to V; F: function(V: V): V2): dict of K to V2` | transform all values |
| function | `Filter(D: dict of K to V; F: function(K: K; V: V): boolean): dict of K to V` | keep matching entries |
| function | `Reduce(D: dict of K to V; Init: U; F: function(Acc: U; Key: K; Value: V): U): U` | fold entries in insertion order |

---

## Detailed reference

### `Length`

```pascal
function Length(D: dict of K to V): integer;
```

Returns the number of key-value pairs in the dict.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

var D: dict of string to integer := ['A': 1, 'B': 2];
Console.WriteLn(Dictionaries.Length(D)); // 2
Console.WriteLn(Dictionaries.Length([:])); // 0
```

### `ContainsKey`

```pascal
function ContainsKey(D: dict of K to V; Key: K): boolean;
```

Returns `true` if the dict contains the given key, `false` otherwise.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

var D: dict of string to integer := ['Alice': 30];
Console.WriteLn(Dictionaries.ContainsKey(D, 'Alice')); // true
Console.WriteLn(Dictionaries.ContainsKey(D, 'Bob')); // false
```

### `Keys`

```pascal
function Keys(D: dict of K to V): array of K;
```

Returns an array of all keys in insertion order.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
Console.WriteLn(Dictionaries.Keys(D)); // [Alice, Bob]
```

### `Values`

```pascal
function Values(D: dict of K to V): array of V;
```

Returns an array of all values in insertion order.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
Console.WriteLn(Dictionaries.Values(D)); // [30, 25]
```

### `Remove`

```pascal
function Remove(D: dict of K to V; Key: K): dict of K to V;
```

Returns a new dict without the given key. If the key does not exist, the original dict is returned unchanged. The original dict is not modified (immutable semantics).

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
var D2: dict of string to integer := Dictionaries.Remove(D, 'B');
Console.WriteLn(D2); // {A: 1, C: 3}
```

---

### `Get`

```pascal
function Get(D: dict of K to V; Key: K): Option of V;
```

Safe lookup. Returns `Some(value)` if the key exists, `None` otherwise. Requires `uses Std.Options as Options;` to pattern-match on the result.

```pascal
uses Std.Dictionaries as Dictionaries;
uses Std.Options as Options;

var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Age: option of integer := Dictionaries.Get(D, 'Alice'); // Some(30)
var Missing: option of integer := Dictionaries.Get(D, 'Eve'); // None
```

---

### `Merge`

```pascal
function Merge(D1: dict of K to V; D2: dict of K to V): dict of K to V;
```

Returns a new dict containing all entries from both `D1` and `D2`. When the same key exists in both, `D2` wins (last-write-wins). The original dicts are not modified.

```pascal
uses Std.Dictionaries as Dictionaries;

var Base: dict of string to integer := ['A': 1, 'B': 2];
var Over: dict of string to integer := ['B': 9, 'C': 3];
var M: dict of string to integer := Dictionaries.Merge(Base, Over);
```

---

### `Map`

```pascal
function Map(D: dict of K to V; F: function(V: V): V2): dict of K to V2;
```

Transforms every value in `D` by applying `F` to it. Keys are preserved; the result is a new dict of the same size. The original dict is not modified.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

function DoublePrice(V: real): real;
begin
  return V * 2.0;
end function;

  var Prices: dict of string to real := ['Apple': 1.0, 'Banana': 0.5];
  var Doubled: dict of string to real := Dictionaries.Map(Prices, DoublePrice);
  Console.WriteLn(Doubled); // {Apple: 2.0, Banana: 1.0}
```

---

### `Filter`

```pascal
function Filter(D: dict of K to V; F: function(K: K; V: V): boolean): dict of K to V;
```

Returns a new dict containing only the entries for which `F(K, V)` returns `true`. The original dict is not modified.

```pascal
uses Std.Console as Console;
uses Std.Dictionaries as Dictionaries;

function IsPassingScore(K: string; V: integer): boolean;
begin
  return V >= 60;
end function;

  var Scores: dict of string to integer := ['Alice': 90, 'Bob': 55, 'Carol': 80];
  var Passing: dict of string to integer := Dictionaries.Filter(Scores, IsPassingScore);
  Console.WriteLn(Passing); // {Alice: 90, Carol: 80}
```

---

### `Reduce`

```pascal
function Reduce(D: dict of K to V; Init: U; F: function(Acc: U; Key: K; Value: V): U): U;
```

Visits entries in insertion order. Each callback receives the current accumulator, key, and value. The accumulator type `U` is inferred from `Init`, and the callback must return `U`.

```pascal
uses Std.Conv as Conv;
uses Std.Dictionaries as Dictionaries;

function Describe(Acc: string; Key: string; Value: integer): string;
begin
  return (((Acc + Key) + ':') + Conv.IntToStr(Value)) + ';';
end function;

var Scores: dict of string to integer := ['Alice': 90, 'Bob': 55];
var Text: string := Dictionaries.Reduce(Scores, '', Describe);
```

For an empty dictionary, `Reduce` returns `Init` without invoking `F`. Arguments are evaluated once in the usual left-to-right order. The operation visits the entries of its input value as they stood when the call began. A callback may change a captured mutable dictionary binding, but that does not change which entries this call visits. On callback failure, the operation stops and propagates the error without returning a partial result; earlier callback side effects remain visible.

---

## Dict literals and indexing

Dict literals use bracket syntax with `:` separating keys from values:

```pascal
var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Empty: dict of string to integer := [:];

```

Indexing uses bracket syntax (same as arrays):

```pascal
var Age: integer := D['Alice']; // read
mutable var M: dict of string to integer := ['A': 1];
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

# `Std.Dict`

Operations on **dictionaries** (`dict of K to V`). Dictionaries are ordered key-value collections that preserve insertion order.

```pascal
program Example;
uses Std.Console, Std.Dict;
begin
  var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
  WriteLn(Length(Ages));
  WriteLn(ContainsKey(Ages, 'Alice'))
end.
```

**Maintenance (implementers only):** align with [`std_registry/builtins/dict.rs`](../../../crates/fpas-sema/src/std_registry/builtins/dict.rs), [`dict.rs`](../../../crates/fpas-std/src/dict.rs), [`std_calls/dict.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/dict.rs), and [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs).

---

## Importing and names

After `uses Std.Dict;` use short names (`Length`, `ContainsKey`, …) or qualified (`Std.Dict.Length`, …).

**Ambiguity with `Std.Str` and `Std.Array`:** the short name **`Length`** clashes. Qualify as `Std.Dict.Length(D)` vs `Std.Array.Length(A)` vs `Std.Str.Length(S)`.

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

---

## Detailed reference

### `Length`

```pascal
function Length(D: dict of K to V): integer;
```

Returns the number of key-value pairs in the dict.

```pascal
var D: dict of string to integer := ['A': 1, 'B': 2];
WriteLn(Std.Dict.Length(D));  { 2 }
WriteLn(Std.Dict.Length([:]))  { 0 }
```

### `ContainsKey`

```pascal
function ContainsKey(D: dict of K to V; Key: K): boolean;
```

Returns `true` if the dict contains the given key, `false` otherwise.

```pascal
var D: dict of string to integer := ['Alice': 30];
WriteLn(Std.Dict.ContainsKey(D, 'Alice'));    { true }
WriteLn(Std.Dict.ContainsKey(D, 'Bob'))       { false }
```

### `Keys`

```pascal
function Keys(D: dict of K to V): array of K;
```

Returns an array of all keys in insertion order.

```pascal
var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
WriteLn(Std.Dict.Keys(D))  { [Alice, Bob] }
```

### `Values`

```pascal
function Values(D: dict of K to V): array of V;
```

Returns an array of all values in insertion order.

```pascal
var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
WriteLn(Std.Dict.Values(D))  { [30, 25] }
```

### `Remove`

```pascal
function Remove(D: dict of K to V; Key: K): dict of K to V;
```

Returns a new dict without the given key. If the key does not exist, the original dict is returned unchanged. The original dict is not modified (immutable semantics).

```pascal
var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
var D2: dict of string to integer := Std.Dict.Remove(D, 'B');
WriteLn(D2)  { [A: 1, C: 3] }
```

---

### `Get`

```pascal
function Get(D: dict of K to V; Key: K): Option of V;
```

Safe lookup. Returns `Some(value)` if the key exists, `None` otherwise. Requires `uses Std.Option` to pattern-match on the result.

```pascal
uses Std.Dict, Std.Option;

var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Age: Option of integer := Std.Dict.Get(D, 'Alice');    { Some(30) }
var Missing: Option of integer := Std.Dict.Get(D, 'Eve');  { None }
```

---

### `Merge`

```pascal
function Merge(D1: dict of K to V; D2: dict of K to V): dict of K to V;
```

Returns a new dict containing all entries from both `D1` and `D2`. When the same key exists in both, `D2` wins (last-write-wins). The original dicts are not modified.

```pascal
var Base: dict of string to integer := ['A': 1, 'B': 2];
var Over: dict of string to integer := ['B': 9, 'C': 3];
var M: dict of string to integer := Std.Dict.Merge(Base, Over);
{ [A: 1, B: 9, C: 3] }
```

---

### `Map`

```pascal
function Map(D: dict of K to V; F: function(V: V): V2): dict of K to V2;
```

Transforms every value in `D` by applying `F` to it. Keys are preserved; the result is a new dict of the same size. The original dict is not modified.

```pascal
var Prices: dict of string to real := ['Apple': 1.0, 'Banana': 0.5];
var Doubled: dict of string to real := Std.Dict.Map(Prices, function(V: real): real begin return V * 2.0 end);
WriteLn(Doubled)  { [Apple: 2.0, Banana: 1.0] }
```

---

### `Filter`

```pascal
function Filter(D: dict of K to V; F: function(K: K; V: V): boolean): dict of K to V;
```

Returns a new dict containing only the entries for which `F(K, V)` returns `true`. The original dict is not modified.

```pascal
var Scores: dict of string to integer := ['Alice': 90, 'Bob': 55, 'Carol': 80];
var Passing: dict of string to integer :=
  Std.Dict.Filter(Scores, function(K: string; V: integer): boolean begin return V >= 60 end);
WriteLn(Passing)  { [Alice: 90, Carol: 80] }
```

---

## Dict literals and indexing

Dict literals use bracket syntax with `:` separating keys from values:

```pascal
var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Empty: dict of string to integer := [:];
```

Indexing uses bracket syntax (same as arrays):

```pascal
var Age: integer := D['Alice'];       { read }
mutable var M: dict of string to integer := ['A': 1];
M['A'] := 2;                          { update existing key }
M['B'] := 3                           { insert new key }
```

Accessing a non-existent key raises a runtime error. Use `Std.Dict.ContainsKey` to check first.

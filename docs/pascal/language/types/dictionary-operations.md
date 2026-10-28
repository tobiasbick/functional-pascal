# Dictionary operations

Operations on **dictionaries** (`dict of K to V`). Dictionaries are ordered key-value collections that preserve insertion order.

```pascal id=example-1
program Example;

uses Std.Console as Console;

begin
  const Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Console.WriteLn(Ages.Length());
  Console.WriteLn('Alice' in Ages);
end.
```


## Availability and call forms

These built-in type operations are always available without imports. Use
dot calls with positional or fully named explicit arguments. The static
receiver type and operation name select one catalog entry; ordinary free
functions with the same name do not affect this selection.

---

## Quick reference

| Operation | Result / behavior |
| --- | --- |
| `Values.Length(): integer` | Number of entries |
| `Values.IsEmpty(): boolean` | Whether the entry count is zero |
| `Values.Get(Key: K): Option of V` | Safe lookup, or `None` |
| `Values.Keys(): array of K` | Keys in insertion order |
| `Values.Values(): array of V` | Values in insertion order |
| `Values.Map(F: function(V: V): V2): dict of K to V2` | Transform values while retaining keys |
| `Values.Filter(F: function(K: K; V: V): boolean): dict of K to V` | Keep matching entries |
| `Values.Reduce(Init: U; F: function(Acc: U; Key: K; Value: V): U): U` | Fold entries in insertion order |
| `Values.Merge(D2: dict of K to V): dict of K to V` | Combine values; the argument wins on key conflicts |
| `Values.Remove(Key: K): dict of K to V` | Return a value without the key |
| `Values.RemoveInPlace(Key: K)` | Remove key in writable storage; absent keys leave it unchanged |
| `Values.MergeInPlace(D2: dict of K to V)` | Merge into writable storage; argument wins on conflicts |
| `Values.FilterInPlace(F: function(K: K; V: V): boolean)` | Keep matching entries in writable storage |
| `Values.MapInPlace(F: function(V: V): V)` | Transform writable values without changing their type |

`Length()` and `IsEmpty()` use the same canonical names for strings, arrays, and dictionaries.

InPlace operations return no value and obey [collection mutation](collection-mutation.md).
Keys retain their type and insertion order; MapInPlace also retains the value type.


## Detailed reference

### `Length`

```text
Values.Length(): integer
```

Returns the number of key-value pairs in the dict.

```pascal id=example-3 fragment context=body-console
const D: dict of string to integer := ['A': 1, 'B': 2];
Console.WriteLn(D.Length()); // 2
Console.WriteLn([:].Length()); // 0
```

### Key membership with `in`

```text
Key in Values
```

Returns `true` if the dict contains the given key, `false` otherwise.

```pascal id=example-5 fragment context=body-console
const D: dict of string to integer := ['Alice': 30];
Console.WriteLn('Alice' in D); // true
Console.WriteLn('Bob' in D); // false
```

### `Keys`

```text
Values.Keys(): array of K
```

Returns an array of all keys in insertion order.

```pascal id=example-7 fragment context=body-console
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
Console.WriteLn(D.Keys()); // [Alice, Bob]
```

### `Values`

```text
Values.Values(): array of V
```

Returns an array of all values in insertion order.

```pascal id=example-9 fragment context=body-console
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
Console.WriteLn(D.Values()); // [30, 25]
```

### `Remove`

```text
Values.Remove(Key: K): dict of K to V
```

Returns a new dict without the given key. If the key does not exist, the original dict is returned unchanged. The original dict is not modified (immutable semantics).

```pascal id=example-11 fragment context=body-console
const D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
const D2: dict of string to integer := D.Remove('B');
Console.WriteLn(D2); // {A: 1, C: 3}
```

---

### `Get`

```text
Values.Get(Key: K): Option of V
```

Safe lookup. Returns `Some(value)` if the key exists, `None` otherwise. The `Option` result needs no import.

```pascal id=example-13 fragment context=body
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Age: option of integer := D.Get('Alice'); // Some(30)
const Missing: option of integer := D.Get('Eve'); // None
```

---

### `Merge`

```text
Values.Merge(D2: dict of K to V): dict of K to V
```

Returns a new dict containing all entries from both `D1` and `D2`. When the same key exists in both, `D2` wins (last-write-wins). The original dicts are not modified.

```pascal id=example-15 fragment context=body
const Base: dict of string to integer := ['A': 1, 'B': 2];
const Over: dict of string to integer := ['B': 9, 'C': 3];
const M: dict of string to integer := Base.Merge(Over);
// {A: 1, B: 9, C: 3}
```

---

### `Map`

```text
Values.Map(F: function(V: V): V2): dict of K to V2
```

Transforms every value in `D` by applying `F` to it. Keys are preserved; the result is a new dict of the same size. The original dict is not modified.

```pascal id=example-17 fragment context=header-console
function DoublePrice(V: real): real;
begin
  return V * 2.0;
end function;
begin

const Prices: dict of string to real := ['Apple': 1.0, 'Banana': 0.5];
const Doubled: dict of string to real := Prices.Map(DoublePrice);
Console.WriteLn(Doubled); // {Apple: 2.0, Banana: 1.0}

end.
```

---

### `Filter`

```text
Values.Filter(F: function(K: K; V: V): boolean): dict of K to V
```

Returns a new dict containing only the entries for which `F(K, V)` returns `true`. The original dict is not modified.

```pascal id=example-19 fragment context=header-console
function IsPassingScore(K: string; V: integer): boolean;
begin
  return V >= 60;
end function;
begin

const Scores: dict of string to integer := ['Alice': 90, 'Bob': 55, 'Carol': 80];
const Passing: dict of string to integer := Scores.Filter(IsPassingScore);
Console.WriteLn(Passing); // {Alice: 90, Carol: 80}

end.
```

---

### `Reduce`

```text
Values.Reduce(Init: U; F: function(Acc: U; Key: K; Value: V): U): U
```

Visits entries in insertion order. Each callback receives the current accumulator, key, and value. The accumulator type `U` is inferred from `Init`, and the callback must return `U`.

```pascal id=example-21 fragment context=header
uses Std.Text.Format as Conv;
function Describe(Acc: string; Key: string; Value: integer): string;
begin
  return Acc + Key + ':' + Conv.IntToStr(Value) + ';';
end function;

const Scores: dict of string to integer := ['Alice': 90, 'Bob': 55];
const Text: string := Scores.Reduce('', Describe);
// 'Alice:90;Bob:55;'

begin end.
```

For an empty dictionary, `Reduce` returns `Init` without invoking `F`. Arguments are evaluated once in the usual left-to-right order. The operation visits the entries of its input value as they stood when the call began. A callback may change a captured mutable dictionary binding, but that does not change which entries this call visits. On callback failure, the operation stops and propagates the error without returning a partial result; earlier callback side effects remain visible.

---

## Dict literals and indexing

Dict literals use bracket syntax with `:` separating keys from values:

```pascal id=example-22 fragment context=body
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Empty: dict of string to integer := [:];
```

Indexing uses bracket syntax (same as arrays):

```pascal id=example-23 fragment context=body
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Age: integer := D['Alice']; // read
var M: dict of string to integer := ['A': 1];
M['A'] := 2; // update existing key
M['B'] := 3; // insert new key
```

Accessing a non-existent key raises a runtime error. Use `Key in Values` to check
first, or `Values.Get(Key)` for an Option-valued lookup. Membership checks keys,
not dictionary values.

The expression operator `Key in D` is shorthand for checking key membership and returns `boolean`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/builtins/dict.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/dict.rs) |
| Runtime | [`dict.rs`](../../../../crates/fpas-runtime/src/stdlib/dict.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/mod.rs`](../../../../crates/fpas-code/src/intrinsic/mod.rs) |

## See also

- [Collections index](../../std/collections/README.md)
- [Standard library index](../../std/README.md)

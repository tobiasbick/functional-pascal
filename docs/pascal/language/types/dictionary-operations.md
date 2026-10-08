# Dictionary operations

Operations on **dictionaries** (`dict of K to V`). Dictionaries are ordered key-value collections that preserve insertion order.

```pascal
program Example;
uses Std.Console;
begin
  const Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
  WriteLn(Ages.Length());
  WriteLn(Ages.ContainsKey('Alice'));
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
| `Values.ContainsKey(Key: K): boolean` | Key membership |
| `Values.Get(Key: K): Option of V` | Safe lookup, or `None` |
| `Values.Keys(): array of K` | Keys in insertion order |
| `Values.Values(): array of V` | Values in insertion order |
| `Values.Map(F: function(V: V): V2): dict of K to V2` | Transform values while retaining keys |
| `Values.Filter(F: function(K: K; V: V): boolean): dict of K to V` | Keep matching entries |
| `Values.Reduce(Init: U; F: function(Acc: U; Key: K; Value: V): U): U` | Fold entries in insertion order |
| `Values.Merge(D2: dict of K to V): dict of K to V` | Combine values; the argument wins on key conflicts |
| `Values.Remove(Key: K): dict of K to V` | Return a value without the key |

`Length()` and `IsEmpty()` use the same canonical names for strings, arrays, and dictionaries.


## Detailed reference

### `Length`

```text
Values.Length(): integer
```

Returns the number of key-value pairs in the dict.

```pascal
const D: dict of string to integer := ['A': 1, 'B': 2];
WriteLn(D.Length());  // 2
WriteLn([:].Length());  // 0
```

### `ContainsKey`

```text
Values.ContainsKey(Key: K): boolean
```

Returns `true` if the dict contains the given key, `false` otherwise.

```pascal
const D: dict of string to integer := ['Alice': 30];
WriteLn(D.ContainsKey('Alice'));    // true
WriteLn(D.ContainsKey('Bob'));       // false
```

### `Keys`

```text
Values.Keys(): array of K
```

Returns an array of all keys in insertion order.

```pascal
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
WriteLn(D.Keys());  // [Alice, Bob]
```

### `Values`

```text
Values.Values(): array of V
```

Returns an array of all values in insertion order.

```pascal
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
WriteLn(D.Values());  // [30, 25]
```

### `Remove`

```text
Values.Remove(Key: K): dict of K to V
```

Returns a new dict without the given key. If the key does not exist, the original dict is returned unchanged. The original dict is not modified (immutable semantics).

```pascal
const D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
const D2: dict of string to integer := D.Remove('B');
WriteLn(D2);  // {A: 1, C: 3}
```

---

### `Get`

```text
Values.Get(Key: K): Option of V
```

Safe lookup. Returns `Some(value)` if the key exists, `None` otherwise. The `Option` result needs no import.

```pascal


const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Age: Option of integer := D.Get('Alice');    // Some(30)
const Missing: Option of integer := D.Get('Eve');  // None
```

---

### `Merge`

```text
Values.Merge(D2: dict of K to V): dict of K to V
```

Returns a new dict containing all entries from both `D1` and `D2`. When the same key exists in both, `D2` wins (last-write-wins). The original dicts are not modified.

```pascal
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

```pascal
function DoublePrice(V: real): real;
begin
  return V * 2.0;
end function;

const Prices: dict of string to real := ['Apple': 1.0, 'Banana': 0.5];
const Doubled: dict of string to real := Prices.Map(DoublePrice);
WriteLn(Doubled);  // {Apple: 2.0, Banana: 1.0}
```

---

### `Filter`

```text
Values.Filter(F: function(K: K; V: V): boolean): dict of K to V
```

Returns a new dict containing only the entries for which `F(K, V)` returns `true`. The original dict is not modified.

```pascal
function IsPassingScore(K: string; V: integer): boolean;
begin
  return V >= 60;
end function;

const Scores: dict of string to integer := ['Alice': 90, 'Bob': 55, 'Carol': 80];
const Passing: dict of string to integer := Scores.Filter(IsPassingScore);
WriteLn(Passing);  // {Alice: 90, Carol: 80}
```

---

### `Reduce`

```text
Values.Reduce(Init: U; F: function(Acc: U; Key: K; Value: V): U): U
```

Visits entries in insertion order. Each callback receives the current accumulator, key, and value. The accumulator type `U` is inferred from `Init`, and the callback must return `U`.

```pascal
uses Std.Conv;

function Describe(Acc: string; Key: string; Value: integer): string;
begin
  return Acc + Key + ':' + Std.Conv.IntToStr(Value) + ';';
end function;

const Scores: dict of string to integer := ['Alice': 90, 'Bob': 55];
const Text: string := Scores.Reduce('', Describe);
// 'Alice:90;Bob:55;'
```

For an empty dictionary, `Reduce` returns `Init` without invoking `F`. Arguments are evaluated once in the usual left-to-right order. The operation visits the entries of its input value as they stood when the call began. A callback may change a captured mutable dictionary binding, but that does not change which entries this call visits. On callback failure, the operation stops and propagates the error without returning a partial result; earlier callback side effects remain visible.

---

## Dict literals and indexing

Dict literals use bracket syntax with `:` separating keys from values:

```pascal
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Empty: dict of string to integer := [:];
```

Indexing uses bracket syntax (same as arrays):

```pascal
const D: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Age: integer := D['Alice'];       // read
var M: dict of string to integer := ['A': 1];
M['A'] := 2;                          // update existing key
M['B'] := 3;                           // insert new key
```

Accessing a non-existent key raises a runtime error. Use `Values.ContainsKey(Key)` to check first.

The expression operator `Key in D` is shorthand for checking key membership and returns `boolean`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/builtins/dict.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/dict.rs) |
| Runtime | [`dict.rs`](../../../../crates/fpas-std/src/dict.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs) |

## See also

- [Collections index](../../std/collections/README.md)
- [Standard library index](../../std/README.md)

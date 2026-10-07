# AP06: Built-in type operation catalog

Package: [AP06: Fixed dot-call targets](README.md)

Status: agreed scope, automatic availability, and one public call form per
operation. The base catalog is extended to preserve every existing distinct
operation of the five type groups. AP06.1 settles the remaining names and
call forms; AP06.3 implements the catalog. This is a planning document.

## Scope and conventions

- The catalog covers `string`, `array of T`, `dict of K to V`, `Option of T`,
  and `Result of (T, E)`. Their operations are available without `uses`.
- Preserve all distinct operations currently exposed by `Std.Str`,
  `Std.Arrays`, `Std.Dictionaries`, `Std.Options`, and `Std.Results`, including
  operations outside the earlier base catalog. Keep one canonical public
  name per operation; retire genuine synonymous copies only after checking
  their complete semantics, constraints, errors, and evaluation behavior.
- The tables' standard-routine names identify the existing implementation
  and migration source. They are not a second public API in the target
  language. The five former units are removed from public imports and
  ordinary calls; other standard units still require explicit `uses`.
- Instance operations have one public form, `Value.Operation(Arguments)`.
  Their receiver supplies the existing routine's first parameter; instance
  tables list only the remaining arguments in their written order.
  Operations that construct a value without a receiver belong to the
  result type; AP06.1 records one canonical type-qualified form for them.
- `T`, `U`, `K`, `V`, `V2`, `E`, and `E2` denote generic types. Existing
  generic inference and operation-specific type constraints still apply.
  Container receivers with generic element, key, value, or error types use
  their known container shape; an unconstrained type parameter alone has no
  catalog operations.
- Apply the [agreed naming rules](README.md#standard-operation-naming-agreed).
  `Length` is the single count name, and `IsEmpty` is the single emptiness
  name for strings, arrays, and dictionaries.
- Existing entries keep their current FPAS semantics. Collection processing
  remains eager; there is no implicit iterator, deferred execution, unwrap,
  retry, or conversion. The receiver is evaluated once before the written
  arguments, which are evaluated once from left to right.
- Scalars, channels, and task handles have no instance catalog entries. Add
  operations for these types only after a concrete need and an explicit
  catalog extension decision. Record methods retain their declared behavior.
- `Push` and `Pop` are preserved as array operations. Their implicit receiver
  uses ordinary dot syntax without a `var` marker or additional parentheses;
  the catalog records that they require a writable receiver. Explicitly
  written `var` arguments keep their markers, and AP17's safety rules still
  apply to writable receivers. AP17.3 delivers these checks. The user
  requested a [follow-up discussion](README.md#follow-up-discussion) before
  AP06.3/AP17.3. Each operation retains one public call form.
- `Join` belongs to `array of string`, according to its actual receiver,
  despite its former placement in `Std.Str`. `Fill` constructs an array;
  `Chr` constructs a string and does not become an integer instance method.
- Existing function references such as `Std.Str.Length` migrate to named
  wrappers or existing anonymous functions calling the type operation.
  Documentation and editor signatures come from the catalog rather than
  public stubs for the removed units.

The tables account for all 75 existing routines in the five former units,
plus the three agreed `IsEmpty` additions. This preserves scope, not duplicate
public spellings. AP06.1 validates equivalent operations and canonical names,
including `Substring` versus the existing array `Slice` name, before code
changes. It must not discard distinct behavior as a mere alias.

## String

Receiver: `string`. Existing behavior: [`Std.Str`](../../../pascal/std/text/str/README.md).

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Str.Length` | `integer` | Number of Unicode scalars |
| `IsEmpty()` | `Std.Str.IsEmpty` (new) | `boolean` | Whether the scalar count is zero |
| `Contains(Sub: string)` | `Std.Str.Contains` | `boolean` | Substring membership |
| `StartsWith(Pre: string)` | `Std.Str.StartsWith` | `boolean` | Prefix test |
| `EndsWith(Suf: string)` | `Std.Str.EndsWith` | `boolean` | Suffix test |
| `IndexOf(Sub: string)` | `Std.Str.IndexOf` | `integer` | First scalar index, or `-1` |
| `Trim()` | `Std.Str.Trim` | `string` | Remove leading and trailing whitespace |
| `ToUpper()` | `Std.Str.ToUpper` | `string` | Uppercased value |
| `ToLower()` | `Std.Str.ToLower` | `string` | Lowercased value |
| `Replace(Old: string; New: string)` | `Std.Str.Replace` | `string` | Replace all occurrences |
| `Split(Delim: string)` | `Std.Str.Split` | `array of string` | Split into segments |
| `Map(F: function(C: string): string)` | `Std.Str.Map` | `string` | Transform each scalar into exactly one scalar |
| `Filter(F: function(C: string): boolean)` | `Std.Str.Filter` | `string` | Keep matching scalars |
| `Reduce(Init: U; F: function(Acc: U; C: string): U)` | `Std.Str.Reduce` | `U` | Fold scalars left to right |

Additional existing operations to preserve:

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `Substring(Start: integer; Len: integer)` | `Std.Str.Substring` | `string` | Return the checked scalar range; canonical name reviewed with array `Slice` |
| `LastIndexOf(Sub: string)` | `Std.Str.LastIndexOf` | `integer` | Last matching scalar index, or `-1` |
| `IsNumeric()` | `Std.Str.IsNumeric` | `boolean` | Whether the value matches the existing numeric-text rules |
| `RepeatStr(N: integer)` | `Std.Str.RepeatStr` | `string` | Repeat the complete string |
| `PadLeft(Width: integer; PadChar: string)` | `Std.Str.PadLeft` | `string` | Pad on the left |
| `PadRight(Width: integer; PadChar: string)` | `Std.Str.PadRight` | `string` | Pad on the right |
| `PadCenter(Width: integer; PadChar: string)` | `Std.Str.PadCenter` | `string` | Center within the padded width |
| `FromChar(N: integer)` | `Std.Str.FromChar` | `string` | Repeat a receiver that must contain exactly one scalar |
| `CharAt(Index: integer)` | `Std.Str.CharAt` | `string` | Read the scalar at the checked index |
| `SetCharAt(Index: integer; C: string)` | `Std.Str.SetCharAt` | `string` | Return a value with one scalar replaced |
| `Ord()` | `Std.Str.Ord` | `integer` | Unicode codepoint of a receiver that must contain exactly one scalar |
| `Insert(Index: integer; Sub: string)` | `Std.Str.Insert` | `string` | Insert text at the scalar index |
| `Delete(Index: integer; Len: integer)` | `Std.Str.Delete` | `string` | Remove the checked scalar range |
| `Reverse()` | `Std.Str.Reverse` | `string` | Return reversed scalars |
| `TrimLeft()` | `Std.Str.TrimLeft` | `string` | Remove leading whitespace |
| `TrimRight()` | `Std.Str.TrimRight` | `string` | Remove trailing whitespace |
| `Format(Arguments...)` | `Std.Str.Format` | `string` | Use the receiver as the format template and retain existing variadic arguments |

String length and character indexes count Unicode scalars, not UTF-8 bytes or
grapheme clusters. String callbacks receive a one-scalar `string`; `Map`
requires each callback result to contain exactly one scalar.
`FromChar` and `RepeatStr` are not interchangeable aliases: `FromChar` rejects
empty and multi-scalar receivers, while `RepeatStr` accepts whole strings.
Retain that distinction. `Format` is variadic even though the current generated
editor declaration lists only its template parameter.

## Array

Receiver: `array of T`. Existing behavior: [`Std.Arrays`](../../../pascal/std/collections/array/README.md).

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Arrays.Length` | `integer` | Number of elements |
| `IsEmpty()` | `Std.Arrays.IsEmpty` (new) | `boolean` | Whether the element count is zero |
| `Contains(Value: T)` | `Std.Arrays.Contains` | `boolean` | Element membership |
| `IndexOf(Value: T)` | `Std.Arrays.IndexOf` | `integer` | First matching index, or `-1` |
| `Map(F: function(X: T): U)` | `Std.Arrays.Map` | `array of U` | Transform each element |
| `Filter(F: function(X: T): boolean)` | `Std.Arrays.Filter` | `array of T` | Keep matching elements |
| `Reduce(Init: U; F: function(Acc: U; V: T): U)` | `Std.Arrays.Reduce` | `U` | Fold elements left to right |
| `Find(F: function(X: T): boolean)` | `Std.Arrays.Find` | `Option of T` | First matching element, or `None` |
| `FindIndex(F: function(X: T): boolean)` | `Std.Arrays.FindIndex` | `integer` | First matching index, or `-1` |
| `Any(F: function(X: T): boolean)` | `Std.Arrays.Any` | `boolean` | Whether any element satisfies the predicate |
| `All(F: function(X: T): boolean)` | `Std.Arrays.All` | `boolean` | Whether every element satisfies the predicate |
| `Sort()` | `Std.Arrays.Sort` | `array of T` | Return a sorted value |
| `Reverse()` | `Std.Arrays.Reverse` | `array of T` | Return a reversed value |
| `Slice(Start: integer; Len: integer)` | `Std.Arrays.Slice` | `array of T` | Return the checked subrange |
| `Concat(B: array of T)` | `Std.Arrays.Concat` | `array of T` | Append the second array's elements to the result |
| `FlatMap(F: function(X: T): array of U)` | `Std.Arrays.FlatMap` | `array of U` | Map each element, then flatten the results |

Additional existing operations to preserve:

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `ForEach(F: procedure(X: T))` | `Std.Arrays.ForEach` | `Unit` | Invoke a procedure per element; only a final chain step |
| `Push(Value: T)` | `Std.Arrays.Push` | `Unit` | Append to the caller's array; requires a writable receiver, with no call-site receiver marker |
| `Pop()` | `Std.Arrays.Pop` | `T` | Remove and return the last element; requires a writable receiver, with no call-site receiver marker |

Specialized receiver: `array of string`.

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `Join(Delim: string)` | `Std.Str.Join` | `string` | Join the receiver's strings with the delimiter |

`Sort`, `Reverse`, `Slice`, and `Concat` return values without changing the
caller variable. All listed entries preserve the existing array-operation
semantics and constraints.

## Operations without an instance receiver

These existing operations remain available as operations of their result
type. The table lists all arguments because no instance receiver is passed.
AP06.1 must record the exact type-qualified spelling, including how a generic
array type is written. No factory syntax is implicitly approved by this table.

| Owning type | Operation and all arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- | --- |
| `string` | `Chr(N: integer)` | `Std.Str.Chr` | `string` | Construct one scalar from a valid Unicode codepoint |
| `array of T` | `Fill(Value: T; Count: integer)` | `Std.Arrays.Fill` | `array of T` | Construct an array of repeated values |

## Dictionary

Receiver: `dict of K to V`. Existing behavior: [`Std.Dictionaries`](../../../pascal/std/collections/dict.md).

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Dictionaries.Length` | `integer` | Number of entries |
| `IsEmpty()` | `Std.Dictionaries.IsEmpty` (new) | `boolean` | Whether the entry count is zero |
| `ContainsKey(Key: K)` | `Std.Dictionaries.ContainsKey` | `boolean` | Key membership |
| `Get(Key: K)` | `Std.Dictionaries.Get` | `Option of V` | Safe lookup, or `None` |
| `Keys()` | `Std.Dictionaries.Keys` | `array of K` | Keys in insertion order |
| `Values()` | `Std.Dictionaries.Values` | `array of V` | Values in insertion order |
| `Map(F: function(V: V): V2)` | `Std.Dictionaries.Map` | `dict of K to V2` | Transform values while retaining keys |
| `Filter(F: function(K: K; V: V): boolean)` | `Std.Dictionaries.Filter` | `dict of K to V` | Keep matching entries |
| `Reduce(Init: U; F: function(Acc: U; Key: K; Value: V): U)` | `Std.Dictionaries.Reduce` | `U` | Fold entries in insertion order |
| `Merge(D2: dict of K to V)` | `Std.Dictionaries.Merge` | `dict of K to V` | Combine values; the argument wins on key conflicts |
| `Remove(Key: K)` | `Std.Dictionaries.Remove` | `dict of K to V` | Return a value without the key |

Dictionary operations preserve their current insertion-order semantics.
`Merge` and `Remove` return values without changing the caller variable.

## Option

Receiver: `Option of T`. Existing behavior: [`Std.Options`](../../../pascal/std/result/option.md).

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `IsSome()` | `Std.Options.IsSome` | `boolean` | Whether a value is present |
| `IsNone()` | `Std.Options.IsNone` | `boolean` | Whether the option is absent |
| `Map(F: function(V: T): U)` | `Std.Options.Map` | `Option of U` | Transform the present value |
| `AndThen(F: function(V: T): Option of U)` | `Std.Options.AndThen` | `Option of U` | Chain an optional operation |
| `OrElse(F: function(): Option of T)` | `Std.Options.OrElse` | `Option of T` | Invoke a fallback only for `None` |
| `Unwrap()` | `Std.Options.Unwrap` | `T` | Extract the value; panic for `None` |
| `UnwrapOr(Default: T)` | `Std.Options.UnwrapOr` | `T` | Extract the value or use the default |

## Result

Receiver: `Result of (T, E)`. Existing behavior: [`Std.Results`](../../../pascal/std/result/result.md).

| Dot call and remaining arguments | Standard routine | Result type | Meaning |
| --- | --- | --- | --- |
| `IsOk()` | `Std.Results.IsOk` | `boolean` | Whether the result is successful |
| `IsError()` | `Std.Results.IsError` | `boolean` | Whether the result is an error |
| `Map(F: function(V: T): U)` | `Std.Results.Map` | `Result of (U, E)` | Transform the successful value |
| `AndThen(F: function(V: T): Result of (U, E))` | `Std.Results.AndThen` | `Result of (U, E)` | Chain an operation with the same error type |
| `OrElse(F: function(Err: E): Result of (T, E2))` | `Std.Results.OrElse` | `Result of (T, E2)` | Invoke error recovery; may change the error type |
| `Unwrap()` | `Std.Results.Unwrap` | `T` | Extract the value; panic for `Error` |
| `UnwrapOr(Default: T)` | `Std.Results.UnwrapOr` | `T` | Extract the value or use the default |

## Shared signatures and required differences

- `Length()` returns `integer`, and `IsEmpty()` returns `boolean`, for all
  three collection receiver types. Their counted units differ by type.
- `Map(F)` transforms the contained value. Arrays can change element type,
  dictionaries change value type while keeping keys, and `Option`/`Result`
  change the success type. Strings preserve one scalar per callback result.
- `Filter(F)` preserves the receiver type. String and array predicates take
  one scalar or element; dictionary predicates take key then value.
- `Reduce(Init, F)` places the initial accumulator before the callback. The
  callback receives the accumulator first, then the scalar or element, or
  dictionary key then value.
- `Contains` and `IndexOf` use an explicit substring or element argument.
  Dictionary membership is explicitly `ContainsKey`.
- `Option.OrElse` uses a callback without arguments; `Result.OrElse` supplies
  the error. `UnwrapOr` uses an ordinary default argument, evaluated even
  when the present or successful value is selected. These preserve existing
  evaluation and callback rules.

## IsEmpty delivery

AP06.3 adds `IsEmpty()` as a native operation of strings, arrays, and
dictionaries. Each returns whether the corresponding `Length` is zero,
evaluates its receiver once, and does not mutate the caller variable. Names
such as `Std.Str.IsEmpty` in the mapping tables identify implementation
ownership only; they do not introduce publicly callable Std routines.

Reuse existing length handling. Update the standard-library registration,
required checking/lowering/runtime layers, catalog-derived editor signatures,
current type documentation, and regression tests in the same delivery.
AP06.2's migration does not depend on these new operations.

## Verification

- Check every catalog mapping against its standard routine's receiver,
  argument roles, result type, generic constraints, and existing semantics.
- Automatic catalog checks enforce unique names per receiver, common
  canonical names, comparable signatures, and the documented differences.
- Account for every existing operation, with either a retained type operation
  or a verified synonymous mapping to its one canonical replacement. Keep
  operations with different constraints or errors distinct.
- Test every supported entry, type-changing chains, and receiver evaluation
  before its arguments. Completion and signature help use the same catalog.
- Test `IsEmpty` on empty and nonempty strings, arrays, and dictionaries,
  without imports, and on receivers with observable evaluation.
- Test `Join` on `array of string`, `ForEach` as a final procedure call,
  variadic `Format`, and the agreed type-qualified `Chr` and `Fill` forms.
- Verify automatic availability, rejection of the removed units and their
  free-call forms, and explicit imports for other standard units.
- Verify that scalars, channels, task handles, and unrelated standard routines
  do not acquire instance entries through free-routine lookup.

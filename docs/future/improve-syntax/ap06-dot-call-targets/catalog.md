# AP06: Built-in type operation catalog

Package: [AP06: Fixed dot-call targets](README.md)

Status: complete. The catalog defines 78 public native operations. Their
signatures, naming, import, factory, collision, and mutation rules are enforced
by `crates/fpas-sema/src/std_registry/native/`. Automatic checks validate the
catalog and its type-handbook signatures.

## Scope and conventions

- The catalog covers `string`, `array of T`, `dict of K to V`, `Option of T`,
  and `Result of (T, E)`. Their operations are available without `uses`.
- Every operation has one canonical public name. Operations with different
  semantics, constraints, or errors remain distinct.
- The private implementation IDs in the tables identify runtime ownership,
  not a second public API. `Std.Str`, `Std.Arrays`, `Std.Dictionaries`,
  `Std.Options`, and `Std.Results` are unavailable as public imports, free
  calls, or function references. Other standard units require explicit `uses`.
- Instance operations have one public form, `Value.Operation(Arguments)`.
  Their receiver supplies the existing routine's first parameter; instance
  tables list the public explicit parameters in declaration order.
  Operations that construct a value without a receiver belong to the
  result type and use `string.Chr(...)` or `array.Fill(...)` as recorded below.
- Explicit arguments of fixed-signature operations may be fully positional
  or fully named, using the
  stable public parameter names in the tables. The receiver is unnamed.
  Apply AP09's case-insensitive name matching, required-argument checks, and
  rejection of unknown, duplicate, or mixed arguments. Named arguments
  evaluate in written order after the receiver, then map to parameter order.
  Thus `Items.Push(3)` and `Items.Push(Value := 3)` select the same operation.
  Variadic `Format` takes positional arguments only; its heterogeneous tail
  has no public parameter names.
- `T`, `U`, `K`, `V`, `V2`, `E`, and `E2` denote generic types. Existing
  generic inference and operation-specific type constraints still apply.
  Container receivers with generic element, key, value, or error types use
  their known container shape; an unconstrained type parameter alone has no
  catalog operations.
- Apply the [agreed naming rules](README.md#standard-operation-naming-agreed).
  `Length` is the single count name, and `IsEmpty` is the single emptiness
  name for strings, arrays, and dictionaries. Treat strings as character
  sequences for naming and use array names for corresponding operations:
  string `Slice` replaces `Substring` while keeping Unicode-scalar behavior.
- Local and imported free routines may share catalog names; those names are
  not globally reserved. A native dot call is selected by its static receiver
  type and case-insensitive operation name. An ordinary free call keeps its
  ordinary name resolution and is not a parallel native API. Invalid native
  argument counts, names, or types never trigger free-routine lookup.
- Each receiver type has one signature per case-insensitive operation name.
  Existing generic inference and constraints apply after target selection;
  trailing arguments and the expected result type do not select overloads.
  Matching names on different receiver types are allowed. Reject duplicate
  catalog entries during validation. Record members, including callable
  fields and instance/static methods, retain their existing shared
  case-insensitive namespace and duplicate-declaration errors.
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
  apply to writable receivers. The shared checks accept writable roots,
  record fields, array elements, imported variables, and forwarded reference
  parameters; they reject read-only bindings and temporary values. The result
  of `Pop` may initialize a `const`. See
  [writable receivers](README.md#writable-receivers).
- `Join` belongs to `array of string`. `Fill` constructs an array; `Chr`
  constructs a string.
- Function values use named wrappers or anonymous functions calling the type
  operation. Documentation and editor signatures come from the catalog.

The [coverage inventory](inventory.md) lists the 78 entries and their
regression owners. Array naming governs equivalent string operations;
`Std.Str.Substring` is the private implementation ID for native string `Slice`.

## String

Receiver: `string`. Type documentation: [String operations](../../../pascal/language/types/string/README.md).

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Str.Length` | `integer` | Number of Unicode scalars |
| `IsEmpty()` | `Std.Str.IsEmpty` | `boolean` | Whether the scalar count is zero |
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

Other operations:

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `Slice(Start: integer; Len: integer)` | `Std.Str.Substring` | `string` | Return the checked scalar range; same public name and argument roles as array `Slice` |
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
| `Format(Arguments...)` | `Std.Str.Format` | `string` | Use the receiver as the format template; heterogeneous variadic arguments are positional only |

String length and character indexes count Unicode scalars, not UTF-8 bytes or
grapheme clusters. String callbacks receive a one-scalar `string`; `Map`
requires each callback result to contain exactly one scalar.
`FromChar` and `RepeatStr` are not interchangeable aliases: `FromChar` rejects
empty and multi-scalar receivers, while `RepeatStr` accepts whole strings.
`Format` is variadic with a positional public form, for example `'%s: %d'.Format('x', 3)`. `Arguments...` describes
the variadic tail rather than a parameter name; reject named arguments with
AP09's positional-only diagnostic. Preserve existing format specifiers and
argument-count/type errors, without an array or spread-argument alternative.

## Array

Receiver: `array of T`. Type documentation: [Array operations](../../../pascal/language/types/array/README.md).

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Arrays.Length` | `integer` | Number of elements |
| `IsEmpty()` | `Std.Arrays.IsEmpty` | `boolean` | Whether the element count is zero |
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

Other operations:

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `ForEach(F: procedure(X: T))` | `Std.Arrays.ForEach` | `Unit` | Invoke a procedure per element; only a final chain step |
| `Push(Value: T)` | `Std.Arrays.Push` | `Unit` | Append to the caller's array; requires a writable receiver, with no call-site receiver marker |
| `Pop()` | `Std.Arrays.Pop` | `T` | Remove and return the last element; requires a writable receiver, with no call-site receiver marker |

Specialized receiver: `array of string`.

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `Join(Delim: string)` | `Std.Str.Join` | `string` | Join the receiver's strings with the delimiter |

`Sort`, `Reverse`, `Slice`, and `Concat` return values without changing the
caller variable. All listed entries preserve the existing array-operation
semantics and constraints. `Sort` preserves its current runtime support for
integer, real, string, and boolean elements; unsupported values still fail at
runtime. `Fill` retains its count and collection-limit checks.

## Operations without an instance receiver

These existing operations remain available as operations of their result
type. The table lists all arguments because no instance receiver is passed.
The agreed public forms are `string.Chr(...)` and `array.Fill(...)`, available
without imports. `Fill` infers the element type `T` from `Value` and returns
`array of T`; no explicit generic type arguments are written. There is no
parallel `(array of T).Fill(...)` form. Binding type annotations remain
required and are checked against the result using existing type rules;
this does not extend AP22 local inference.

| Owning type | Public factory and all arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- | --- |
| `string` | `string.Chr(N: integer)` | `Std.Str.Chr` | `string` | Construct one scalar from a valid Unicode codepoint |
| `array of T` | `array.Fill(Value: T; Count: integer)` | `Std.Arrays.Fill` | `array of T` | Construct an array of repeated values |

Examples using fully named arguments:

```pascal
const Letter: string := string.Chr(N := 65);
const Numbers: array of integer := array.Fill(Value := 7, Count := 3);
const Words: array of string := array.Fill(Value := 'hi', Count := 2);
```

Positional arguments such as `string.Chr(65)` and `array.Fill(7, 3)` select
the same factories. All written arguments follow AP09 mapping and evaluation
rules; there is no receiver to evaluate or name. Existing `Chr` and `Fill`
semantics and error behavior remain unchanged.

## Dictionary

Receiver: `dict of K to V`. Type documentation: [Dictionary operations](../../../pascal/language/types/dictionary-operations.md).

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `Length()` | `Std.Dictionaries.Length` | `integer` | Number of entries |
| `IsEmpty()` | `Std.Dictionaries.IsEmpty` | `boolean` | Whether the entry count is zero |
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

Receiver: `Option of T`. Type documentation: [Option operations](../../../pascal/language/types/option-operations.md).

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
| --- | --- | --- | --- |
| `IsSome()` | `Std.Options.IsSome` | `boolean` | Whether a value is present |
| `IsNone()` | `Std.Options.IsNone` | `boolean` | Whether the option is absent |
| `Map(F: function(V: T): U)` | `Std.Options.Map` | `Option of U` | Transform the present value |
| `AndThen(F: function(V: T): Option of U)` | `Std.Options.AndThen` | `Option of U` | Chain an optional operation |
| `OrElse(F: function(): Option of T)` | `Std.Options.OrElse` | `Option of T` | Invoke a fallback only for `None` |
| `Unwrap()` | `Std.Options.Unwrap` | `T` | Extract the value; panic for `None` |
| `UnwrapOr(Default: T)` | `Std.Options.UnwrapOr` | `T` | Extract the value or use the default |

## Result

Receiver: `Result of (T, E)`. Type documentation: [Result operations](../../../pascal/language/types/result-operations.md).

| Dot call and remaining arguments | Private implementation | Result type | Meaning |
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
- `Slice(Start, Len)` returns a checked range for both strings and arrays.
  String indexes and lengths count Unicode scalars; array indexes and lengths
  count elements. `Substring` is a private implementation ID,
  not an additional native name.
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

## IsEmpty

`IsEmpty()` is a native operation of strings, arrays, and dictionaries. It
returns whether the corresponding `Length` is zero, evaluates its receiver
once, and does not mutate the caller variable. The private IDs in the tables
do not introduce publicly callable Std routines.

## Regression coverage

Catalog checks enforce unique case-insensitive names per receiver, common
canonical names, signatures, type constraints, and documented differences.
Compiler and FPAS regressions cover:

- Every supported entry, type-changing chains, Unicode-scalar string behavior,
  and receiver evaluation before arguments.
- Local/imported free routines with catalog names, fixed dot-call resolution
  on invalid arguments, and duplicate catalog or record-member errors.
- Shared string/array `Slice` names and range checks; rejection of native
  string `Substring` with a `Slice` hint.
- Positional and named native arguments, including writable `Push`, written
  evaluation order, generic inference, and invalid-name/count/mixing errors.
- Empty and nonempty `IsEmpty` receivers, specialized `Join`, final procedure
  `ForEach`, and positional heterogeneous `Format` with its error behavior.
- `string.Chr` and `array.Fill`, reordered named arguments, zero-count type
  inference, binding annotation checks, and rejection of alternate factory
  spellings or explicit type arguments.
- Automatic availability, rejected helper imports/free-call forms, and the
  absence of catalog entries for scalars, channels, and task handles.

Completion, signature help, hover, handbook signatures, and API export use the
same catalog. Test owners are listed in [inventory.md](inventory.md).

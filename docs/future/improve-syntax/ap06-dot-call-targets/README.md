# AP06: Fixed dot-call targets

Status: complete (Q05). Effort: medium. Package completion is tracked in the
[central README](../README.md); work-package completion is listed below.

## Implemented behavior

- A dot call selects a declared record member or one operation from the
  [native catalog](catalog.md), by static receiver type and case-insensitive
  name. Results may return Self, a new record, or another type that continues
  the chain. Callable record fields remain actual member calls. Computed
  properties were removed by [AP14](../ap14-remove-computed-properties/README.md);
  callers invoke declared accessor methods with parentheses.
- String, array, dictionary, Option and Result operations are available without
  imports. Instance calls use `Value.Operation(...)`. The five retired helper
  units have no public imports, ordinary calls or free routine references.
  Other standard units require imports.
- The catalog contains 78 entries: 75 distinct operations and three empty
  checks. All eager-processing, Unicode-scalar, dictionary-order and
  value-returning semantics are retained. `Sort`, `Merge` and `Remove` return
  values without changing caller storage. `Join` belongs to arrays of string.
- The only factories are `string.Chr(N)` and `array.Fill(Value, Count)`, accepting
  positional or fully named arguments. Fill infers its element type from Value,
  even when Count is zero. Typed-array factory spelling and explicit generic
  arguments are rejected; binding type annotations remain required.
- Fixed signatures accept positional or fully named explicit arguments under
  AP09. Names are stable and case-insensitive; unknown, duplicate, missing and
  mixed names are rejected. The receiver is unnamed and evaluates once before
  explicit arguments, which evaluate once in written order before mapping.
  Variadic `Format` is positional and heterogeneous.
- Own free functions and callable values use ordinary calls; they are never
  found through their first parameter. Same-named free routines may coexist
  with native operations. Invalid native arguments never trigger fallback.
- Each receiver type has one signature per operation name. Trailing arguments
  and expected result types do not choose overloads. Generic inference follows
  target selection. Duplicate catalog names are rejected, including specialized
  array overlap. Record members retain their shared case-insensitive namespace.
- Scalars, channels, task handles and unconstrained generic receivers have no
  native instance operations. Known generic container shapes retain their
  catalog operations. Dot notation introduces no pipe operator or alternate
  free-function call form.

## Writable receivers

`Items.Push(Value)` and `Items.Pop()` require writable storage. Their implicit
receiver needs neither a `var` marker nor extra parentheses. Valid receivers
include `var` bindings, writable fields/array elements, imported variables and
forwarded reference parameters. Constants, read-only parameters, dictionary
entries and computed receivers are rejected.

```pascal
var Items: array of integer := [1, 2];
Items.Push(Value := 3);
const Last: integer := Items.Pop();
```

The catalog marks writable receivers. [AP17](../ap17-visible-caller-mutation/README.md)
storage, aliasing, lifetime, task, evaluation and failure rules apply.
Explicit reference arguments retain their `var` markers; own free routines with
reference parameters remain ordinary calls. Record methods retain their declared
Self behavior.

## Standard-operation naming (agreed)

The catalog uses a common vocabulary across receiver types. These rules apply
to catalog operations; user-defined record methods retain their declared names.

- **Same operation, same name.** Equivalent operations use the same dot name
  on every receiver type that supports them. `Length()` is the canonical count
  operation for strings, arrays, and dictionaries: Unicode scalar count,
  element count, and entry count respectively. Shared operations such as `Map`,
  `Filter`, and `Reduce` follow the same rule.
- **String names follow arrays (agreed).** For naming, treat a string as a
  sequence of characters and use the array name for corresponding operations.
  In particular, string `Slice(Start: integer; Len: integer)` replaces the
  public `Substring` spelling and matches array `Slice`; do not expose both
  names. Preserve the string operation's Unicode-scalar indexes, bounds
  checks, result type, and evaluation behavior. String-specific operations
  retain their names when there is no corresponding array operation.
- **One canonical name per operation.** Do not add synonymous catalog names
  such as `Size()` or `Count()` alongside or instead of `Length()` for these
  counts.
- **Comparable signatures.** Corresponding argument roles appear in the same
  order wherever the receiver types permit it. Document differences required
  by the operation, such as an array element predicate versus a dictionary
  key/value predicate.
- **Precise names for different meanings.** A name identifies what an
  operation does. Dictionary key membership uses `ContainsKey`, making its
  distinction from element or substring membership explicit.
- **Type-appropriate availability.** A common vocabulary does not require
  every type to expose every operation. Catalog entries state their meaning,
  argument roles, result type, and any required differences across types.
- **Checked consistency.** Automatic catalog checks and regression tests
  enforce canonical names and comparable signatures, including when the
  catalog is extended.

## Open decisions

None.

## Dependencies

AP09 provides named-argument mapping. AP17 provides shared caller-mutation
checks. Native availability does not depend on AP05 import aliases.
AP12 and AP14 depend on the fixed member/catalog rule.

## Work packages

- [x] [AP06.1: Catalog names and remaining operation rules](01-catalog-and-rules-decision.md)
- [x] [AP06.2: Prepare type-operation and free-call migration](02-migrate-non-catalog-calls.md)
- [x] [AP06.3: Implement native operations and remove duplicate call forms](03-fixed-dot-resolution.md)

## Implementation and coverage

`crates/fpas-sema/src/std_registry/native/` supplies the checker, compiler and
editor catalog. [Coverage](inventory.md) accounts for every entry, consumer
migration, API removal and regression owner. Current usage is documented in
[dot calls](../../../pascal/language/functions/fluent-calls.md) and the
[built-in type pages](../../../pascal/language/types/README.md).

# AP06: Fixed dot-call targets

Status: agreed direction (Q05, revised): built-in type operations are always
available with one public call form, consistent names, and preservation of
existing distinct operations. Canonical names for the extended inventory,
type-qualified factory syntax, name conflicts, and explicit receiver mutation
marking must be settled in AP06.1 before implementation. Effort: medium.
Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

This package was previously titled "Remove automatic receivers". The revised
decision keeps dot chaining for record methods and native built-in type
operations, removes automatic lookup of arbitrary free functions, and replaces
the five former type-helper units with a single public API on the types.

## Goal

A dot call `Value.Name(Arguments)` has a fixed, predictable target: a record
method of the value's record type, or one operation assigned to its built-in
type. Built-in operations require no imports and have no duplicate ordinary
call form. An imported or local free function can never reinterpret a dot call.

Current behavior: `Value.Name(Args)` may resolve to any visible free routine or
callable value whose first parameter accepts the receiver, with layered lookup
rules ([receiver calls](../../../pascal/language/functions/fluent-calls.md),
[postfix chaining](../../../pascal/language/functions/postfix-chaining.md)).

## Decisions (Q05, revised)

- **Record methods keep dot chaining.** Instance methods declared in a record
  (with `Self`) stay callable as `Value.Method(Arguments)`. A method returns
  `Self`'s record type or a new value; the chain continues on the static type
  of that result.
- **Native operations on catalog types.** For `string`, arrays,
  dictionaries, `Option`, and `Result`, existing operations
  such as `Trim`, `Map`, and `Filter` are assigned to dot notation in a fixed,
  unambiguous catalog. Each catalog entry maps one receiver type and one name
  to exactly one implementation. A result may change type, and the
  chain continues on the new type.
- **Automatic availability and one call form.** The five types provide their
  operations directly, without `uses Std.Str`, `Std.Arrays`,
  `Std.Dictionaries`, `Std.Options`, or `Std.Results`. Instance operations use
  `Value.Operation(...)`; operations without an instance receiver use one
  type-qualified form agreed in AP06.1. Remove the five units from public
  imports and remove their ordinary calls and free function references.
  Other standard units, such as `Std.Fs`, `Std.Net`, and `Std.Console`, still
  require explicit imports and follow AP05.
- **Preserve all distinct operations.** The base catalog is not a deletion
  boundary. Keep existing additional operations as type operations, retiring
  only verified synonymous copies in favor of one canonical name. The
  complete inventory and implementation mappings, argument roles, result
  types, and required differences are in [catalog.md](catalog.md).
  `Join` belongs to `array of string`; `Fill` is an array factory and `Chr`
  a string factory. Existing operations keep their FPAS semantics,
  including eager collection processing and Unicode scalar indexes.
  `Sort`, `Merge`, and `Remove` keep returning values without changing the
  caller variable. AP06.3 adds `IsEmpty` for strings,
  arrays, and dictionaries. Scalars, channels, and task handles receive
  instance operations only after a concrete need and an explicit extension
  decision. Existing `Push` and `Pop` remain array operations; AP06.1 settles
  receiver mutation marking, with AP17.3 delivering the `var` rules.
- **No automatic free-function lookup.** Dot notation no longer searches
  visible free functions, procedures, or callable values by their first
  parameter. User-defined free functions and all other routines are called
  with ordinary call syntax, nested or through intermediate bindings.
- **Receiver passing stays.** The left value is passed as `Self` to a method,
  or as the first input argument of a catalog operation. The receiver is
  evaluated once, before the written arguments, which keep their order.
- **Unchanged from Q05:** no pipe operator. Dot notation remains available for
  actual record members (fields, callable fields, methods).

Draft, using agreed AP16 bindings:

```pascal
function WordLength(Word: string): integer;
begin
  return Word.Length();
end function;

const Input: string := '  hello world  ';
const Words: array of string := Input.Trim().Split(' ');
const Sizes: array of integer := Words.Map(WordLength);
const Next: Point := Origin.Offset(1.0, 2.0).Normalize();  // record methods
const Clean: string := Normalize(Input);                    // own free function
const Bad: string := Input.Normalize();   // error: 'Normalize' is not a method
                                          // or standard operation of string
```

## Standard-operation naming (agreed)

The catalog uses a common vocabulary across receiver types. These rules apply
to catalog operations; user-defined record methods retain their declared names.

- **Same operation, same name.** Equivalent operations use the same dot name
  on every receiver type that supports them. `Length()` is the canonical count
  operation for strings, arrays, and dictionaries: Unicode scalar count,
  element count, and entry count respectively. Shared operations such as `Map`,
  `Filter`, and `Reduce` follow the same rule.
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

These must be decided explicitly with the user in AP06.1, before any
implementation work package starts:

1. **Extended catalog names and factory forms.** Validate the complete
   inventory against the naming rules and settle any equivalent operation
   names across types, such as string `Substring` and array `Slice`. Record
   one type-qualified spelling for `Chr` and generic array `Fill` and their
   type inference. Preservation is agreed; their final public spelling must
   be explicit before implementation.
2. **Name conflicts.** How a catalog name relates to a local or imported free
   function with the same name; resolution of overloaded standard routines;
   a record field holding a callable value with the same name as a method.
   The shared naming rules above are already agreed.
3. **Mutation marking.** How AP17's call-site `var` marking applies to the
   receiver of `Push`, `Pop`, and other caller-mutating type operations.
   Keeping these operations and one public call form is already agreed.

Catalog scope, the base names, preservation of additional distinct operations,
automatic availability, and one public call form are agreed in
[catalog.md](catalog.md). The inventory validates those decisions.

## Dependencies

None for built-in type operation availability. AP05 continues to define imports
for other units; native type operations do not depend on import aliases.

AP12 and AP14 depend on this package.

## Order

AP06.1 validates the agreed catalog and records the remaining decisions.
AP06.2 prepares consumer migration using forms accepted by the current compiler.
AP06.3 adds native type operations, `IsEmpty`, and the agreed factory forms,
finishes migrations requiring the new syntax, and removes the old public units
and their free-call API in the same delivery.

## Work packages

- [ ] [AP06.1: Catalog names and remaining operation rules](01-catalog-and-rules-decision.md)
- [ ] [AP06.2: Prepare type-operation and free-call migration](02-migrate-non-catalog-calls.md)
- [ ] [AP06.3: Implement native operations and remove duplicate call forms](03-fixed-dot-resolution.md)

## Acceptance

An imported or local free function cannot reinterpret a dot call. Record-method
calls and catalog operations retain their meaning and allow type-changing
chains. Every former free-function receiver call is migrated to an ordinary
call with equivalent behavior when its target is not a native type operation.
Calls to the five former type-helper units migrate to their one canonical
type-operation form. Equivalent catalog operations use the same
canonical name and comparable argument roles across receiver types; automatic
checks and tests enforce the agreed naming rules.
The five types expose all agreed distinct operations automatically, including
`IsEmpty` for strings, arrays, and dictionaries. No existing distinct behavior
is dropped, and the old units provide no parallel public path. Other units
retain explicit imports. Existing operation semantics remain unchanged.

## Reference

The reference branch `codex/syntax-changes` removed record methods and all
receiver calls. That direction is superseded by this decision and is not
adopted. Its owner map remains useful: sema `check/expr/calls/fluent.rs`,
`calls/methods.rs`, `check/expr/bound_method.rs`, and compiler
`lowering/calls/fluent.rs`.

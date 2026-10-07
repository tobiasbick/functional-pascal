# AP06.3: Implement native type operations and remove duplicate call forms

Package: [AP06: Fixed dot-call targets](README.md)

## Scope

Replace the automatic receiver lookup with fixed resolution: record members
first, then the entry in the [complete operation catalog](catalog.md) for the
receiver's static type. Remove the selection of free routines and callable
values by receiver. Make native operations automatically available, preserve
all existing distinct operations, add `IsEmpty` and the agreed factory forms,
and remove the five former public type-helper units and their duplicate calls.

## Prerequisites

- AP06.2 (migration preparations and explicit list of remaining conversions).
- AP06.1 (complete names, factory forms, conflicts, and the recorded outcome
  of the user-requested implicit-receiver follow-up discussion).

## Implementation

- Sema: for a record receiver, resolve declared members only. For `string`,
  arrays, dictionaries, `Option`, and `Result`, resolve the catalog entry for
  the static receiver type and name. Otherwise report an error. Reuse
  existing generic inference and type constraints for container receivers.
- Represent the catalog once, next to the standard-library registry, and use
  the same data for checking, lowering, completion, and documentation tests.
- Resolve native operations directly by static type, independently of imports
  or visible free routines. Implement the agreed type-qualified factory forms
  for `Chr` and `Fill`; expose `Join` on `array of string`.
- Preserve every additional distinct operation, including `ForEach`, variadic
  `Format`, `Push`, and `Pop`, and apply verified canonical replacements only
  where AP06.1 identifies genuine aliases or equivalent names across types.
- Add `IsEmpty` for strings, arrays, and dictionaries as native operations.
  Each returns whether its receiver's `Length` is zero without mutation.
  Reuse existing length handling and update required implementation layers.
- Remove `Std.Str`, `Std.Arrays`, `Std.Dictionaries`, `Std.Options`, and
  `Std.Results` from the public import/symbol surface. Reject their ordinary
  calls, aliases, and free function references with a canonical-operation hint.
  Existing internal implementation IDs may remain; they are not public units.
- Generate documentation and editor signatures from the native operation
  catalog. Remove public editor stubs for the five former units and adapt
  API export and language-service indexing to avoid reintroducing them.
- Keep all existing catalog operations' FPAS semantics, including eager
  collection processing, Unicode scalar indexes, dictionary insertion
  order, and value-returning array and dictionary operations.
- Add automatic catalog checks for the
  [agreed standard-operation naming rules](README.md#standard-operation-naming-agreed).
  Check shared operation names and corresponding argument roles across
  receiver types, reject synonymous entries for one operation, and cover
  documented type-required differences. Apply these checks to future catalog
  extensions as well.
- Remove the free-routine and callable-value receiver path and its
  first-parameter filtering.
- Pass the receiver as `Self` or as the first argument; evaluate it once,
  before the written arguments.
- Diagnostics: for a rejected dot call, name the receiver type and show the
  ordinary call form, for example `Normalize(Input)`; when a catalog entry
  with a similar name exists, mention it.
- Language service: completion after `.` lists fields, methods, and catalog
  operations only; signature help covers catalog operations.
- Named arguments: receiver calls, native type operations, and polymorphic
  standard-library operations take positional arguments only today (FP3026,
  AP09.1). Decide with the user whether catalog operations accept named
  explicit arguments (the receiver stays unnamed); if so, give each catalog
  entry parameter names and reuse `crates/fpas-sema/src/check/calls/named.rs`.
- `var` receivers: a receiver call of a free routine whose first parameter is
  `var` is rejected today (FP3027, AP17.1). Decide together with the
  implicit-receiver exception whether such calls stay rejected or use the
  writable-receiver checks of AP17.3.
- Apply the agreed implicit-receiver exception after the requested follow-up
  discussion: native mutating operations use ordinary dot calls without a
  receiver `var` marker or additional parentheses. Store the writable-receiver
  requirement in the catalog; reuse AP17's argument validity, aliasing,
  lifetime, `go`, evaluation, and failure checks. Explicitly written `var`
  arguments retain their markers. Coordinate delivery with AP17.3 and record
  the order in AP06.1; keep one public type-operation form.

## Affected areas

- `crates/fpas-sema/src/check/expr/calls/fluent.rs`, `calls/methods.rs`,
  `std_registry/`.
- `crates/fpas-compiler/src/lowering/calls/fluent.rs`.
- Standard-library registration, unit discovery/export, and the checking,
  lowering, and runtime layers needed for native operations and factories.
- API export and generated declarations under `lib/api/Std/`; remove the
  five public stubs and provide native signatures from the catalog.
- `crates/fpas-language-service/src/intellisense/` (completion, signature help).
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

Finish all conversions deferred by AP06.2: factory syntax, canonical names,
remaining ordinary calls and function references, imports and aliases, and
generated declarations. Remove the old public API in the same delivery.
Negative tests keep removed forms only as rejection cases.

## Documentation

- Rewrite `docs/pascal/language/functions/fluent-calls.md` as the dot-call page
  (methods and catalog), update `postfix-chaining.md`, `record-methods.md`,
  and document native operations on the corresponding type pages. Migrate
  the five former Std-unit references and API indexes to type documentation;
  include all preserved operations, factories, `IsEmpty`, automatic
  availability, and their one canonical public form.
- Update unit/import documentation to remove the five public units and retain
  explicit imports for the remaining standard library.
- `docs/specs/grammar.ebnf` if the postfix description changes.
- Diagnostics reference entry.

## Verification

- Record-method chains returning `Self` and new values.
- Every agreed catalog entry, including `Option` and `Result` operations.
  Cover type-changing chains such as string to array, array to `Option`,
  dictionary lookup to `Option`, and `Result` mapping and recovery.
- `IsEmpty` on empty and nonempty strings, arrays, and dictionaries, in
  native dot form without imports; a receiver with observable evaluation runs
  once. Test the same automatic availability for every catalog entry.
- All additional preserved operations: specialized `Join`, final-procedure
  `ForEach`, variadic `Format`, distinct `FromChar`/`RepeatStr` constraints,
  and the agreed type-qualified `Chr` and `Fill` forms.
- Existing operation semantics: eager callbacks, Unicode scalar lengths
  and indexes, dictionary entry order, and no caller mutation by `Sort`,
  `Merge`, or `Remove`.
- Consistent canonical names across receiver types: `Length()` on strings,
  arrays, and dictionaries, and shared `Map`, `Filter`, and `Reduce` operations
  included in the agreed catalog. Verify argument roles and type-required
  differences, and reject synonymous `Size()` or `Count()` dot forms for
  `Length()`. Catalog checks detect inconsistent names or argument order.
- Rejection of free-function and callable-value receiver calls, with hint.
- Rejection of imports, aliases, ordinary calls, and free function references
  for the five removed units. Completion/signature help exposes only native
  operations and their agreed factory forms for these types.
- Other Std units still require explicit imports. Added imports cannot alter
  a native operation's target or availability.
- Rejection of unlisted catalog names and catalog calls on scalars, channels,
  task handles, or an unconstrained generic receiver. Mutating entries use
  the agreed unmarked-receiver exception and reject `const` and temporary
  receivers. Cover writable direct, field, element, and forwarded receivers,
  and preservation of other array values that share storage. `Push` and
  `Pop` are retained with their existing return types and chaining behavior.
- A local or imported function named like a catalog operation behaves as the
  agreed conflict rules state; it never selects a different dot-call target.
- Receiver-before-argument evaluation order; `go` with a dot call.
- Completion and signature-help tests.

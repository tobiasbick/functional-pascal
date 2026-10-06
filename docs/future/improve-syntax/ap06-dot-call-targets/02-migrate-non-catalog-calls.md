# AP06.2: Prepare type-operation and free-call migration

Package: [AP06: Fixed dot-call targets](README.md)

## Scope

Prepare the migration to the [complete type operation catalog](catalog.md).
Rewrite unrelated free-routine receiver calls into ordinary calls. Rewrite
ordinary calls to the five type-helper units into their agreed receiver form
where the current compiler accepts it. This work package changes no language
rule; migrations needing native factory syntax or import removal finish in
AP06.3, together with removal of the old public API.

## Prerequisites

- AP06.1 (agreed catalog and rules).

## Implementation

- Find receiver calls through resolved symbols: calls that select a free
  routine or callable value not in the catalog.
- Find ordinary calls and free function references selecting operations of
  the five former units. Convert receiver-based calls to the canonical dot
  form while retaining the imports the current compiler still requires.
- Replace function references such as `Std.Str.Length` with existing named
  wrappers or anonymous functions that invoke the type operation.
- Rewrite unrelated free-routine chains to nested calls, for example
  `Source.Load().Validate().Publish()` to `Publish(Validate(Load(Source)))`,
  or to intermediate bindings where nesting hurts readability. Qualify names
  where needed. Native catalog chains keep their dot form.
- Preserve evaluation order: receiver first, then arguments left to right.
  Introduce an intermediate binding when a rewrite would otherwise reorder
  side effects.
- Leave record-method calls, field calls, and catalog calls unchanged.
- Preserve existing `Option` and `Result` chains and all additional distinct
  type operations. Do not treat an operation as outside the catalog merely
  because it was absent from the earlier base list.
- Record calls whose new canonical names or type-qualified factory forms
  need AP06.3, and all imports and API stubs removed by that delivery. Do not
  introduce unsupported syntax or remove required imports prematurely.
- The new `IsEmpty` operations are delivered in AP06.3; do not introduce them
  during this migration.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`.
- Rust-embedded fixtures, formatter goldens, CLI templates, editor snippets,
  documentation examples, and repository skills that teach receiver calls.

## Migration

This work package is the migration. A temporary semantic report on the branch
lists remaining unrelated receiver calls and calls/imports/references whose
conversion requires AP06.3. It is not merged.

## Documentation

Examples adopt ordinary calls for unrelated routines and dot calls for type
operations where the current compiler supports them. The language rules in
`docs/pascal/language/functions/fluent-calls.md` stay unchanged until AP06.3.

## Verification

- The temporary report shows zero non-catalog receiver calls.
- Every call/import/reference requiring AP06.3 is explicitly recorded for
  its atomic removal with the old public API. Distinct behavior is preserved.
- `fpas test tests/suite.fpasprj`, example/app checks, and the workspace tests
  pass with unchanged results.

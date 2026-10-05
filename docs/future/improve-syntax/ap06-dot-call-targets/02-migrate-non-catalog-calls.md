# AP06.2: Migrate non-catalog receiver calls

Package: [AP06: Fixed dot-call targets](README.md)

## Scope

Rewrite every receiver call whose target is neither a record method nor an
agreed catalog operation into an ordinary call. The current language accepts
both forms, so this work package changes no language rule.

## Prerequisites

- AP06.1 (agreed catalog and rules).

## Implementation

- Find receiver calls through resolved symbols: calls that select a free
  routine or callable value not in the catalog.
- Rewrite chains to nested calls, for example
  `Reduce(Map(Filter(Values, IsEven), Double), 0, Sum)`, or to intermediate
  bindings where nesting hurts readability. Qualify names where needed.
- Preserve evaluation order: receiver first, then arguments left to right.
  Introduce an intermediate binding when a rewrite would otherwise reorder
  side effects.
- Leave record-method calls, field calls, and catalog calls unchanged.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`.
- Rust-embedded fixtures, formatter goldens, CLI templates, editor snippets,
  documentation examples, and repository skills that teach receiver calls.

## Migration

This work package is the migration. A temporary semantic report on the branch
lists remaining non-catalog receiver calls; it is not merged.

## Documentation

Documentation examples move to ordinary calls. The language rules in
`docs/pascal/language/functions/fluent-calls.md` stay unchanged until AP06.3.

## Verification

- The temporary report shows zero non-catalog receiver calls.
- `fpas test tests/suite.fpasprj`, example/app checks, and the workspace tests
  pass with unchanged results.

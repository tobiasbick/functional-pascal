# AP16.2: Migrate immutable var to const

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Rewrite every immutable `var` binding to `const`. After this work package,
`var` without `mutable` no longer appears in repository sources except in
negative tests. Language rules are unchanged.

## Prerequisites

- AP16.1 (computed `const`).
- AP11.2 (one keyword per declaration).

## Implementation

- Use semantic information to find each immutable `var` binding at every
  level and rewrite it to `const` with the same type and initializer.
- Do not touch `mutable var` or parameters.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`; generated
  `lib/api/` declarations and their generator; Rust-embedded fixtures;
  templates; snippets; documentation examples; skills.

## Migration

This work package is the migration.

## Documentation

Documentation examples use `const` for immutable bindings. The `var` rule on
`variables.md` is unchanged until AP16.3.

## Verification

- A temporary report finds no immutable `var` outside negative tests.
- Full FPAS suite, example/app checks, and workspace tests pass unchanged.

# AP16.2: Migrate immutable var to const

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Rewrite every immutable `var` binding to `const`. After this work package,
`var` without `mutable` no longer appears in repository consumers except in
negative tests, focused tests of the still-valid immutable `var` syntax, and
the current language reference for that syntax. Language rules are unchanged.

## Prerequisites

- AP16.1 (computed `const`).
- AP11.2 (one keyword per declaration).

## Implementation

- Use semantic information to find each immutable `var` binding at every
  level and rewrite it to `const` with the same type and initializer.
- Do not touch `mutable var` or parameters.
- Preserve scalar guard bindings. A bare identifier in `when Name if ...`
  can currently introduce a fresh binding even if it shadows an immutable
  `var`. After that outer declaration becomes `const`, the same label would
  resolve as a value label. Use the existing semantic scalar-binding metadata
  to identify affected arms; rename the arm binding and its resolved uses to
  a fresh name without changing nested bindings or unrelated members.
- Keep focused lexer, parser, semantic, formatter, and editor tests that
  specifically verify the immutable `var` form. Record these exceptions in
  the temporary migration report; ordinary fixtures use `const`.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`; generated
  `lib/api/` declarations and their generator; Rust-embedded fixtures;
  templates; snippets; documentation examples; skills.

## Migration

This work package is the migration.

## Documentation

Documentation examples use `const` for immutable bindings. The `var` rule on
`variables.md`, the formal grammar, keyword list, and focused syntax-reference
examples remain accurate for the still-valid immutable `var` form until
AP16.3. Authoring guidance and snippets prefer `const`.

## Verification

- A temporary report finds no immutable `var` outside the explicitly listed
  syntax-reference and test exceptions. Inspect generated declarations and
  their generator, Rust fixtures, templates, snippets, and skills as well as
  `.fpas` sources.
- Migrated guard bindings retain the same scope and matched value. Preserve
  initializer reachability, call counts, ordering, captures, and mutability;
  include regression coverage for a guard binding shadowing a migrated name.
- Full FPAS suite, example/app checks, and workspace tests pass unchanged.

## Result

Ordinary source consumers, embedded fixtures, documentation examples, source
generators, and editor fixtures use `const`. Authoring guidance prefers
`const`, and the editor has an immutable-binding snippet. The intrinsic API
generator already emits `public const`; regeneration preserves its declaration
surface.

The temporary AST-based migration and semantic scalar-guard comparison found
no existing arm whose binding classification changed. Regression coverage
compares the old shadowing form with a fresh-name migration for static and
computed initializers, including captures, nested shadowing, and same-named
record fields. Initialization and capture regressions from AP16.1 remain in
the suites.

At AP16.2 delivery, the remaining immutable `var` occurrences were explicitly
recorded syntax-reference and test exceptions. AP16.3 has completed their final
keyword transition. AP16.2 itself changed no binding or parameter mutability
rule.

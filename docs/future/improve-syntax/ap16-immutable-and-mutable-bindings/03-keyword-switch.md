# AP16.3: Keyword switch to const and var

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Make `var` the reassignable binding and remove the `mutable` keyword from
bindings and parameters. AP16.3 owns the complete keyword switch; AP17.1
subsequently introduces true reference parameters. Old `mutable` parameters
become read-only parameters with a local `var` copy wherever their writable
binding or shared capture cell is needed.

## Prerequisites

- AP16.2 (ordinary immutable `var` consumers have migrated; only explicitly
  recorded syntax-reference and test exceptions remain).

## Implementation

- Parser and sema: `var` declares a reassignable binding; `mutable var` and
  `mutable` parameters are rejected with a diagnostic showing `var` and the
  local-copy form respectively; assignment to a `const` suggests `var`.
- `for` loop variables are immutable per iteration without a keyword;
  assignment to them is an error. Verify current behavior first.
- Closures: a captured `var` shares one mutable cell, as `mutable var` did.
- Caller-mutating intrinsics such as `Push` and `Pop` keep their current
  special rule, which now requires a `var` binding instead of a `mutable var`;
  AP17.3 replaces that rule.
- Lexer: remove `mutable`; it becomes an identifier.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser declarations and
  parameters, `crates/fpas-sema/src/check/decl/vars.rs`, routine parameter
  checking, closure capture, editor highlighting and snippets.

## Migration

- Rewrite `mutable var` to `var` everywhere.
- For each `mutable` parameter, remove the modifier and introduce a local
  `var` copy at the start of the body when needed for direct reassignment,
  field or element writes, caller-mutating intrinsics such as `Push`/`Pop`,
  or captures by closures or nested named routines, including read-only
  captures. Preserve the shared cell and task-bound capture behavior. Never
  turn local changes into caller mutation.
- Give the copy a fresh name and rewrite only references resolved to that
  parameter. Preserve shadowing, same-named fields, and signature names.
  In particular, `Std.Http.Sse.EmitEvent` needs a copy for its `Push` calls.
- All repository consumers, including generated declarations, fixtures, and
  documentation examples.
- Replace AP16.2's remaining positive immutable-`var` fixtures and reference
  examples with `const`, or update focused parser/formatter fixtures to test
  the new reassignable `var` meaning. Adapt negative tests to the new rule.
  Do not silently turn a retained immutable binding into a mutable one.

## Documentation

- `docs/pascal/language/basics/variables.md`, `local-variables.md`,
  `docs/pascal/language/functions/mutable-parameters.md` (remove, or replace by
  the read-only parameter rule on `parameters.md`), `closures.md`, keyword
  list, `docs/specs/grammar.ebnf`, authoring skill.

## Verification

- Reassignment of `var`, rejection for `const`, loop variables, captures of
  `const` and `var`, shared values, removed `mutable` with hints.
- Each migrated binding and parameter keeps its mutability and behavior.
- Full FPAS suite and example/app checks.

## Result

Delivered. `var` is writable at program, unit, and local scope; `const`, value
parameters, and loop variables are read-only. Captured local variables retain
shared mutable cells. `mutable` is an ordinary identifier; retired declaration
prefixes produce contextual migration hints. Parser, semantic and persisted
parameter types no longer carry the removed modifier. Compiled-unit and object
formats are version 8 so old artifacts are rebuilt.

Repository sources, embedded fixtures, generated intrinsic declarations and
their generator, formatter, language service, snippets, highlighting, authoring
guidance, and language references use the new forms. Parameter copies preserve
field/element writes, array intrinsics, and capture behavior; renamed references
preserve shadowing and member names. Qualified exported variable writes and
read-only exported constants have interface round-trip and linking coverage.
The old mutable-parameter reference page is replaced by the local-copy rule in
`parameters.md`.

Verification: Rust formatting and workspace build, FPAS formatting, all 22
application/example project checks, intrinsic API regeneration, editor compile,
grammar, contracts, manifest checks, and relative documentation links passed.
The FPAS suite passed 476 tests with one skip. The workspace run passed 3,595
Rust tests; its two previously recorded TCP failures remain. An unchanged LSP
initialization/shutdown fixture hung and was terminated; a separate complete
initialization, capability-registration acknowledgement, shutdown, and exit
protocol check passed. All AP16.3 regression tests passed.

AP16 is complete. AP17.1 has since added true reference parameters; the
existing simple writable-array target exception for `Push`/`Pop` remains until
AP17.3.

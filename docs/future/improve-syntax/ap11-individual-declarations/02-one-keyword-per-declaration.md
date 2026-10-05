# AP11.2: One keyword per declaration

Package: [AP11: Individual declarations](README.md)

## Scope

Require a separate `type`, `const`, or `var` keyword for each declaration and
remove declaration groups.

## Prerequisites

- AP11.1 (order-independent types).

## Implementation

- Parser: one declaration per keyword at unit, program, and routine level, and
  in `mutable var` declarations while they exist. A second declaration without
  a keyword gets a diagnostic that shows the repeated keyword.
- Formatter: emit one declaration per keyword; remove group emission.

## Affected areas

- `crates/fpas-parser/src/parser/decl/data/const_var.rs`, `decl/data/type_defs.rs`.
- `crates/fpas-fmt/src/emit/decl/`.
- Editor snippets and CLI templates.

## Migration

- Split every declaration group in `lib/`, `apps/`, `examples/`, `tests/`,
  Rust-embedded fixtures, generated `lib/api/` declarations and their
  generator, templates, snippets, and documentation examples.
- Preserve exported names, scopes, and visibility.

## Documentation

- `docs/specs/grammar.ebnf` (`const_block`, `var_block`, `type_block`,
  `mutable_var_block`), `docs/pascal/language/basics/constants.md`,
  `variables.md`, `local-variables.md`, `docs/pascal/tools/fmt-style.md`,
  FPAS authoring skill.

## Verification

- Parser tests for each declaration kind and the group diagnostic.
- Formatter round trip and idempotence on the whole corpus.
- Full FPAS suite and example/app checks.

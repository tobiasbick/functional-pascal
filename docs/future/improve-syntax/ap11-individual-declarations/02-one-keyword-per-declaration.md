# AP11.2: One keyword per declaration

Package: [AP11: Individual declarations](README.md)

Status: complete on `codex/syntax-changes-2`.

## Scope

Require a separate `type`, `const`, or `var` keyword for each declaration and
remove declaration groups.

## Prerequisites

- AP11.1 (order-independent types).

## Implementation

- Parser: one declaration per keyword in existing declaration positions.
  Program and unit declarations support types, constants, and variables;
  routine statement lists supported variable bindings at AP11 delivery. AP16.1
  subsequently added local `const` bindings; local types remain unsupported.
  A second declaration without a keyword gets FP2015, pointing at its name
  and showing the complete repeated prefix.
- Formatter: emit one declaration per keyword; remove group emission.
- At AP11 delivery, repeat the complete `mutable var` form per mutable
  declaration. AP16.3 has replaced it with writable `var`. Repeat `public`
  before each exported declaration; visibility is not inherited from an
  earlier declaration.

## Affected areas

- `crates/fpas-parser/src/parser/decl/data/{const_var,type_defs,recovery}.rs`
  and `parser/stmt/terminators.rs`.
- `crates/fpas-fmt/src/emit/decl/`.
- `crates/fpas-cli/version_sync.rs` and
  `crates/fpas-sema/examples/export_intrinsic_std_api.rs`.
- Editor grammar and fixtures; existing editor snippets and CLI templates
  already emit individual declaration keywords.

## Migration

- Split every declaration group in `lib/`, `apps/`, `examples/`, `tests/`,
  Rust-embedded fixtures, generated `lib/api/` declarations and their
  generator, templates, snippets, and documentation examples.
- Preserve exported names, scopes, and visibility.
- Repeat a public group's `public` modifier for every declaration it contains,
  and keep each declaration's original position and initialization order.

## Documentation

- `docs/specs/grammar.ebnf` (`const_declaration`, `var_declaration`,
  `type_declaration`; the former `mutable_var_declaration` was removed by
  AP16.3), `docs/pascal/language/basics/constants.md`,
  `variables.md`, `local-variables.md`, `docs/pascal/tools/fmt-style.md`,
  FPAS authoring skill.

## Verification

- Parser tests for each declaration kind and the group diagnostic.
- Verify repeated `public` and `mutable var`, preserved export visibility,
  and unchanged value initialization order when splitting groups.
- Formatter round trip and idempotence on the whole corpus.
- Full FPAS suite and example/app checks.

## Completion

- [x] Require one keyword per declaration and diagnose removed groups with FP2015.
- [x] Emit individual declarations; remove the formatter's group emitter.
- [x] Migrate source groups, Rust fixtures, generated APIs, editor fixtures,
  and current documentation examples without changing declaration order or exports.
- [x] Regenerate intrinsic editor APIs from an individual-declaration generator;
  repeated generation is identical.
- [x] Update grammar, language/visibility/formatter documentation, and authoring rules.
- [x] Cover keyword repetition, missing-keyword recovery, visibility, mutability,
  local scope, unit imports/sidecar reuse, and initialization effects.
- [x] Pass formatter round trip and idempotence across `lib`, `apps`, `examples`,
  and `tests`, plus the parser corpus and formatter goldens.
- [x] Pass workspace build, Rust formatting, workspace tests, FPAS formatting,
  the full FPAS suite, all example/app manifest checks, and editor grammar/contracts.

The workspace test run excludes the three previously established unrelated
failures: LSP watcher registration, socket write deadline retries, and TCP
backpressure cancellation. The FPAS suite has one regular skipped test.

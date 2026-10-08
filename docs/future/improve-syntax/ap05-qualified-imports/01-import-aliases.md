# AP05.1: Import aliases

Package: [AP05: Qualified imports](README.md)

Status: complete.

## Result

`uses Unit as Alias;` exposes the unit's public symbols only through
`Alias.Name`. It opens no short names and hides that import's original unit
path. Plain imports keep short and fully qualified names; ambiguous short
names remain errors at the point of use.

`as` is contextual immediately after a unit name in `uses`, recognized
case-insensitively without a globally reserved lexer keyword. Identifiers named
`As` remain valid elsewhere, including qualified unit segments and aliases.

## Import and collision rules

Each unit occurs at most once in a file's direct imports, comparing canonical
unit names case-insensitively. Repeated plain imports, multiple aliases, and
plain/aliased combinations are rejected with FP3002 at the repeated entry.
Shared transitive dependencies are allowed.

Aliases cannot collide with each other, primitive type names, imported or
owning unit namespace roots, or declarations in the importing file. Local
bindings, routine and generic parameters, loop variables, and pattern bindings
cannot shadow an alias. FP3002 identifies the conflict and recommends renaming
the declaration or alias. Hidden original paths receive an alias-specific
correction hint.

An alias names one unit; it does not expose sibling, nested, private, or
transitive supporting units. Aliases are local to their source file and are
not re-exported. Unit resolution, library export checks, nominal type identities,
interfaces, linked symbols, and sidecar dependencies keep canonical unit names.

## Implementation ownership

- `crates/fpas-parser/src/ast/imports.rs` and `parser/imports.rs` retain unit
  identities, written aliases, and exact source spans.
- `crates/fpas-sema/src/check/imports.rs`, `scope/imports.rs`, and
  `interface/install.rs` enforce visibility, uniqueness, and namespace rules.
- `crates/fpas-project/src/unit_graph/` resolves canonical dependencies and
  assigns diagnostic source IDs to import and alias spans.
- `crates/fpas-compiler/src/lowering/imports/names.rs` maps source aliases onto
  existing canonical callable, global, constant, and type bindings.
  Qualified routine values, task targets, and collection/record receivers use
  those bindings without changing linked identities.
- `crates/fpas-fmt/src/emit/program.rs` emits aliases with the normal wrapping
  and comment rules. Language-service AST consumers use the new import node;
  alias-aware editor behavior belongs to [AP05.2](02-alias-aware-tooling.md).

## Regression coverage

- `crates/fpas-parser/src/tests/decl/import_aliases.rs`: contextual syntax,
  casing, alias spans, ordinary `As` identifiers, and malformed-import recovery.
- `crates/fpas-sema/src/interface/tests/import_aliases.rs`: alias-only public
  names, records, enums, constants, variables, routines, canonical exported
  types, repeated imports, namespace collisions, and transitive visibility.
- `crates/fpas-project/tests/unit_graph.rs` and `project_integrity.rs`:
  canonical dependency ordering, shared dependencies, source IDs, and library
  export restrictions through aliases.
- `crates/fpas-build/tests/import_aliases.rs`: compiled-unit calls, named and
  `var` arguments, native mutations, static/bound record methods, enum
  construction, routine values, tasks, colliding helpers, and dependency
  sidecar reuse after a program alias is renamed.
- `crates/fpas-cli/src/main_tests/projects/edge_cases/unit_names.rs`: direct
  duplicate imports in plain, aliased, mixed, and case-varied forms emit FP3002.
- `crates/fpas-fmt/src/emit/program.rs`: casing, wrapping, comment preservation,
  and idempotence.
- `tests/stdlib/imports/aliases_test.fpas`: intrinsic aliases, constants,
  task calls, standard enum values, and import-independent native operations.

## Documentation

- [Units](../../../pascal/program-structure/units.md#import-aliases).
- [Grammar](../../../specs/grammar.ebnf) (`uses_clause`, `import_entry`).
- [Diagnostics](../../../pascal/tools/diagnostics.md),
  [formatter style](../../../pascal/tools/fmt-style.md), and FPAS authoring and
  project skills.

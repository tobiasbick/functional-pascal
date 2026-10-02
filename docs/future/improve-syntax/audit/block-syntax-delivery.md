# Stage 3 block syntax and names delivery

This records the first implementation item in
[stage 3](../stages/03-syntax-and-names.md). The preceding
[inventory](syntax-and-names.md) describes the pre-change checkout.

## Implemented behavior

- Programs, units, named and anonymous routines, records, enums, conditionals,
  cases, loops, and record updates use their documented closers. Plain lexical
  blocks keep `begin ... end;`; repeat loops close with `until Condition;`.
- Every statement requires its final `;`. Empty statement bodies require
  `null;`; bare empty statements are rejected. Anonymous routine expressions
  leave argument commas and parentheses to the surrounding expression.
- `if`, `elsif`, `else`, `when`, and case `else` bodies contain statement lists.
  Each conditional branch and case arm has its own declaration scope. Explicit
  plain blocks retain their additional nested scope. `else if` remains a nested
  conditional with its own closer.
- Each declaration repeats `type`, `const`, or `var`, including exported
  declarations. Compilation-unit type headers resolve before signatures and
  type bodies; value initializers and record field defaults remain ordered.
- Each import declares one explicit alias. Only that alias exposes the imported
  unit, without short names, full source paths, or implicit access to nested or
  transitive units. Aliases are case-insensitive and reserved in every lexical
  scope. Duplicate units, alias collisions, and type/routine collisions fail.
- The reserved `null` keyword replaces the JSON variant spelling with
  `JsonValue.NullValue`. Intrinsic registration, runtime values, generated API
  declarations, examples, tests, and current documentation use the new name.

Parser recovery reports concrete replacement syntax and preserves outer named
closers. Alias diagnostics retain source locations, including imported types
and nested calls, and report an invalid unit access once per source path.

## Implementation and consumer boundaries

New focused modules own parser block boundaries
(`fpas-parser/src/parser/blocks.rs`), semantic import resolution and type-header
collection (`fpas-sema/src/check/name_resolution/imports.rs` and
`check/decl/types/collection.rs`), and compiler qualification, designators, and
globals (`fpas-compiler/src/lowering/context/{imports,designators,globals}.rs`).
Declaration-list emission replaces the formatter's former grouped-declaration
emission. The intrinsic declaration exporter separates rendering and
qualification, then regenerates `lib/api/Std/` from its producer.

Compiler and linker identities remain canonical unit names. Source aliases map
to those identities for types, constants, variables, callable values, enum
facades, member access, mutation, and task calls. Aliased intrinsic tasks retain
their argument overloads, formatting argument count, and `Result` return shape.
Debugger expressions continue to use their compiled symbol catalog.

The migration includes `lib/`, `examples/`, `tests/`, `apps/`, formatter goldens,
Rust and TypeScript source fixtures, CLI scaffolds, benchmark source generators,
editor snippets, highlighting, indentation, completion, navigation, semantic
tokens, auto-import edits, and rename-conflict checks. Auto-import qualifies the
reference, inserts the alias declaration, and formats the result as one edit.
There is no compatibility mode or retained migration tool in production code.

Current specification owners are `docs/specs/grammar.ebnf`, the corresponding
language and unit pages under `docs/pascal/`, and
`docs/pascal/tools/fmt-style.md`. Standard-library examples and FPAS authoring
guidance use the implemented syntax.

## Regression coverage

The new `syntax_and_names` parser, semantic, compiler, and formatter integration
tests cover positive, negative, and edge cases. They include all currently
implemented block-table rows, malformed and truncated closers, mandatory and
extra semicolons, explicit empty bodies, nested branch ownership, argument
boundaries, comments, case variants, declaration ordering, forward and recursive
types, alias cycles, every lexical alias-collision category, and visibility.

Execution tests check condition evaluation count and order, returns through
`elsif`, loop bodies, case-arm scope, record-copy updates, anonymous calls,
aliased intrinsics, and task result shapes. CLI project tests cover enum
re-exports, qualified globals and callable values, field/index mutation,
cross-unit linking, and rejection of undeclared source access paths.

Formatter tests compare normalized AST structure and second-format identity.
Editor tests cover alias replacement spans inside nested arguments, Unicode
diagnostic locations, completion after named closers, reserved-alias rename
conflicts, unsaved-buffer formatting, highlighting, snippets, and auto-imports.
Generated project fixtures cover both zero and multiple imported units.

## Verification

The final verification commands are:

```text
cargo fmt
cargo build
cargo test --workspace --no-fail-fast
fpas fmt --check examples tests apps lib
fpas test tests/suite.fpasprj
node editors/vscode/scripts/run-tests.mjs
git diff --check
```

All commands pass. The workspace suite passes, and the additional CLI test
compiles the unit/program examples directly from `units.md`. The real VS Code
extension-host suite passes diagnostics, formatting, navigation, IntelliSense,
semantic tools, project workflows, debugger, and lifecycle checks.

Additional source checks cover all 109 example/app programs: 95 pass standalone
checks; the remaining 14 require their project dependencies. All 22 example/app
projects pass `fpas check`. Of 52 complete handbook programs, 49 pass standalone
checks; three project-dependent examples are covered by CLI project integration
tests. The FPAS suite passes 458 tests with one intentional skip and no failures.

## Remaining stage work

A separate existing compiler issue rejects a program and routine sharing the
same name with `F9001` / `DuplicateName`. The handbook's greeting example uses
`GreetingDemo` as its program name; this delivery does not change that unrelated
compiler naming behavior.

Operator precedence, logical evaluation restrictions, and `Std.Bits`
replacements remain the next open implementation item. Future expressions,
purity, mutation replacement, and task scopes belong to their owning later
stages. This delivery does not complete stage 3 or those later constructs.

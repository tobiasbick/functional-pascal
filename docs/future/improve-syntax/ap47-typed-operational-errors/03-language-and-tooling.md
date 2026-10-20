# AP47.3: Language, compiler and editor support

Status: Complete. Depends on completed AP47.2.
Contract: [AP47.1](01-decision.md). Completion: [package README](README.md).

## Deliverable

Deliver a vertical language slice: a program or separately compiled source unit
can declare errors, construct/match them, use Error-constrained generic routines
and print them or obtain Message(). Native Std functions retain their old Result
signatures until their migration packages. Constructor routine values and
MapError are added by AP47.4, not assumed available here.

## Selected work

- Reuse the existing Error token contextually. Add error declarations and
  `end error` recovery to the AST/parser, including generic declarations and
  optional literal labels. Parse errors do not change Error(value) expressions.
- Extend nominal semantic types with a family discriminator, literal messages
  and checked Cause slots. Share enum registration, aliases, qualification,
  substitutions and finite-recursion logic. Add Error to built-in constraints
  and its implication to Printable without changing unrelated constraints.
- Reuse enum construction and closed-pattern coverage. Reject backing integers,
  payload defaults, unsupported declarations, incompatible nominal assignments,
  a bare root Error type and incorrectly typed Cause fields. Diagnostics identify
  the source declaration and show the selected canonical replacement.
- Add one native receiver entry for Message() restricted to concrete error
  families or Error-constrained parameters. Register its checked return type,
  read-only behavior, intrinsic ID, lowering and runtime route consistently.
- Emit the essential metadata from AP47.2 for generic specializations, public
  interfaces, imported aliases and source-less cached-unit consumers. For errors,
  intern_enum/reserve_enum must key fully applied semantic instances rather than
  replace them with the generic declaration template. Emit schema references and
  constructor requirements for unresolved generic construction using AP47.2's
  hidden readonly factory parameters; propagate them through calls and callable
  bindings. Reuse ordinary function calls and MakeEnum in the concrete factories.
  This internal support ships here; source-level constructor values still wait
  for AP47.4. Preserve erasure for ordinary enums and unrelated routine bodies.
  Generated constructor/inspection display names use the defining qualified name,
  not an import alias, instance key or executable-local slot. Preserve debug/source
  projections and reload compatibility checks when instance/family/variant/message
  metadata or hidden construction requirements differ.
- Keep runtime errors, panic and contract violations on their current path.
  Errors are ordinary values, not exception handlers or catchable panic objects.

## Actual owners and planned edits

The source slice reuses `TypeBody::Enum` with a native-error discriminator in
`ast/decl.rs` and `types/enums.rs`, so pattern checking and navigation retain
one owner. New focused modules under semantic type checking, compiler lowering
and object emission own payload admission, semantic identities and hidden
constructor transport. The existing native-operation catalog and intrinsic
dispatch own Message() and a private, pure generic-pattern query. Source regressions live under `tests/stdlib/result/` and
compiler artifact tests; formatter, public-signature and navigation fixtures
extend their existing owners. This continuation uses the current AP47 campaign
branch with its completed runtime prerequisite.

| Path | Responsibility |
| --- | --- |
| `crates/fpas-lexer/src/`, `crates/fpas-parser/src/ast/`, `src/parser/` | Contextual token use, declarations, recovery and spans |
| `crates/fpas-sema/src/types/`, `src/check/decl/types/`, `src/check/expr/` | Nominal family/constraint checking, construction and patterns |
| `crates/fpas-sema/src/std_registry/native/`, `src/std_registry/builtins/` | Message() catalog, inference and checked receiver behavior |
| `crates/fpas-ir/src/function.rs`, `src/program.rs`, `src/validate/` | Hidden construction parameters and schema references through checked IR/call validation |
| `crates/fpas-compiler/src/lowering/types/layouts.rs`, `src/lowering/calls/`, `src/bytecode/`, `src/intrinsic_catalog.rs` | Error instance interning, construction requirements, factory calls and metadata lowering |
| `crates/fpas-bytecode/src/intrinsic/`, `crates/fpas-std/src/` | One Message intrinsic and renderer reuse |
| `crates/fpas-fmt/src/` | Canonical error blocks, messages and layout |
| `crates/fpas-language-service/src/`, `crates/fpas-lsp/src/` | Hover, completion, public signatures and diagnostics |
| `editors/vscode/` | Existing Error highlighting plus declaration terminators and examples |
| `docs/specs/grammar.ebnf`, `docs/pascal/` | Implemented syntax, constraints, pattern and output rules |
| `tests/`, `examples/pascal/` | Language-level owner regression and one small complete example |

Expose the same nominal kind to source queries, definition/refs/type queries and
rename. Variant fields, Cause, generic constraints and message literals must not
create fake symbol references. Preserve incomplete/unsaved-source behavior. Do
not add a separate error navigation service or duplicate an enum's code path.
For editor/source-query symbol classification reuse the existing Enum and
EnumMember categories; declaration/type display shows the native error spelling.
Keep existing query JSON field names and coverage/limit semantics. Generated
constructor bodies create no extra user-visible definition/reference occurrence.

## Migration and documentation

No existing Error constructor, ordinary enum, Result/Option or Std signature is
removed here. Add the current error-type handbook page only with the executable
feature; link it from types/error handling and update grammar and generic rules.
Update the compact agent reference with a complete fence and extend its existing
executable-example verification. Record Message() beside native type operations.
Preserve strict `%s` formatting: it still takes a string, so use Cause.Message()
when building formatted text. Printable does not imply implicit string conversion.

## Acceptance and verification

Primary source-level tests exercise a custom error with a literal message,
optional label absence, fields, causes and an Error-constrained generic wrapper.
They prove exact output, exhaustive nested matching, nominal rejection, normal
equality and rejected ordering. Table-driven negative checks cover malformed
terminators, illegal payload/backing/default syntax and all Cause/constraint
errors. A simple fixture is source-only; a separate fixture owns import/cache
transport and stripped-image execution risks. That fixture calls one erased
generic wrapper with two concrete cause families, including through an imported
generic helper and a typed callable. Verify distinct instance IDs, alias reuse,
typed matching and exact Cause checks after linking without dependency source.
Include a private constructing helper behind an exported generic, recursion with
a stable instance key and a parameter used only in an ordinary data payload.
Include fieldless generic errors, task callables, and direct/bound generic record
methods, retaining both record and method arguments across cached-unit imports.
Prove construction requirements reach each caller without changing visible
arity/labels or duplicating unrelated generic bodies. Reuse AP47.2's malformed
artifact fixtures rather than replaying them as source tests.

Formatter round trips are stable. LSP/CLI source queries recognize unsaved error
types and preserve existing JSON envelopes. Existing enum/Error(expression)
fixtures still pass. Compiler fixtures verify Message() does not permit host
effects inside contracts. Run the appropriate crate tests, workspace checks,
language suite, compact-reference example test and editor grammar verification.
No MapError example is published as runnable until AP47.4 is delivered.

The [performance comparison](../../../bench/ap47-language.md) records unchanged
VM, tooling and artifact workloads against the completed AP47.2 source baseline.
The independent imported-generic-enum projection failure and NotesUpdate
reference-count assertion remain the checkout follow-ups recorded in AP47.2.

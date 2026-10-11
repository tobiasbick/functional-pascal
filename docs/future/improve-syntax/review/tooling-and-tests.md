# Tooling and test repair plan

This is the tooling portion of the **Spec** axis of the [current review](README.md).
All three findings were reproduced through the public `LanguageService` API.
Each probe first checked that its FPAS source had no semantic diagnostics.
The temporary Rust probes are separate from the existing suite and were removed
from the crate after execution. Their inputs and query contracts are below.

## T01 — P2: generic data type parameters are missing from editor symbols

Status: repaired and verified. Owner: AP24 generic type symbol extraction.

### Requirement and reproduction

The [editor handbook](../../../pascal/tools/editor-integration.md), lines
362–372, promises navigation following lexical resolution; lines 393–401 cover
references/rename, and lines 500–503 include type-parameter semantic tokens.
[Migration rules](../development-process.md#migration-rules) require editor
consumers to follow delivered language changes.

```pascal
program Review;
type Box of Item = record
  Value: Item;
end record;
begin
  const B: Box of integer := Box(Value := 1);
  discard B;
end.
```

Using a loose `WorkspaceContext`, analyze this document and query
`LanguageService::definitions` at `Item` in `Value: Item`. Expected: one
`SymbolKind::TypeParameter` at the declaration's `Item`. Actual: an empty list.
This is a semantically valid generic record, not a parser recovery scenario.
Rename and token coverage were not reached by this failing definition assertion;
they are required repair checks, not separately claimed executed failures.

### Cause and tasks

At the reviewed revision,
[symbols/extract/mod.rs](../../../../crates/fpas-language-service/src/symbols/extract/mod.rs),
lines 166–173, passes only `TypeDef.body` to
[members.rs](../../../../crates/fpas-language-service/src/symbols/extract/members.rs),
lines 12–18. It drops the type parameter list. The existing parameter-symbol
extraction in [routines.rs](../../../../crates/fpas-language-service/src/symbols/extract/routines.rs),
lines 198–245, is used only for generic routines. The same omission applies to
generic enum declarations by inspection; the executable failing assertion
covered the record route.

The repair shares
[type_parameters.rs](../../../../crates/fpas-language-service/src/symbols/extract/type_parameters.rs)
between routines and data types. Source-ordered identifier lookup preserves
parameter spans even when an owner has the same name. Qualified navigation and
completion share the symbol model's member filter, so lexical parameters do not
become fields or enum members.
[Permanent regressions](../../../../crates/fpas-language-service/tests/semantic_tools/generic_parameters.rs)
cover record/enum scopes, constraints, equal owner names, rename validity and
qualified-access exclusion. Each exposed failure was reproduced before repair.
Current editor handbook rules remain unchanged.

- [x] Share parameter-symbol extraction with generic record and enum types,
  preserving declaration spans, constraints and lexical scopes.
- [x] Extend public language-service tests with record and enum definition,
  references and rename cases. Include an unrelated outer type of the same name
  and a constrained parameter so scope and token classification are exercised.
- [x] Verify rename changes the declaration and its bound uses only, and the
  edited source still checks. Verify semantic tokens mark the parameter as such.

## T02 — P2: case-expression bindings resolve to a shadowed outer constant

Status: repaired and verified. Owner: AP21 expression binding extraction.

### Requirement and reproduction

The [case-expression rules](../../../pascal/language/control-flow/case-of-intro.md#case-expressions)
give each arm its own pattern bindings. The editor's documented lexical
resolution must follow that same identity.

```pascal
program Review;
const Value: integer := 9;

function Read(Input: Option of integer): integer;
begin
  return case Input of
    when Some(const Value): Value;
    when None: 0;
  end case;
end function;

begin
  discard Read(Some(1));
end.
```

Query `LanguageService::definitions` at the arm's result expression `Value`.
Expected: the `const Value` inside `Some(...)`. Actual: the outer program
constant. In the compact one-line fixture the returned selection offset was
`22`, while the pattern declaration was at `136`.

### Cause and tasks

At the reviewed revision,
[symbols/extract/routines.rs](../../../../crates/fpas-language-service/src/symbols/extract/routines.rs),
lines 47–184, collects statement declarations without descending into expression
patterns. The return expression's binder never enters the symbol tree, so
[navigation/resolve.rs](../../../../crates/fpas-language-service/src/navigation/resolve.rs),
lines 143–160, finds the outer declaration instead.

Related missing statement-pattern symbols predate AP21; this finding records
the newly delivered expression route and does not claim the entire underlying
limitation was introduced by AP21.

The repair separates
[statements.rs](../../../../crates/fpas-language-service/src/symbols/extract/statements.rs)
from routine extraction and adds
[expressions.rs](../../../../crates/fpas-language-service/src/symbols/extract/expressions.rs).
Both use [patterns.rs](../../../../crates/fpas-language-service/src/symbols/extract/patterns.rs)
for case and condition binders. Bindings start after the complete matching test
and end with their branch or arm. The shared symbol visibility predicate also
recognizes declaration positions without exposing a binder in earlier uses.
[Public API regressions](../../../../crates/fpas-language-service/tests/semantic_tools/decision_bindings.rs)
check definitions, references, isolated rename edits and valid edited sources.
Current pattern and editor handbook rules remain unchanged.

- [x] Collect expression pattern bindings with branch/arm scopes, including
  expressions in returns, initializers, arguments and nested expressions.
  Reuse the same binding owner for statement and expression patterns.
- [x] Add a public definition regression for the reproducer and a rename
  isolation test. Renaming either the outer constant or the arm binder must
  leave the other identity untouched and produce valid FPAS.
- [x] Cover sibling arms with equal binder names, nested cases and `if`
  expressions with `is`. Confirm binders do not leak into later arms or `else`.

## T03 — P3: generic callable-field signature help remains unspecialized

Status: confirmed; repair open. Owner: AP24 record-member presentation.

### Requirement and reproduction

The [editor handbook](../../../pascal/tools/editor-integration.md), lines
472–478, promises signature help for callable fields. AP24 adds instantiated
generic record fields to the existing editor consumer surface.

```pascal
program Review;
type Box of Item = record
  Callback: function(Value: Item): Item;
end record;

function Identity(Value: integer): integer;
begin
  return Value;
end function;

begin
  const B: Box of integer := Box(Callback := Identity);
  discard B.Callback(1);
end.
```

Call `LanguageService::signature_help` inside `B.Callback(...)`.
Expected parameter: `Value: integer`. Actual parameter: `Value: Item`, with
label `function Callback(Value: Item): Item`. Semantic analysis accepts the
program, so this is stale editor type information rather than failed inference.

### Cause and tasks

[intellisense/record_members.rs](../../../../crates/fpas-language-service/src/intellisense/record_members.rs),
lines 32–39, specializes `detail` and `type_name` but leaves `callable` unchanged.
[signature_help.rs](../../../../crates/fpas-language-service/src/intellisense/signature_help.rs),
line 70, then consumes the original declaration signature installed by
[symbols/extract/members.rs](../../../../crates/fpas-language-service/src/symbols/extract/members.rs),
lines 41–42.

- [ ] Update the callable signature from the instantiated semantic field type,
  retaining parameter names, modes and the original declaration location.
- [ ] Extend signature-help tests for function and procedure fields, including
  aliases/imported records and a nested concrete type argument.
- [ ] Verify both parameter and return type text, active argument selection,
  and unchanged definition/rename targeting. Keep this in the shared member
  specialization owner, not a signature-help-only string replacement.

## Other tooling and documentation checks

The formatter preserved the inspected contract/generic comments and was
idempotent on the scratch fixture. Repository FPAS formatting and TextMate
grammar verification passed. No independent formatter or current-documentation
defect was established in this review. The full VS Code extension-host suite
was not run; public language-service probes do not substitute for that check.

For each repair, extend the existing behavior-owning tests and demonstrate the
original failure before changing production code. These omissions are not a
reason to delete existing tests or add source-text inventory assertions.

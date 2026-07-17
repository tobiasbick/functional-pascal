# Record properties

## Status

Planned language feature. This document defines computed properties for record values. Current
behavior remains documented under `docs/pascal/`.

Property reads build on ordinary member access and the expression postfix-chaining implementation.
Property writes extend assignment checking but do not require assignment through arbitrary
temporary expressions.

Current next step: finish the in-progress postfix-chaining work, then implement this plan after
capturing closures and the bound-method milestone in
[events-and-bound-methods.md](events-and-bound-methods.md).

## Motivation

Record APIs currently expose operations through instance methods or type-owned functions:

```pascal
var Caption: string := TuiButton.GetText(Button);
TuiButton.SetText(Button, 'Save');
```

Computed properties make state-shaped APIs readable without exposing their storage:

```pascal
var Caption: string := Button.Text;
Button.Text := 'Save';
Button.Enabled := CanSave;
```

This is especially important for live handle records. `Button.Text` does not need to be a field in
the small handle value. Its accessors can validate the handle and read or update the canonical entry
in a Functional Pascal registry.

## Decision

Records may declare computed instance properties backed by existing instance methods:

```pascal
type
  TuiButton = record
    function GetText(Self: TuiButton): string;
    procedure SetText(Self: TuiButton; Value: string);

    property Text: string read GetText write SetText;
  end;
```

Reading and writing use normal Pascal member syntax:

```pascal
WriteLn(Button.Text);
Button.Text := 'Save';
```

Conceptual lowering is:

```pascal
Button.GetText();
Button.SetText('Save');
```

The receiver is evaluated exactly once. Accessor resolution is static and uses the record's declared
property metadata; there is no runtime name lookup.

## Declaration forms

Version 1 supports read-only, write-only, and read-write properties:

```pascal
property Width: integer read GetWidth;
property Password: string write SetPassword;
property Text: string read GetText write SetText;
```

Rules:

- A property must declare at least one accessor.
- The property name shares the case-insensitive member namespace with fields, methods, static
  functions, and other properties.
- `read` names one instance function declared on the same record.
- `write` names one instance procedure declared on the same record.
- Accessors may be declared before or after the property; the complete record is resolved as one
  declaration group.
- A property has the visibility of its containing exported record in version 1. Finer member
  visibility is a separate feature.
- Type aliases expose the resolved record's properties in the same way they expose its methods.

Version 1 does not allow a field name directly after `read` or `write`. Accessors keep validation,
invalidation, diagnostics, and registry ownership explicit and avoid adding hidden field mutation.

## Accessor signatures

For a property of type `T` on record `R`, the read accessor must have this effective signature:

```pascal
function Getter(Self: R): T;
```

The write accessor must have this effective signature:

```pascal
procedure Setter(Self: R; Value: T);
```

The accessor names and parameter names are unrestricted; only their types and positions matter.
The normal instance-method convention removes `Self` at the call site.

Accessors in version 1:

- cannot declare extra parameters;
- cannot be static;
- cannot be generic unless all type arguments are fixed by the containing record type;
- cannot use a function as a setter or a procedure as a getter;
- use normal type compatibility without property-specific implicit conversion.

The setter receives `Self` by value under the current record-method convention. Property assignment
does not mutate the receiver binding. A setter may update data reached through a handle, reference,
array, registry, or other existing mutable facility. Mutation of an ordinary record field still
requires the existing `mutable var` field-assignment rules.

This distinction permits:

```pascal
var Button: TuiButton := FindButton();
Button.Text := 'Save';
```

even though `Button` is an immutable binding: the handle value is unchanged, while its registry
entry is updated by `SetText`.

## Read semantics

A readable property is an expression and may participate anywhere its result type is accepted:

```pascal
var Length: integer := Window.Title.Length;
var Text: string := CreateButton().Text;
```

Property access has the same precedence as field access. Postfix chaining may continue from its
result with fields, indexes, methods, or further properties.

The getter is invoked once at the point of access. Repeating `Button.Text` in source invokes the
getter again; properties are not automatically cached.

Reading a write-only property is a compile-time error naming the property and suggesting assignment.

## Write semantics

Property assignment is recognized when the final member of an assignment target is a writable
property:

```pascal
Button.Text := BuildCaption();
Form.AcceptButton.Enabled := true;
```

Evaluation order is fixed:

1. evaluate the receiver path exactly once from left to right;
2. evaluate the right-hand expression exactly once;
3. invoke the setter with the receiver and value.

The receiver before the final property must be a valid designator in version 1. Assignment through
a temporary remains invalid:

```pascal
CreateButton().Text := 'Lost';  { error in version 1 }
```

Supporting statement assignment through arbitrary postfix results is a separate language decision.

Writing a read-only property is a compile-time error naming the property and its getter. Property
assignment is a statement and does not produce a value, so chained assignment is not supported.

## Properties versus fields

Properties are behavior, not stored record data:

- record literals cannot initialize a property;
- record update expressions cannot name a property;
- equality, formatting, serialization, and pattern matching ignore properties;
- copying a record copies its fields normally and creates no property storage;
- default field values do not apply to properties;
- a property getter may compute a different value on every call.

This avoids the surprising aliasing semantics that would result from storing event slots or other
live state directly in otherwise value-shaped handle records.

## Properties versus methods

Use a property when the operation represents observable state and takes no explicit argument beyond
the assigned value. Use a method when the operation represents an action, may have multiple
arguments, or deserves an imperative name.

Good property shapes:

```pascal
Button.Text
Button.Enabled
List.SelectedIndex
Window.Bounds
```

Operations that remain methods:

```pascal
Window.Close()
List.Insert(Index, Item)
Canvas.DrawText(Position, Text)
Action.Activate(Source)
```

## Events as specialized properties

This plan implements only ordinary computed properties. The event plan builds a restricted event
member on the same accessor and assignment machinery:

```pascal
Button.OnClick := HandleClick;
Button.OnClick := nil;
```

Event ownership, `Assigned`, callable compatibility, clearing, and owner-only invocation remain in
[events-and-bound-methods.md](events-and-bound-methods.md). They must not be special-cased as hidden
record fields.

## Diagnostics

Diagnostics must name the record and property and include a correction for:

- missing both accessors;
- unknown accessor name;
- static or wrong-kind accessor;
- getter return-type mismatch;
- setter value-type mismatch;
- extra accessor parameters;
- reading a write-only property;
- writing a read-only property;
- using a property in a record literal or record update;
- assigning through a temporary receiver;
- recursively resolving a malformed property declaration.

## Scope boundaries

Version 1 does not add:

- indexed or parameterized properties;
- default properties;
- static or unit properties;
- direct field-backed `read Field` or `write Field` declarations;
- property observers or automatic change notifications;
- accessor overloading;
- member-level visibility sections;
- assignment expressions or chained assignment;
- arbitrary setter assignment through temporary values;
- an object or class model.

## Expected implementation shape

```text
crates/fpas-parser/src/
  ast/types/record_property.rs        — NEW: property declaration AST
  parser/types/record_property.rs     — NEW: declaration parsing and recovery
  tests/types/record_properties.rs    — NEW: syntax and recovery tests

crates/fpas-sema/src/check/decl/types/
  record_properties.rs               — NEW: member names and accessor validation

crates/fpas-sema/src/check/expr/
  property.rs                         — NEW: getter resolution and result type

crates/fpas-sema/src/check/stmt/
  property_assignment.rs             — NEW: setter target and value checking

crates/fpas-compiler/src/compiler/
  expr/property.rs                    — NEW: getter lowering
  stmt/property_assignment.rs         — NEW: setter lowering and evaluation order

crates/fpas-fmt/src/emit/
  record_property.rs                 — NEW: declaration formatting
```

Reuse existing instance-method call metadata and lowering. Do not add property-specific VM opcodes:
a getter or setter is an ordinary resolved method call after semantic analysis.

Project linking and source-map rewriting must traverse accessor declarations and property reads and
writes wherever they introduce new AST nodes.

## Implementation order

1. Add property declaration AST, parser recovery, formatter output, and round-trip tests.
2. Add the property member namespace and accessor signature validation.
3. Resolve read-only property expressions and lower them as getter calls.
4. Extend assignment targets with writable properties and fixed evaluation order.
5. Add type-alias, generic-record, linker-rewrite, and source-map coverage.
6. Add negative diagnostics and ensure fields and methods retain current behavior.
7. Update grammar and current language documentation after the implementation works.
8. Convert a small non-TUI canary from explicit getter/setter calls.
9. Run complete verification and remove this future plan.

## Required tests

- read-only, write-only, and read-write declarations;
- getter and setter calls on a normal record and a live-handle-shaped record;
- immutable handle binding accepted for a setter that updates external state;
- getter result followed by field, index, method, and property postfix operations;
- receiver and assigned value evaluated exactly once and in source order;
- read-only/write-only misuse diagnostics;
- wrong accessor kind, arity, receiver, and value type diagnostics;
- duplicate names across fields, methods, static functions, and properties;
- property exclusion from literals, record updates, equality, and serialization;
- public type alias property access;
- formatter and parse-format-parse stability;
- project dependency rewriting and source spans;
- no new bytecode opcode.

## Acceptance criteria

- all syntax examples in this document compile with the specified behavior;
- property access lowers to existing instance calls without VM special cases;
- reads and writes evaluate every source expression exactly once;
- properties do not alter record storage or copy semantics;
- immutable handle values can expose stateful computed setters safely;
- invalid declarations and accesses produce precise diagnostics;
- grammar, formatter, sema, compiler, linker, source maps, docs, and tests agree;
- complete workspace, Clippy, FPAS regression, formatter, and diff verification passes.

## Plan lifecycle

Keep this file under `docs/future/` until properties are implemented and documented under
`docs/pascal/`. Then remove its future index entry and delete this plan.

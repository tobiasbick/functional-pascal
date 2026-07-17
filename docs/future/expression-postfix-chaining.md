# Expression postfix chaining

## Status

Planned language feature. This document is an implementation handoff, not current language
documentation. Implemented behavior remains documented under `docs/pascal/`.

Current next step: implement checklist item 1, AST and parser support. No implementation item is
complete yet.

### Progress checklist

- [ ] 1. AST and parser suffix loop
- [ ] 2. Formatter emission and round-trip coverage
- [ ] 3. Semantic field and index chaining
- [ ] 4. Semantic instance-method chaining and method metadata
- [ ] 5. Compiler lowering and end-to-end tests
- [ ] 6. Grammar and current documentation
- [ ] 7. Tui2 regression migrated to real chaining
- [ ] 8. Full verification and plan removal

The motivating missing forms are:

```pascal
var Green: integer := BuildPalette().ForRole(TuiStyleRole.Normal).Foreground.Green;
var First: string := LoadItems()[0];
```

Functional Pascal currently accepts fields and indexes only as parts of a designator that starts
with an identifier. A function or method call is a complete primary expression, so its result
cannot be followed by another field, index, or method operation.

## Decision

Add postfix chaining to expression results. Version 1 supports these suffixes:

```text
.Field
[IndexExpression]
.Method(Arguments)
```

Suffixes are evaluated strictly from left to right. Each step receives the value and static type
produced by the preceding step.

Examples that must work:

```pascal
Factory.Create().Value
Factory.Create().Transform(2).Value
Factory.Create()[0]
(Factory.Create()).Value
Palette.WithRole(TuiStyleRole.Warning, Custom).ForRole(TuiStyleRole.Warning)
```

Postfix chaining has the same precedence as existing designator field and index access: it binds
tighter than unary, multiplicative, additive, comparison, and record-update expressions.

## Scope boundaries

Version 1 deliberately does not add:

- overloads, extension methods, optional chaining, null propagation, or a pipeline operator;
- implicit copying, mutation, or assignment through a temporary result;
- calls of returned function values such as `Factory()()`;
- chained call statements whose final result is discarded, such as `Factory().Close();`;
- changes to existing simple `Designator`, `Expr::Call`, `Stmt::Call`, or assignment-target syntax;
- a new bytecode opcode.

The initial feature is expression-only. A procedure cannot occur in an expression chain because it
does not produce a value. Statement chaining can be planned separately after expression chaining
is stable.

## Chosen AST shape

Keep the current AST for ordinary designators and calls. Add one wrapper only when at least one
suffix follows an already parsed primary expression:

```rust
Expr::Postfix {
    base: Box<Expr>,
    operations: Vec<PostfixOperation>,
    span: Span,
}

pub enum PostfixOperation {
    Field {
        name: String,
        span: Span,
    },
    Index {
        index: Box<Expr>,
        span: Span,
    },
    MethodCall {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
}
```

Do not encode `.Method(...)` as a `Field` followed by a generic `Call`. Keeping it as one operation
lets semantic analysis resolve record methods without inventing bound-method values. It also lets
the compiler reuse the current instance-method calling convention: receiver first, followed by
explicit arguments.

`Expr::span()` returns the combined span from the beginning of `base` through the final suffix.
`PostfixOperation` must be public and documented because it is part of the public parser AST.

## Grammar

Update `docs/specs/grammar.ebnf` conceptually as follows:

```ebnf
primary_expr       = primary_atom { postfix_suffix } ;

postfix_suffix     = '.' identifier [ call_args ]
                   | '[' expression ']' ;
```

The existing identifier-led `designator [call_args]` remains a `primary_atom`. This preserves
qualified names such as `Std.Math.Sqrt(4.0)`, static record functions, enum constructors, imported
unit symbols, and current short-name resolution. Once that atom is complete, the parser consumes
any remaining postfix suffixes.

Parser behavior for a dot suffix is fixed:

- `.Name(` parses as `PostfixOperation::MethodCall`;
- `.Name` without `(` parses as `PostfixOperation::Field`;
- `[` parses as `PostfixOperation::Index`;
- no empty suffix list is emitted; return the original expression unchanged.

Refactor `parse_primary` into an atom step plus a suffix loop. Do not duplicate argument-list or
index-expression parsing.

## Semantic analysis

Add `check_postfix_expr(base, operations)` under a focused expression module. It starts with
`check_expr(base)` and updates the current type for every operation.

### Field

Resolve aliases with the existing `resolve_visible_type`. The receiver must be a record. Look up
the field case-insensitively and return its type. Reuse the existing diagnostics for non-record
receivers and unknown record fields.

### Index

Check the index expression exactly once. Reuse the current designator rules:

- array index: `integer`, result is the element type;
- dictionary index: compatible key type, result is the value type;
- string index: `integer`, result is `string`;
- every other receiver: type error.

Extract shared index checking if necessary; do not copy the array/dictionary/string rules into two
independent implementations.

### Method call

Resolve aliases, require a record receiver, and find an instance method on that record. Static
record functions remain callable only through a type designator and therefore are not valid as a
postfix operation on a value.

For an instance function:

1. validate its implicit `Self` parameter through the existing method helpers;
2. check explicit argument count, generic inference, and argument types with existing helpers;
3. record the qualified method target for code generation;
4. return the function return type.

For an instance procedure in an expression, emit the existing “does not return a value” style of
diagnostic and return `Ty::Error` to prevent cascades.

Unknown methods must produce a concrete diagnostic naming the receiver type and method, for
example:

```text
Record `TuiPalette` has no method `Missing`
```

The hint should tell the user to check the record declaration or use a field without parentheses.

### Method-call metadata

The existing `MethodCallMap` is keyed by AST-node identity. Add a documented helper that returns
the stable address key of a `PostfixOperation`, and store the resolved `MethodCallTarget::Instance`
under the `MethodCall` operation's key. The AST remains immutable throughout sema and compilation,
so addresses of operations inside the completed vector remain stable.

Do not identify operations only by source span: recovery nodes and generated tests may share spans.

Ensure `ExprTypeMap` records the final type of the complete `Expr::Postfix`, as it does for all other
expressions.

## Compiler lowering

Add a focused postfix-expression lowering module. Compile `base` exactly once, leaving its value on
the stack. Then lower operations in source order:

- `Field`: emit the existing string constant and `Op::FieldGet`;
- `Index`: compile the index expression and emit `Op::IndexGet`;
- `MethodCall`: the receiver is already on the stack; compile explicit arguments, fetch the
  qualified instance target from `MethodCallMap`, and emit the existing `Op::Call` with arity
  `args.len() + 1`.

Every operation consumes the preceding value and leaves exactly one resulting value. No temporary
local, cloning opcode, stack rotation, or new bytecode instruction is required.

Use checked `u8` conversion for method arity through the existing compiler helper. Missing sema
metadata is an internal compiler error with an actionable diagnostic, not an `unwrap` or silent
fallback.

## Formatter

Add postfix emission to the expression formatter with the highest expression precedence.

Canonical short output stays on one line:

```pascal
Palette.WithRole(TuiStyleRole.Warning, Custom).ForRole(TuiStyleRole.Warning).Foreground
```

If the rendered chain exceeds the formatter's 100-column limit, break before suffixes and indent
continuations by two spaces from the expression's base column:

```pascal
Palette.WithRole(TuiStyleRole.Warning, Custom)
  .ForRole(TuiStyleRole.Warning)
  .Foreground
```

An index suffix follows the same rule. Parenthesize the base only when existing precedence rules
require it; preserve explicit `Expr::Paren` nodes.

Add formatter round-trip coverage. Formatting must not change the AST meaning or turn a qualified
root call into a postfix method call.

## Files to change

Keep one concern per file. Expected layout:

```text
crates/fpas-parser/src/
  ast/expr.rs                         — add Expr::Postfix and PostfixOperation
  parser/expr/postfix.rs              — extend: parse suffix loop after a primary atom
  parser/expr/primary.rs              — split primary atom from postfix application
  tests/expr/postfix.rs               — NEW: parser shape and recovery tests
  tests/expr/mod.rs                   — register test module

crates/fpas-sema/src/check/
  context.rs                          — postfix-operation identity helper/map contract
  expr/mod.rs                         — dispatch Expr::Postfix
  expr/postfix.rs                     — NEW: field/index/method type checking
crates/fpas-sema/src/tests/
  expr.rs                             — MOVED/SPLIT: existing expression tests
  expr/
    mod.rs                            — NEW: expression test module root
    postfix.rs                        — NEW: positive and negative semantic tests

crates/fpas-compiler/src/compiler/
  expr/mod.rs                         — dispatch Expr::Postfix
  expr/postfix.rs                     — NEW: sequential lowering
crates/fpas-compiler/src/tests/
  postfix_chaining.rs                 — NEW: end-to-end execution and errors
  mod.rs                              — register test module

crates/fpas-fmt/src/emit/expr/
  mod.rs                              — dispatch and precedence integration
  postfix.rs                          — NEW: compact and wrapped emission
crates/fpas-fmt/tests/golden/
  postfix_chaining.expected.fpas      — NEW
crates/fpas-fmt/tests/golden_output.rs
                                      — register golden test with inline unformatted input

docs/specs/grammar.ebnf               — implemented grammar
docs/pascal/language/functions/README.md
                                      — implemented call-result chaining
docs/pascal/language/types/record-methods.md
                                      — method chaining on returned records
docs/pascal/tools/fmt-style.md         — canonical wrapping rule
tests/stdlib/tui2/cell_values_test.fpas
                                      — replace palette lookup temporaries with a real chain
```

If an existing target file approaches 400 lines, split the relevant concern before adding logic.
Do not put postfix checking into the already broad designator implementation merely because the
field and index rules began there.

## Required tests

### Parser

- call result followed by a field;
- call result followed by an index;
- call result followed by one method call;
- two method calls followed by a field;
- parenthesized call result followed by a field;
- qualified root call remains the existing `Expr::Call` inside the postfix base;
- missing identifier after `.` and missing `]` produce parser diagnostics without panic.

### Semantic analysis

- field type on a returned record;
- index result for returned array, dictionary, and string;
- instance method argument and return-type propagation across two steps;
- type aliases on intermediate record types;
- unknown field, unknown method, non-record member access, wrong index type, and non-indexable
  receiver;
- static function attempted through a returned value;
- procedure method attempted inside an expression;
- generic instance function in the middle of a chain.

### Compiler and VM

- each base call executes exactly once;
- `Create().Next().Value` produces the expected value;
- `CreateArray()[index]` produces the expected element;
- method arguments are evaluated once and in source order;
- chained field access emits no new opcode and leaves a balanced stack;
- method arity overflow remains diagnosed.

### Formatter

- compact chain;
- long chain wrapped before suffixes;
- field and index mixtures;
- explicit parentheses preserved;
- format-parse-format stability.

### FPAS regression

After implementation, simplify the Tui2 palette test to exercise the motivating API directly,
for example:

```pascal
AssertEquals(
  2,
  Palette.WithRole(TuiStyleRole.Warning, Custom)
    .ForRole(TuiStyleRole.Warning)
    .Foreground.Green
)
```

Use the formatter's actual canonical layout in the committed test.

## Implementation order

Work in this order and keep every step buildable:

1. Add AST nodes, spans, parser suffix loop, and parser tests.
2. Add formatter support and round-trip tests so the new AST can be inspected safely.
3. Add semantic field and index chaining with tests.
4. Add semantic instance-method chaining and operation-keyed method metadata.
5. Add compiler lowering and end-to-end compiler/VM tests.
6. Update grammar and current user documentation only after behavior works.
7. Replace the temporary-variable pattern in the Tui2 regression with chaining.
8. Run the complete verification matrix and remove this future plan when all acceptance criteria
   are satisfied.

Do not begin with Tui2-specific special cases. The feature belongs to the general expression
pipeline and Tui2 is only its first real consumer.

## Acceptance criteria

The feature is complete only when all of the following are true:

- all syntax examples in the Decision section compile and run;
- the base expression and every argument are evaluated exactly once in source order;
- field, index, and instance-method suffixes can be mixed arbitrarily;
- simple calls, qualified calls, designators, assignments, enum constructors, static record
  functions, `go`, `try`, and record updates retain their existing behavior;
- invalid suffixes produce specific sema diagnostics without parser or compiler panics;
- `fpas fmt` preserves meaning and respects the canonical wrapping rule;
- no new bytecode opcode or VM special case was introduced;
- the Tui2 palette regression uses at least one real chain;
- current documentation describes only the implemented syntax;
- the full verification matrix passes.

## Verification matrix

Run from the repository root:

```text
cargo fmt --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -q -p fpas-cli -- fmt --check tests/stdlib/tui2/cell_values_test.fpas
cargo run -q -p fpas-cli -- test --std-lib lib tests/
git diff --check
```

## Plan lifecycle

While implementation is incomplete, update this document with completed checklist items and an
explicit next step. Once every acceptance criterion passes:

1. ensure the grammar and `docs/pascal/` pages are the authoritative description;
2. remove the entry from `docs/future/README.md` and the optional Tui2 cross-reference;
3. delete this plan file, matching the lifecycle used for the static-record-function plan.

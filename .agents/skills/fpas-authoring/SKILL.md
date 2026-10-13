---
name: fpas-authoring
description: Write, fix, or format Functional Pascal .fpas sources, including programs, units, examples, and regression tests.
---

# FPAS authoring

Start with the [compact language reference](../../../docs/pascal/agent-reference.md)
for syntax, common Pascal/Delphi/Ada pitfalls, and complete tested examples. Use the
[topic reference](../../../docs/pascal/README.md) and nearby working sources for
details. Repository placement and shared completion checks live in
[AGENTS.md](../../../AGENTS.md).

## Read for the task

| Task | Reference |
|------|-----------|
| Programs, units, imports, visibility | [Units](../../../docs/pascal/program-structure/units.md) |
| Bindings and expressions | [Basics](../../../docs/pascal/language/basics/README.md) |
| Routines, parameters, closures | [Functions](../../../docs/pascal/language/functions/README.md) |
| Branches and loops | [Control flow](../../../docs/pascal/language/control-flow/README.md) |
| Standard-library calls | [Std reference](../../../docs/pascal/std/README.md) |
| Assertions, scripted input, golden output | [Test runner](../../../docs/pascal/std/testing/test.md) |

Read the pages relevant to the change. For exact productions, consult
[grammar.ebnf](../../../docs/specs/grammar.ebnf).

## Source workflow

1. Inspect the owning manifest and neighboring sources before choosing a program
   or unit. Use [fpas-projects](../fpas-projects/SKILL.md) when manifests or
   dependencies need editing.
2. Reuse a working pattern: [hello world](../../../docs/pascal/getting-started/hello-world.md),
   [assertion test](../../../tests/stdlib/str/higher_order_test.fpas), or
   [headless TUI test](../../../tests/stdlib/tui/mvu_host_signature_test.fpas).
3. For a new regression test, confirm that
   [tests/suite.fpasprj](../../../tests/suite.fpasprj) includes its path.

## Syntax to keep explicit

- Each type, constant, or variable repeats its own `type`, `const`, or `var` keyword. Repeat `public` on each exported declaration.
  Use `const` for immutable values and `var` for reassignment.
  Parameters and loop variables are read-only; copy a parameter into a fresh
  local `var` when it needs local changes or shared mutable captures.
  Computed `const` bindings are allowed inline in statement lists;
  their initializer runs whenever execution reaches the declaration. Keep record
  fields and enum members in their existing syntax; local types are not allowed.
- Scalar `case` value labels and both range endpoints require compile-time
  constants. Calls and computed bindings belong in guard conditions.
- Patterns bind explicitly: `when Some(const Value):`,
  `when Shape.Rect(const W, _):`, and `when const N if N > 0:` for a scalar
  guard binding. `_` ignores one field. Patterns nest (`Ok(Some(const User))`)
  and compare payloads with literals or compile-time constants; a plain
  identifier that names no constant is an error. Coverage is checked
  recursively. Closed enum, Option, and Result cases list every variant
  explicitly and reject `else`, including a redundant branch. Only scalar
  integer/string cases require `else`; Boolean and subrange cases cover their
  complete finite domain and forbid `else`. Statements and expressions share
  these rules; distinct follows its supported base type. Guards never count
  toward coverage. Duplicate labels within an arm and labels fully covered by
  earlier unguarded arms are rejected; partial overlaps remain legal.
  `Value is Some(_)` is a normal
  Boolean expression supporting grouping, logical operators, guards, loops,
  arguments and returns. It never binds names; `const Name` is rejected at any
  depth. Use case arms to extract payloads. `is` is reserved. Return binders
  require the colon in `ensures (Returned): Expression;`; `ensures (Flag);`
  is an ordinary grouped Boolean condition.
- Import every referenced unit with `uses`, including fully qualified `Std.*`
  calls. Qualify ambiguous short names with the current unit name from its handbook.
  Import each unit only once per file, comparing names case-insensitively.
  `uses Std.Math as Math;` exposes only `Math.Name`, with no short names or
  access through `Std.Math.Name`. Alias names cannot be shadowed by declarations,
  parameters, loop variables, or pattern bindings. `as` remains contextual in `uses`.
- Functions return with `return`. Consume function results or use an allowed
  explicit [discard](../../../docs/pascal/language/functions/discard.md).
- Named free, member, and nested routine signatures are available throughout
  their lexical scope, including mutually recursive siblings. Values initialize
  in source order. Call or capture a helper only after its direct and transitive
  dependencies are initialized; FP3041 rejects unsafe or unprovable use.
  Callable bindings remain ordinary initialized values. Record defaults run
  at construction in their declaring environment.
- Native collection mutation requires InPlace: Items.PushInPlace(Value),
  Items.SortInPlace(), or Values.RemoveInPlace(Key). Without InPlace, operations
  preserve receivers: Push returns a new array; Pop returns ArrayPopResult of T
  with Remaining and Value. PopInPlace returns the removed element. Both Pop forms
  fail on empty arrays. InPlace needs writable storage and retains its static type;
  MapInPlace/FlatMapInPlace cannot change element/value types. User Self stays read-only.
- Statements end with `;`. Use each construct's named closer, such as
  `end function;`, `end if;`, and `end unit;`; programs end with `end.`.
  Expression closers receive their terminator from the enclosing statement.
- Unit declarations and record members are private by default; `public` exports
  an individual declaration or member in a unit.
- Construct records through their type with named fields, for example
  `Point(X := 1, Y := 2)`. Omit fields only when they have defaults; use
  `TypeName()` for empty records or to use all defaults. Supplied values run in
  written order, followed by omitted defaults in declaration order.
- Strings use single quotes and doubled quotes for escaping. Source comments use
  `//`; adjacent standalone blocks provide [Markdown documentation](../../../docs/pascal/language/basics/comments.md).

## Verify sources

Use [formatter output](../../../docs/pascal/tools/fmt-style.md) as the style authority:

```text
fpas fmt <paths>
fpas fmt --check <paths>
fpas check <file-or-project>
fpas test <test-path>
```

Check interactive programs without launching them in batch validation. Use
scripted or headless tests for their runtime behavior. After FPAS regression-test
changes, run the repository suite as required by AGENTS.md.

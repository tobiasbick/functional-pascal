---
name: fpas-authoring
description: Write, fix, or format Functional Pascal .fpas sources, including programs, units, examples, and regression tests.
---

# FPAS authoring

Use the current [language reference](../../../docs/pascal/README.md) and nearby
working sources. Repository placement and shared completion checks live in
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

- Each type, constant, or variable repeats its own `type`, `const`, `var`, or
  complete `mutable var` prefix. Repeat `public` on each exported declaration.
  Bindings use `var` for immutable values and `mutable var` for reassignment.
  Keep record fields and enum members in their existing syntax; local types
  and constants are not declaration positions.
- Import every referenced unit with `uses`, including fully qualified `Std.*`
  calls. Qualify ambiguous short names with the current unit name from its handbook.
- Functions return with `return`. Consume function results or use an allowed
  explicit [discard](../../../docs/pascal/language/functions/discard.md).
- Statements end with `;`. Use each construct's named closer, such as
  `end function;`, `end if;`, and `end unit;`; programs end with `end.`.
  Expression closers receive their terminator from the enclosing statement.
- Unit declarations and record members are private by default; `public` exports
  an individual declaration or member in a unit.
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

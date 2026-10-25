---
name: fpas-authoring
description: Write, fix, or format Functional Pascal .fpas sources, including programs, units, examples, and regression tests.
---

# FPAS authoring

Read the [agent reference entrypoint](../../../docs/pascal/agents/README.md) when
writing FPAS, then load the topics needed for the task. It supplies syntax and
pitfalls for agents unfamiliar with the language. Use the
[handbook](../../../docs/pascal/README.md) for detailed semantics and library APIs.
Repository rules live in [AGENTS.md](../../../AGENTS.md); placement and
verification workflows live in [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## Read for the task

| Task | Reference |
| --- | --- |
| Bindings, expressions, branches, loops, collections | [Basics](../../../docs/pascal/agents/basics.md) |
| Parameters, caller mutation, callbacks, closures, contracts | [Functions](../../../docs/pascal/agents/functions.md) |
| Records, enums, generics, domain types, imports | [Types and units](../../../docs/pascal/agents/types.md) |
| Pattern matching, Result, Option, propagation, typed errors | [Patterns and errors](../../../docs/pascal/agents/patterns-errors.md) |
| Tasks, scopes, channels, cancellation and groups | [Concurrency](../../../docs/pascal/agents/concurrency.md) |
| API lookup, source navigation, diagnostics, debugging, profiling | [Tools](../../../docs/pascal/agents/tools.md) |
| Assertions, scripted input, golden output | [Test runner](../../../docs/pascal/std/testing/test.md) |

## Source workflow

1. Inspect the owning manifest and neighboring sources. Use
   [fpas-projects](../fpas-projects/SKILL.md) when manifests or dependencies change.
2. Look up exact declarations with `fpas doc <name>` or `--report json` instead
   of guessing an API. For exact syntax, consult [grammar.ebnf](../../../docs/specs/grammar.ebnf).
3. Follow the [formatter style](../../../docs/pascal/tools/fmt-style.md), then
   run `fpas fmt <paths>`, `fpas fmt --check <paths>` and
   `fpas check <file-or-project>`. `fpas check --fix` applies compiler fixes in
   place; format and inspect the result.
4. Run relevant runtime tests. For a new repository regression test, confirm that
   [tests/suite.fpasprj](../../../tests/suite.fpasprj) includes its path. After FPAS
   regression-test changes, run `fpas test tests/suite.fpasprj`. Check interactive
   programs without launching them in batch validation; use scripted/headless
   tests for execution.

Use [`fpas debug`](../../../docs/pascal/tools/debugger-batch.md) for a bounded
failure/stop snapshot and [`--profile`](../../../docs/pascal/tools/profiler.md)
before writing performance variants. Use [fpas-bench](../fpas-bench/SKILL.md) for
controlled timing comparisons. For handbook examples, follow the
[Markdown example instructions](../../../docs/pascal/tools/documentation-examples.md).

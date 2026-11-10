---
name: fpas-authoring
description: Write and review Functional Pascal CLR .fpas sources, including programs, units, examples, and regression fixtures.
---

# FPAS CLR authoring

Start with the [agent reference](../../../docs/pascal/agents/README.md), then load
the topics needed for the task. The [handbook](../../../docs/pascal/README.md)
owns language behavior. Follow [AGENTS.md](../../../AGENTS.md) and the
[contribution workflow](../../../CONTRIBUTING.md).

## Read for the task

| Task | Reference |
| --- | --- |
| Bindings, expressions, loops, arrays, and strings | [Basics](../../../docs/pascal/agents/basics.md) |
| Routines, managed parameters, delegates, captures, and contracts | [Functions](../../../docs/pascal/agents/functions.md) |
| Records, unions, domains, generics, units, and globals | [Types and units](../../../docs/pascal/agents/types.md) |
| Matching, nullable flow, exceptions, and resource cleanup | [Patterns and resources](../../../docs/pascal/agents/patterns-errors.md) |
| Tasks, await, cancellation, and shared state | [Concurrency](../../../docs/pascal/agents/concurrency.md) |
| Actual .NET APIs, events, attributes, and public signatures | [Interop](../../../docs/pascal/agents/interop.md) |
| Compiler commands and API/runtime inspection | [Tools](../../../docs/pascal/agents/tools.md) |
| Source comments and API documentation | [Comment rules and Markdown sections](../../../docs/pascal/language/basics/comments.md) |
| Regression selection and controlled fixtures | [Tests](../../../tests/README.md) and [coverage](../../../tests/coverage.md) |

## Source workflow

1. Inspect the owning `.fpasproj`, imports, and neighboring sources. Use
   [fpas-projects](../fpas-projects/SKILL.md) for project/dependency changes.
2. Check the [grammar](../../../docs/specs/syntax.md) and actual reference
   metadata. Resolve exact CLR names, overloads, parameter modes, and nullability.
   Use supported .NET APIs through explicit aliases.
3. Keep case-sensitive names, lowercase keywords, explicit types, and named
   terminators. Consume returned values or use discard. Use CLR nullability and
   exceptions; define application unions for domain outcomes.
   Document FPAS declarations with adjacent `//` Markdown comments. Use the
   documented sections for parameters and returns; `///` and XML tags are not
   source documentation markup.
4. Compile through the current CLR toolchain using the
   [development workflow](../../../docs/build/development.md). Use `fpas compile`
   for explicit sources and the SDK for projects. Format changed sources with
   `fpas fmt`. Use [`fpas fmt --check`](../../../docs/pascal/tools/fmt-style.md#command)
   to verify style during a read-only review.
5. Select the tests that exercise the changed behavior. Use the compiler and
   integration suites for toolchain regressions. For FPAS test projects, run
   [native framework tests](../../../docs/pascal/tools/testing.md) with `fpas test`.
   Rebuild copied inputs before execution. Use bounded scenarios for interactive
   programs.

## Readable logical sections

Use one blank line between groups that perform different jobs: collecting inputs,
validating them, preparing data, performing work and returning or reporting the
result. Keep related bindings and operations together, including in tests.
`fpas fmt` preserves authored sections before `const` and `var`; inspect the result
for readability.

The documentation checker compiles README and handbook examples.
LanguageSpecificationTests runs the complete programs and checks their behavior.
Follow [example conventions](../../../docs/pascal/tools/documentation-examples.md)
and [reference maintenance](../../../docs/pascal/agents/maintenance.md).
Debug/profile actual generated programs with supported .NET tools before writing
variants; use [fpas-bench](../fpas-bench/SKILL.md) for authorized measurements.
Write maintenance helpers in FPAS when supported CLR APIs suffice. Record concrete
blockers in [dogfooding gaps](../../../docs/future/dogfooding-gaps.md).

Internal designs may change. FPAS syntax changes require explicit user approval;
deferred proposals do not supply that approval.

# Examples

These projects demonstrate the CLR language and actual .NET APIs. Each linked
README identifies its inputs, build/run workflow, and expected behavior. Package
the local SDK first using the [development workflow](../docs/build/development.md#build-an-fpas-application).

## Basics

| Example | Demonstrates |
| --- | --- |
| [Hello](basics/hello/README.md) | Minimal program, Console, and SDK project |
| [Loops](basics/loops/README.md) | Typed/repeat loops, extreme bounds, and async execution |
| [Units](basics/units/README.md) | Program and companion units with explicit imports |
| [Shared state](basics/shared-state/README.md) | Unit globals, initialization, delegates, and managed references |

## Types and data models

| Example | Demonstrates |
| --- | --- |
| [Data domains](types/data-domains/README.md) | Aliases, decimal, CLR enums, distinct values, and checked ranges |
| [Inventory model](types/inventory-model/README.md) | Records, defaults, private state, equality, and copy updates |
| [Game actions](types/game-actions/README.md) | Application unions and exhaustive payload matching |
| [Generic tree](types/generic-tree/README.md) | Recursive generic unions, inference, and cross-unit use |

## Functions

| Example | Demonstrates |
| --- | --- |
| [Local functions](functions/local-functions/README.md) | Lexical recursion, captures, and escaping delegates |
| [Generic pipeline](functions/generic-pipeline/README.md) | Generic routines, constraints, and higher-order functions |

## .NET interop

| Example | Demonstrates |
| --- | --- |
| [Attributes](interop/attributes/README.md) | Real CLR metadata and JSON property naming |
| [Events](interop/events/README.md) | Actual .NET collection events and retained handler identity |
| [Type inspection](interop/type-inspection/README.md) | Type tokens, nominal identity, and generic parameters |

## Files and directories

| Example | Demonstrates |
| --- | --- |
| [Read a file](io/read-file/README.md) | StreamReader, exceptions, and deterministic disposal |
| [Directory inventory](io/directory-inventory/README.md) | Arguments, files, collections, static LINQ calls, and captured state |

## Numeric computation

| Example | Demonstrates |
| --- | --- |
| [Mandelbrot](numerics/mandelbrot/README.md) | Numeric calculations, casts, loops, and console rendering |

## User interfaces

| Example | Demonstrates |
| --- | --- |
| [GUI counter](ui/gui/counter/README.md) | Shared MVU transitions and declarative widgets in a native GUI window |
| [GUI widget gallery](ui/gui/gallery/README.md) | Graphical widgets, themes, responsive layout, controlled editing, effects and modal focus |
| [TUI filter list](ui/tui/filter-list/README.md) | Controlled filtering, selection, overlays and retained subviews |
| [TUI widget gallery](ui/tui/gallery/README.md) | Interactive widget previews, themes, and headless control |
| [Mandelbrot Explorer](ui/tui/mandelbrot/README.md) | Interactive RGB cell canvas, keyboard and mouse, and cancellable parallel rendering |

## Related documentation

The [handbook example index](../docs/pascal/examples/README.md) contains additional
complete programs compiled and executed directly from Markdown. Verification is
owned by the [xUnit harness](../tests/README.md); examples are demonstrations,
while complete end-user applications belong under [apps](../apps/README.md).

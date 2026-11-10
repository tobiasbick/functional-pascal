# Examples

These projects demonstrate the CLR language and actual .NET APIs. Each linked
README identifies its inputs, build/run workflow, and expected behavior. Package
the local SDK first using the [development workflow](../docs/build/development.md#build-an-fpas-application).

| Example | Demonstrates |
| --- | --- |
| [Hello](hello/README.md) | Minimal program, Console, and SDK project |
| [Mandelbrot](mandelbrot/README.md) | Numeric calculations, casts, loops, and console rendering |
| [Mandelbrot Explorer](tui/mandelbrot/README.md) | Interactive RGB cell canvas, keyboard and mouse, and cancellable parallel rendering |
| [TUI Widget Gallery](tui/gallery/README.md) | Interactive widget previews, themes, and headless control |
| [GUI counter](gui/counter/README.md) | Shared MVU transitions and declarative widgets in a native GUI window |
| [GUI widget gallery](gui/gallery/README.md) | Graphical widgets, themes, responsive layout, controlled editing, effects and modal focus |
| [Units](units/README.md) | Program and companion units with explicit imports |
| [Read a file](read-file/README.md) | StreamReader, exceptions, and deterministic disposal |
| [Directory inventory](directory-inventory/README.md) | Arguments, files, collections, static LINQ calls, and captured state |
| [Inventory model](inventory-model/README.md) | Records, defaults, private state, equality, and copy updates |
| [Game actions](game-actions/README.md) | Application unions and exhaustive payload matching |
| [Generic pipeline](generic-pipeline/README.md) | Generic routines, constraints, and higher-order functions |
| [Generic tree](generic-tree/README.md) | Recursive generic unions, inference, and cross-unit use |
| [Data domains](data-domains/README.md) | Aliases, decimal, CLR enums, distinct values, and checked ranges |
| [Shared state](shared-state/README.md) | Unit globals, initialization, delegates, and managed references |
| [Local functions](local-functions/README.md) | Lexical recursion, captures, and escaping delegates |
| [Loops](loops/README.md) | Typed/repeat loops, extreme bounds, and async execution |
| [Events](events/README.md) | Actual .NET collection events and retained handler identity |
| [Type inspection](type-inspection/README.md) | Type tokens, nominal identity, and generic parameters |
| [Attributes](attributes/README.md) | Real CLR metadata and JSON property naming |

The [handbook example index](../docs/pascal/examples/README.md) contains additional
complete programs compiled and executed directly from Markdown. Verification is
owned by the [xUnit harness](../tests/README.md); examples are demonstrations,
while complete end-user applications belong under [apps](../apps/README.md).

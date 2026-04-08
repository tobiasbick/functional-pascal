# Examples

Functional Pascal samples aligned with the **current** compiler and standard library (`Std.*`).

## How to run

### Single-file programs

Use when the file begins with `program` and only imports `Std.*` (or needs no other units):

```sh
fpas examples/hello.fpas
fpas examples/fibonacci.fpas
fpas examples/pascal/std/str_basics.fpas
fpas examples/pascal/std/dict_basics.fpas
fpas examples/pascal/std/task_basics.fpas
```

### Projects (`.fpasprj`)

Use when the main program imports **non-library units** (for example `App.*` or `Mandelbrot.*`). The project file links all sources:

```sh
fpas examples/pascal/units-basic/units-basic.fpasprj
fpas examples/math/mandelbrot/mandelbrot.fpasprj
```

Do **not** pass a `unit` source alone (for example `mandelbrot_color.fpas` or `math_utils.fpas`) — the compiler expects a `program` as the main file.

## Single-file programs (by topic)

| Path | Topic |
|------|--------|
| `hello.fpas` | Minimal `program` / `uses` / `begin` … `end.` |
| `fibonacci.fpas` | Recursion and counting `for` loops |
| `pascal/higher-order-functions/higher_order_functions.fpas` | First-class functions and array helpers |
| `pascal/enum-data/` | Enums with associated data and `case` |
| `pascal/error-handling/` | `Result`, `Option`, `panic`, and `try` |
| `pascal/for/` and `pascal/for-in/` | Counting loops and `for … in` |
| `pascal/generics/generic_functions.fpas` | Generic functions |
| `pascal/pattern-matching/` | Guards and exhaustiveness |
| `pascal/record-methods/` | Record methods |
| `pascal/records/defaults_with_update.fpas` | Default fields and `with` updates |
| `pascal/std/str_basics.fpas` | `Std.Str` — trim, split/join, `Format`, search/replace |
| `pascal/std/dict_basics.fpas` | `Std.Dict` — literals, `Get`, `Merge`, `Map`/`Filter` (qualified when also using `Std.Array` / `Std.Option`) |
| `pascal/std/task_basics.fpas` | `Std.Task` — `go`, `Wait`, `WaitAll` |
| `math/julia/julia.fpas` | ASCII Julia set (**interactive** — see below) |

## Multi-unit projects

| Path | Contents |
|------|----------|
| `pascal/units-basic/` | `units-basic.fpasprj`, `unit App.Math`, `App.Reporting`, program `UnitsBasic` |
| `math/mandelbrot/` | `mandelbrot.fpasprj`, program `MandelbrotShowcase`, units `Mandelbrot.Color` and `Mandelbrot.Render` |

Helper units under those folders are built only through the project; see the one-line `{ ... }` comment at the top of each unit file.

## Interactive demos (terminal)

These run until you exit (for example **Escape**). Run from a real terminal if possible.

| Path | Notes |
|------|--------|
| `math/julia/julia.fpas` | Single-file; pan/zoom with keys after first draw |
| `math/mandelbrot/mandelbrot.fpasprj` | Project; fullscreen Mandelbrot explorer |
| `pascal/tui/minimal_application.fpas` | `Std.Tui.Application` loop; **Escape** to quit |

TUI APIs use types such as **`Std.Tui.TuiEvent`** and **`Std.Tui.EventKind`** (see `docs/pascal/std/tui.md`). The console’s own event type remains **`Std.Console.Event`**.

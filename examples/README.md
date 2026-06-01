# Examples

Functional Pascal samples aligned with the **current** compiler and standard library (`Std.*`).

## How to run

### Single-file programs

Use when the file begins with `program` and only imports `Std.*` (or needs no other units):

```sh
fpas examples/hello.fpas
fpas examples/fibonacci.fpas
fpas examples/pascal/std/args_basics.fpas -- one two
fpas examples/pascal/std/str_basics.fpas
fpas examples/pascal/std/dict_basics.fpas
fpas examples/pascal/std/json_basics.fpas
fpas examples/pascal/std/graph_basics.fpas
fpas examples/pascal/std/task_basics.fpas
fpas examples/math/mandelbrot/mandelbrot_graph.fpas
fpas examples/pascal/tui/host_dispatch_minimal.fpas
fpas examples/pascal/tui/host_dispatch_paint.fpas
fpas examples/pascal/tui/host_dispatch_quit.fpas
fpas examples/pascal/tui/local_view_paint.fpas
fpas examples/pascal/tui/view_scoped_commands.fpas
fpas examples/pascal/tui/show_modal_existing_view.fpas
fpas examples/pascal/tui/show_dialog.fpas
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
| `pascal/basics/literals_alias_string_index.fpas` | Constants, number literals, type aliases, and string indexing |
| `pascal/control-flow/while_repeat_example.fpas` | `while` and `repeat until` loops |
| `pascal/functions/mutable_nested_functions.fpas` | Mutable parameters, nested functions, and mutual recursion |
| `pascal/functions/nested_functions.fpas` | Nested helper function (`Square` inside `Hypotenuse`) |
| `pascal/higher-order-functions/higher_order_functions.fpas` | First-class functions and array helpers |
| `pascal/enum-data/` | Enums with associated data and `case` |
| `pascal/error-handling/` | `Result`, `Option`, `panic`, and `try` |
| `pascal/for/for_example.fpas` | Index-based counting `for` with `break` / `continue` |
| `pascal/for/downto_example.fpas` | `for I := N downto M` |
| `pascal/for-in/for_in_example.fpas` | `for V in array` |
| `pascal/for-in/dict_for_in_example.fpas` | `for K in dict` (key iteration) |
| `pascal/concurrency/go_statement_example.fpas` | Fire-and-forget `go` (no `task` handle) |
| `pascal/generics/generic_functions.fpas` | Generic functions |
| `pascal/generics/generic_record_methods.fpas` | Method-level generics and constraints on record methods |
| `pascal/pattern-matching/` | Guards and exhaustiveness |
| `pascal/record-methods/` | Record methods |
| `pascal/records/defaults_with_update.fpas` | Default fields and `with` updates |
| `pascal/std/args_basics.fpas` | `Std.Args` — arguments passed after `--` |
| `pascal/std/str_basics.fpas` | `Std.Str` — trim, split/join, `Format`, search/replace |
| `pascal/std/dict_basics.fpas` | `Std.Dict` — literals, `Get`, `Merge`, `Map`/`Filter` (qualified when also using `Std.Array` / `Std.Option`) |
| `pascal/std/env_basics.fpas` | `Std.Env` — environment lookup and missing values |
| `pascal/std/fs_basics.fpas` | `Std.Fs` — create directories, write/read UTF-8 text, path checks |
| `pascal/std/graph_basics.fpas` | `Std.Graph` — open, draw, present, poll, close |
| `pascal/std/json_basics.fpas` | `Std.Json` — parse, inspect, and stringify JSON trees |
| `pascal/std/parse_basics.fpas` | `Std.Parse` — `Result`-based integer, real, and boolean parsing |
| `pascal/std/path_basics.fpas` | `Std.Path` — join, normalize, basename, dirname, extension |
| `pascal/std/proc_basics.fpas` | `Std.Proc` — process launch failure as `Result` |
| `pascal/std/random_basics.fpas` | `Std.Random` — random real and inclusive integer ranges |
| `pascal/std/task_basics.fpas` | `Std.Task` — `go`, `Wait`, `WaitAll` |
| `pascal/std/time_basics.fpas` | `Std.Time` — monotonic time, elapsed time, timestamp, sleep |
| `pascal/std/array_basics.fpas` | `Std.Array` — `Length`, `Sort`, `Any`, `All` |
| `math/mandelbrot/mandelbrot_graph.fpas` | `Std.Graph` — native Mandelbrot explorer |
| `pascal/tui/local_view_paint.fpas` | `Std.Tui` — local view paint, parent-relative layout, `HostSetViewRect` |
| `pascal/tui/view_scoped_commands.fpas` | `Std.Tui` — `HostBindCommandToView` and focus/ancestor command routing |
| `pascal/tui/show_modal_existing_view.fpas` | `Std.Tui` — `ShowModal` for an existing view subtree |
| `pascal/tui/show_dialog.fpas` | `Std.Tui` — `ShowDialog` plus modal-local command binding |
| `pascal/tui/poll_redraw_timeout.fpas` | Poll-style `ReadEventTimeout` + `RedrawPending` loop (**Escape** exits) |
| `math/julia/julia.fpas` | ASCII Julia set (**interactive** — see below) |
| `math/julia/julia_graph.fpas` | Native-window Julia explorer with `Std.Graph` |

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
| `math/julia/julia_graph.fpas` | Single-file native-window Julia explorer; arrows pan, `WASD` changes Julia constant, wheel zooms, left click recenters, `Esc` quits |
| `math/mandelbrot/mandelbrot_graph.fpas` | Single-file native-window Mandelbrot explorer; arrows pan, wheel zooms, left click recenters, `1/2/3` switch palettes, `Esc` quits |
| `math/mandelbrot/mandelbrot.fpasprj` | Project; fullscreen Mandelbrot explorer |
| `pascal/tui/minimal_application.fpas` | `Application.Configure` + `Application.Run` dispatch mode; **Escape** to quit |
| `pascal/tui/host_dispatch_minimal.fpas` | One **`HostProcessNext`** call then **`Close`** (dispatch bridge); same TUI session behavior as `minimal_application.fpas` |
| `pascal/tui/host_dispatch_paint.fpas` | **`HostRegisterOnPaint`** + **`HostDispatchRedraw`** (one paint pass) |
| `pascal/tui/host_dispatch_quit.fpas` | **`HostRequestQuit`** from **`OnPaint`**, then **`HostRunLoop`** (cooperative exit) |
| `pascal/tui/local_view_paint.fpas` | Local view paint only; press **M** to move a child view and **Escape** to quit |
| `pascal/tui/view_scoped_commands.fpas` | Focus-aware view commands; **Tab** changes focus, **Ctrl+S** resolves per panel, **Escape** quits |
| `pascal/tui/show_modal_existing_view.fpas` | Existing view subtree becomes modal; **Tab** stays in the subtree, **Escape** closes the modal |
| `pascal/tui/show_dialog.fpas` | Owned modal dialog; **Ctrl+D** opens it, **Escape** closes it, **Ctrl+Q** quits |
| `pascal/tui/poll_redraw_timeout.fpas` | Poll-style loop; **Escape** exits (see `docs/pascal/std/tui.md`) |

TUI apps use the dispatch model: `Application.Configure(App, Handlers)` registers `On*` handlers; `Application.Run(App)` starts the hosted loop. See `docs/pascal/std/tui-app.md` for the full dispatch API and `docs/pascal/std/tui.md` for poll-style API status. The console's own event type remains **`Std.Console.Event`**.

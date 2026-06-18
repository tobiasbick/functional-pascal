# Examples

Functional Pascal samples aligned with the **current** compiler and standard library (`Std.*`).

## Automated smoke test (non-interactive only)

Many examples under `examples/` are **interactive** (TUI alternate screen, native graph window, key loops). **Do not** glob-run all `*.fpas` files in a shell loop or batch script — that will hang on demos such as `math/mandelbrot/mandelbrot.fpasprj`.

Use the curated allowlist in [`crates/fpas-cli/src/main_tests/examples.rs`](../crates/fpas-cli/src/main_tests/examples.rs):

```sh
cargo test -p fpas-cli non_interactive_examples_run_successfully
```

Or:

```sh
./scripts/run-non-interactive-examples.sh    # Unix
pwsh scripts/run-non-interactive-examples.ps1
```

When you add a new **console** example that exits on its own, append it to `NON_INTERACTIVE_EXAMPLES` in that file. Library and workspace manifests used only for `fpas check` go in `NON_INTERACTIVE_CHECK_EXAMPLES` in the same file. Interactive demos stay documented in the table below only.

## Stdlib regression suite (`tests/`)

The **FPAS regression suite** lives in [`tests/`](../tests/) as `*_test.fpas` files with optional golden sidecars (`*.expect.stdout`, `*.expect.pixels`). Layout:

| Directory | Contents |
|-----------|----------|
| `tests/stdlib/` | `Std.*` runtime checks (~330 programs) |
| `tests/concurrency/` | `go` / task concurrency |
| `tests/runner/` | `Std.Test` basics, `Skip`, stdout golden |
| `tests/console/` | `PushReadLn` + `ReadLn` |
| `tests/tui/` | Native headless TUI (`OpenForTest`, `TestPump`, …) |
| `tests/graph/` | Headless graph smoke + pixel golden |
| `tests/manual/` | Manual demos (not auto-discovered) |

Run via:

```sh
fpas test tests/
fpas test tests/suite.fpasprj
fpas test tests/runner/assert_basics_test.fpas
cargo test -p fpas-cli fpas_regression_suite_passes
```

Test files are named `*_test.fpas`. `Skip` tests are reported as skipped (use `--strict` to fail the run). See [`docs/pascal/std/testing/test.md`](../docs/pascal/std/testing/test.md) and [`docs/future/test-framework/README.md`](../docs/future/test-framework/README.md).

`tests/manual/assert_fail_demo.fpas` is a manual failure demo (not `*_test.fpas`); run it with `fpas tests/manual/assert_fail_demo.fpas` to inspect **F4023** output.

Expected failures (runtime/compile errors, CLI args) are exercised from `test_suite_negative.rs`.

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
fpas apps/ide/ide.fpasprj
```

Do **not** pass a `unit` source alone (for example `mandelbrot_color.fpas` or `math_utils.fpas`) — the compiler expects a `program` as the main file.

### Library dependency (path-based)

Program and library as separate `.fpasprj` files linked via `[dependencies].projects`:

```sh
fpas examples/pascal/library-deps/app/app.fpasprj
fpas check examples/pascal/library-deps/mylib/mylib.fpasprj
```

See [pascal/library-deps/README.md](pascal/library-deps/README.md).

### Monorepo (library dependency + workspace)

When a program lives in one project and reusable units live in another, use `kind = "library"` plus `[dependencies].projects` on the program `.fpasprj`. Optional `.fpasworkspace` lists all projects for `fpas check`:

```sh
fpas examples/pascal/monorepo/apps/hello/hello.fpasprj
cd examples/pascal/monorepo && fpas check
```

See [pascal/monorepo/README.md](pascal/monorepo/README.md) and [docs/pascal/program-structure/projects.md](../docs/pascal/program-structure/projects.md).

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
| `pascal/std/graph_basics.fpas` | `Std.Graph` — hosted `Configure` + `Run`, draw on `OnPaint`, quit on Escape |
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
| `math/julia/julia.fpas` | ASCII Julia set (**interactive** — see below) |
| `math/julia/julia_graph.fpas` | Native-window Julia explorer with `Std.Graph` |

## Multi-unit projects

| Path | Contents |
|------|----------|
| `pascal/units-basic/` | `units-basic.fpasprj`, `unit App.Math`, `App.Reporting`, program `UnitsBasic` |
| `pascal/library-deps/` | Program `LibDemo` + library `MyLib.Core` via `[dependencies].projects` |
| `pascal/monorepo/` | Workspace + `Demo.Greet` library + `Hello` via `[dependencies].workspace` |
| `math/mandelbrot/` | `mandelbrot.fpasprj`, program `MandelbrotShowcase`, units `Mandelbrot.Color` and `Mandelbrot.Render` |

Helper units under those folders are built only through the project; see the one-line `{ ... }` comment at the top of each unit file.

## Applications (`apps/`)

Larger programs live outside `examples/` but follow the same `.fpasprj` workflow.

| Path | Contents |
|------|----------|
| `apps/ide/` | `ide.fpasprj` — Turbo Pascal–style hosted TUI shell (`Ide.Shell`, `Ide.Menu`, `Ide.Status`, `Ide.Theme`) |

Run from the repository root:

```sh
fpas apps/ide/ide.fpasprj
```

The IDE is **interactive**: it opens the alternate screen and blocks in `Application.Run` until you quit. Use **Alt+X** or **File → Exit** from the menu bar. Host widgets paint the chrome (menu bar, blue desktop, status bar); `OnPaint` is intentionally empty — see `apps/ide/src/shell.fpas` and `docs/pascal/std/tui/app.md`.

CI compiles it with `fpas check apps/ide/ide.fpasprj` (listed in `NON_INTERACTIVE_CHECK_EXAMPLES` in [`crates/fpas-cli/src/main_tests/examples.rs`](../crates/fpas-cli/src/main_tests/examples.rs)). For a smaller single-file menu bar sample, see `pascal/tui/menu_bar.fpas`.

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
| `pascal/tui/menu_bar.fpas` | **`HostCreateMenuBarView`** with Alt+letter shortcuts and pull-down **`OnCommand`** dispatch; **Escape** to quit |
| `pascal/tui/local_view_paint.fpas` | Local view paint only; press **M** to move a child view and **Escape** to quit |
| `pascal/tui/view_scoped_commands.fpas` | Focus-aware view commands; **Tab** changes focus, **Ctrl+S** resolves per panel, **Escape** quits |
| `pascal/tui/show_modal_existing_view.fpas` | Existing view subtree becomes modal; **Tab** stays in the subtree, **Escape** closes the modal |
| `pascal/tui/show_dialog.fpas` | Owned modal dialog; **Ctrl+D** opens it, **Escape** closes it, **Ctrl+Q** quits |
| `apps/ide/ide.fpasprj` | Multi-unit IDE shell — menu bar, desktop fill, status bar; **Alt+X** or **File → Exit** quits |
TUI and Graph apps use the same hosted dispatch model: `Application.Configure(App, Handlers)` registers `On*` handlers; `Application.Run(App)` starts the hosted loop. See `docs/pascal/std/tui/app.md` and `docs/pascal/std/graph/app.md`. The console's own event type remains **`Std.Console.Event`**.

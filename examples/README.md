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
| `tests/tui/` | Headless Turbo Vision (`views/`, `events/`, `smoke/`, `modals/` — `OpenForTest`, `TestClickButton`, `Application.Run`, `AssertScreenCell`, …) |
| `tests/graph/` | Headless graph smoke + pixel golden |
| `tests/manual/` | Manual demos (not auto-discovered) |

Run via:

```sh
fpas test tests/
fpas test tests/suite.fpasprj
fpas test tests/runner/assert_basics_test.fpas
cargo test -p fpas-cli fpas_regression_suite_passes
```

Test files are named `*_test.fpas`. `Skip` tests are reported as skipped (use `--strict` to fail the run). See [`docs/pascal/std/testing/test.md`](../docs/pascal/std/testing/test.md).

`tests/manual/assert_fail_demo.fpas` is a manual failure demo (not `*_test.fpas`); run it with `fpas run tests/manual/assert_fail_demo.fpas` to inspect **F4023** output.

Expected failures (runtime/compile errors, CLI args) are exercised from `test_suite_negative.rs`.

## How to run

### Single-file programs

Use when the file begins with `program` and only imports `Std.*` (or needs no other units):

```sh
fpas run examples/hello.fpas
fpas run examples/fibonacci.fpas
fpas run examples/pascal/std/args_basics.fpas -- one two
fpas run examples/pascal/std/str_basics.fpas
fpas run examples/pascal/std/dict_basics.fpas
fpas run examples/pascal/std/json_basics.fpas
fpas run examples/pascal/std/graph_basics.fpas
fpas run examples/pascal/std/task_basics.fpas
fpas run examples/math/mandelbrot/mandelbrot_graph.fpas
```

### Projects (`.fpasprj`)

Use when the main program imports **non-library units** (for example `App.*` or `Mandelbrot.*`). The project file links all sources:

```sh
fpas run examples/pascal/units-basic/units-basic.fpasprj
fpas run examples/math/mandelbrot/mandelbrot.fpasprj
```

Do **not** pass a `unit` source alone (for example `mandelbrot_color.fpas` or `math_utils.fpas`) — the compiler expects a `program` as the main file.

### Library dependency (path-based)

Program and library as separate `.fpasprj` files linked via `[dependencies].projects`:

```sh
fpas run examples/pascal/library-deps/app/app.fpasprj
fpas check examples/pascal/library-deps/mylib/mylib.fpasprj
```

See [pascal/library-deps/README.md](pascal/library-deps/README.md).

### Monorepo (library dependency + workspace)

When a program lives in one project and reusable units live in another, use `kind = "library"` plus `[dependencies].projects` on the program `.fpasprj`. Optional `.fpasworkspace` lists all projects for `fpas check`:

```sh
fpas run examples/pascal/monorepo/apps/hello/hello.fpasprj
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
| `pascal/tui/turbo_vision_dialog.fpas` | `Std.Tui` — window with controls, menu bar, status line, `Run(App, OnCommand)` |
| `pascal/tui/turbo_vision_outline.fpas` | `Std.Tui` — outline tree in a window |
| `pascal/tui/turbo_vision_window.fpas` | `Std.Tui` — window, static text, input line, menu/status chrome |
| `pascal/tui/exec_dialog.fpas` | `Std.Tui` — modal `ExecView` with `InputLine.Text` / `CheckBox.Checked` read-back |
| `pascal/tui/modal_button.fpas` | `Std.Tui` — `Dialog.NewModal`, `Button.New`, `ExecView` |
| `pascal/tui/message_box.fpas` | `Std.Tui` — upstream message box helper and `CM_OK` result |
| `pascal/tui/file_dialog.fpas` | `Std.Tui` — upstream file dialog helper |
| `pascal/tui/runtime_setters.fpas` | `Std.Tui` — live `StaticText.SetText` / `CheckBox.SetChecked` from `OnCommand` during `Run` |
| `pascal/tui/mouse_toggle_smoke.fpas` | `Std.Tui` — interactive mouse read-back smoke for checkbox/radio |
| `math/mandelbrot/mandelbrot_graph.fpas` | `Std.Graph` — native Mandelbrot explorer |
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

`apps/ide/` is a Turbo Vision-backed IDE shell organized by theme under `src/` (`shell/`, `ui/`, `dialog/`, `workspace/`). Units use the `Ide.*` namespace (for example `Ide.Ui.Menu`, `Ide.Workspace.Session`). **File → Open** accepts `.fpas`, `.fpasprj`, and `.fpasworkspace` paths; the IDE keeps an in-memory session (one workspace or project root plus open source files) and reflects it in the status line. It uses [`Std.Tui`](../docs/pascal/std/tui/README.md) over [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust). Run with `cargo run -p fpas-cli -- run apps/ide/ide.fpasprj`.

## Interactive demos (terminal)

These run until you exit (for example **Escape**). Run from a real terminal if possible.

| Path | Notes |
|------|--------|
| `math/julia/julia.fpas` | Single-file; pan/zoom with keys after first draw |
| `pascal/tui/turbo_vision_dialog.fpas` | Window with list box and controls; Quit exits |
| `pascal/tui/turbo_vision_outline.fpas` | Outline tree in a window; Close exits |
| `pascal/tui/turbo_vision_window.fpas` | Window with menu bar, status line, input line; Quit exits |
| `pascal/tui/exec_dialog.fpas` | Modal dialog; OK/Cancel closes, entered name printed after |
| `pascal/tui/modal_button.fpas` | Modal; OK/Cancel via mouse or Enter/Esc |
| `pascal/tui/message_box.fpas` | Message box; OK closes and prints the result |
| `pascal/tui/file_dialog.fpas` | File dialog; choose a file or cancel |
| `pascal/tui/runtime_setters.fpas` | Window; Toggle updates label/check box live, Quit exits |
| `pascal/tui/mouse_toggle_smoke.fpas` | Interactive mouse read-back smoke for checkbox/radio (manual terminal check) |
| `math/julia/julia_graph.fpas` | Single-file native-window Julia explorer; arrows pan, `WASD` changes Julia constant, wheel zooms, left click recenters, `Esc` quits |
| `math/mandelbrot/mandelbrot_graph.fpas` | Single-file native-window Mandelbrot explorer; arrows pan, wheel zooms, left click recenters, `1/2/3` switch palettes, `Esc` quits |
| `math/mandelbrot/mandelbrot.fpasprj` | Project; fullscreen terminal Mandelbrot explorer (`Std.Console` event loop) |

Automated Turbo Vision coverage is under `tests/tui/` (`views/`, `events/`, `smoke/`, `modals/`). Graph apps use `Application.Configure(App, Handlers)` and `Application.Run(App)`; see `docs/pascal/std/graph/app/README.md`. Custom terminal loops use `Std.Console`; see `docs/pascal/std/console/README.md`.

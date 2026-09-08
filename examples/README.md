# Examples

Functional Pascal samples aligned with the **current** compiler and standard library (`Std.*`).

## Automated smoke test (non-interactive only)

Many examples under `examples/` are **interactive** (TUI alternate screen and key loops). **Do not** glob-run all `*.fpas` files in a shell loop or batch script — that will hang on demos such as `math/mandelbrot/mandelbrot.fpasprj`.

Use the curated allowlist in [`crates/fpas-cli/src/main_tests/examples.rs`](../crates/fpas-cli/src/main_tests/examples.rs):

```sh
cargo test -p fpas-cli example_
```

Or:

```sh
./scripts/run-non-interactive-examples.sh    # Unix
pwsh scripts/run-non-interactive-examples.ps1
```

When you add a new **console** example that exits on its own, add an entry to `example_run_tests!`
in that file or a dedicated `example_*` test in its submodules when asserting output. Examples and
manifests used only for `fpas check` use `example_check_tests!` or a dedicated `example_check_*` test.
Interactive demos are checked without opening their terminal UI.

## Stdlib regression suite (`tests/`)

The **FPAS regression suite** lives in [`tests/`](../tests/) as `*_test.fpas` files with optional golden sidecars. Layout:

| Directory | Contents |
|-----------|----------|
| `tests/stdlib/` | `Std.*` runtime checks, including headless `Std.Tui` coverage |
| `tests/concurrency/` | Tasks, channels, cancellation, selection, task groups, supervision, and timed close |
| `tests/runner/` | `Std.Test` basics, `Skip`, stdout golden |
| `tests/console/` | `PushReadLn` + `ReadLn` |
| `tests/apps/` | Application workflow tests |
| `tests/debugger/` | Source debugger fixtures |
| `tests/manual/` | Manual demos (not auto-discovered) |

Run via:

```sh
fpas test tests/
fpas test tests/suite.fpasprj
fpas test tests/runner/assert_basics_test.fpas
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
fpas run examples/pascal/std/console_cells_basics.fpas
fpas run examples/pascal/std/dict_basics.fpas
fpas run examples/pascal/std/json_basics.fpas
fpas run examples/pascal/std/task_basics.fpas
```

### Network examples

The [`network/`](network/README.md) examples include paired HTTP and raw TCP
clients and servers, HTTPS programs with explicit trust and PEM requirements,
an incremental SSE decoder, and URI/UTF-8 helpers. The local pairs accept a
port after `--`, so client and server can run in separate terminals without
changing source files.

### Performance benchmarks

Prefer the curated harness (before/after save/compare). Full steps: [`docs/bench/README.md`](../docs/bench/README.md).

```sh
cargo bench-fpas save before          # known-good checkout
# … change code …
cargo build --release -p fpas-cli
cargo bench-fpas compare before
```

VM-only (faster):

```sh
cargo bench-fpas run --group vm
```

Manual single-file runs still work. Build the release CLI first so compiler debug checks do not dominate:

```sh
cargo build --release -p fpas-cli
target/release/fpas run examples/pascal/tui/headless_render_benchmark.fpas -- 500
target/release/fpas run examples/pascal/tui/notes-headless/notes-headless-benchmark.fpasprj -- 250
target/release/fpas run examples/pascal/vm/integer_loop_benchmark.fpas -- 50000000
target/release/fpas run examples/pascal/vm/array_push_benchmark.fpas -- 2000000
target/release/fpas run examples/pascal/vm/array_length_benchmark.fpas -- 500000
target/release/fpas run examples/pascal/vm/string_concat_benchmark.fpas -- 5000000
target/release/fpas run examples/pascal/vm/string_length_benchmark.fpas -- 500000
target/release/fpas run examples/pascal/vm/function_call_benchmark.fpas -- 3000000
target/release/fpas run examples/pascal/vm/array_callbacks_benchmark.fpas -- 50000
target/release/fpas run examples/pascal/vm/record_update_benchmark.fpas -- 1000000
target/release/fpas run examples/pascal/vm/unicode_char_at_benchmark.fpas -- 3000000
target/release/fpas run examples/pascal/vm/wrapper_payload_benchmark.fpas -- 5000000
target/release/fpas run examples/pascal/concurrency/task_spawn_wait_benchmark.fpas -- 100000
target/release/fpas run examples/pascal/concurrency/task_array_callbacks_benchmark.fpas -- 20000
```

On Windows, invoke `target/release/fpas.exe`. Optional second argument `MAX_MILLIS` turns a slowdown into a panic. Compare several runs on the same machine with the same release binary and power settings; do not share one fixed threshold across unlike machines.

### Projects (`.fpasprj`)

Use when the main program imports **non-library units** (for example `App.*` or `Mandelbrot.*`). The project file links all sources:

```sh
fpas run examples/pascal/units-basic/units-basic.fpasprj
fpas run examples/math/mandelbrot/mandelbrot.fpasprj
fpas run examples/math/julia/julia.fpasprj
fpas run examples/math/burning_ship/burning_ship.fpasprj
fpas run examples/math/tricorn/tricorn.fpasprj
fpas run examples/math/newton/newton.fpasprj
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
| [`pascal/concurrency/select_events.fpas`](pascal/concurrency/select_events.fpas) | Typed channel delivery selected against a timer |
| [`pascal/concurrency/task_group_workers.fpas`](pascal/concurrency/task_group_workers.fpas) | Group-owned workers, cooperative cancellation, and collected failures |
| [`pascal/concurrency/supervised_worker.fpas`](pascal/concurrency/supervised_worker.fpas) | Bounded worker retries with one task identity |
| [`pascal/concurrency/worker_pipeline.fpas`](pascal/concurrency/worker_pipeline.fpas) | Three workers, capacity-two queues, concurrent collection, and timed close; prints `650` |
| [`pascal/concurrency/task_group_close_timeout.fpas`](pascal/concurrency/task_group_close_timeout.fpas) | A deterministic zero-timeout close retains a cleanup gate until release and join |
| `pascal/concurrency/task_memory_benchmark.fpas` | Parameterized cooperative task-memory benchmark; measure peak RSS externally |
| `pascal/concurrency/task_spawn_wait_benchmark.fpas` | Spawn plus `WaitAll` task-scheduling throughput |
| `pascal/concurrency/task_array_callbacks_benchmark.fpas` | Resumable `Map`, `Filter`, and `Reduce` callback throughput inside a spawned task |
| `pascal/tui/headless_render_benchmark.fpas` | Parameterized headless `Std.Tui` render benchmark with an optional elapsed-time limit |
| `pascal/tui/notes-headless/notes-headless-benchmark.fpasprj` | Real Notes application rendering through a headless `Std.Tui` project benchmark |
| `pascal/tui/mandelbrot-headless/mandelbrot-benchmark.fpasprj` | Production Mandelbrot tasks and bounded delivery, optionally including headless grid painting; `cargo bench-fpas run --group mandelbrot` |
| `pascal/vm/integer_loop_benchmark.fpas` | Tight integer arithmetic loop for VM dispatch / int-op throughput |
| `pascal/vm/array_push_benchmark.fpas` | Growing `Std.Array.Push` for VM array locals / SharedArray COW |
| `pascal/vm/array_length_benchmark.fpas` | Repeated `Std.Array.Length` on a shared live array (read-only COW path) |
| `pascal/vm/string_concat_benchmark.fpas` | Short string concat + `IntToStr` for VM string ops |
| `pascal/vm/string_length_benchmark.fpas` | Repeated `Std.Str.Length` on a long live string |
| `pascal/vm/intrinsic_dispatch_benchmark.fpas` | Mixed array, string, and dictionary length calls for intrinsic-routing throughput |
| `pascal/vm/function_call_benchmark.fpas` | Direct and captured function-call throughput |
| `pascal/vm/array_callbacks_benchmark.fpas` | `Map`, `Filter`, and `Reduce` callback throughput |
| `pascal/vm/record_update_benchmark.fpas` | Record construction, field access, and `with` update throughput |
| `pascal/vm/unicode_char_at_benchmark.fpas` | `Std.Str.CharAt` throughput over multi-byte Unicode text |
| `pascal/vm/wrapper_payload_benchmark.fpas` | `Result` and `Option` payload construction and unwrap throughput |
| `pascal/generics/generic_functions.fpas` | Generic functions |
| `pascal/generics/generic_record_methods.fpas` | Method-level generics and constraints on record methods |
| `pascal/pattern-matching/` | Guards and exhaustiveness |
| `pascal/record-methods/` | Record methods |
| `pascal/records/defaults_with_update.fpas` | Default fields and `with` updates |
| `pascal/std/args_basics.fpas` | `Std.Args` — arguments passed after `--` |
| `pascal/std/console_cells_basics.fpas` | `Std.Console` — framed cell fill/write/read-back and saved-region restore |
| `pascal/std/str_basics.fpas` | `Std.Str` — trim, split/join, `Format`, search/replace |
| `pascal/std/dict_basics.fpas` | `Std.Dict` — literals, `Get`, `Merge`, `Map`/`Filter` (qualified when also using `Std.Array` / `Std.Option`) |
| `pascal/std/env_basics.fpas` | `Std.Env` — environment lookup and missing values |
| `pascal/std/fs_basics.fpas` | `Std.Fs` — create directories, write/read UTF-8 text, path checks |
| `pascal/std/json_basics.fpas` | `Std.Json` — parse, inspect, and stringify JSON trees |
| `pascal/std/parse_basics.fpas` | `Std.Parse` — `Result`-based integer, real, and boolean parsing |
| `pascal/std/path_basics.fpas` | `Std.Path` — join, normalize, basename, dirname, extension |
| `pascal/std/proc_basics.fpas` | `Std.Proc` — process launch failure as `Result` |
| `pascal/std/random_basics.fpas` | `Std.Random` — random real and inclusive integer ranges |
| `pascal/std/task_basics.fpas` | `Std.Task` — `go`, `Wait`, `WaitAll` |
| `pascal/std/time_basics.fpas` | `Std.Time` — monotonic time, elapsed time, timestamp, sleep |
| `pascal/std/array_basics.fpas` | `Std.Array` — `Length`, `Sort`, `Any`, `All` |
| `network/http_server.fpas` + `network/http_client.fpas` | Local HTTP server plus buffered and streaming client |
| `network/tcp_echo_server.fpas` + `network/tcp_echo_client.fpas` | Raw TCP lifecycle, timeouts, UTF-8, and partial writes |
| [`network/tcp_parallel_echo_server.fpas`](network/tcp_parallel_echo_server.fpas) | Four connection workers, owned listener, signal-aware shutdown, and explicit process escalation on loopback |
| `network/https_server.fpas` + `network/https_client.fpas` | TLS listener credentials and verified HTTPS requests |
| `network/sse_decoder.fpas` | Incremental Server-Sent Events decoding across fragments |
| `network/uri_utf8.fpas` | Absolute URI parsing and UTF-8 conversion |

All `math/` fractal demos are multi-unit `.fpasprj` projects — see the table below.

## Multi-unit projects

| Path | Contents |
|------|----------|
| `pascal/units-basic/` | `units-basic.fpasprj`, `unit App.Math`, `App.Reporting`, program `UnitsBasic` |
| `pascal/library-deps/` | Program `LibDemo` + library `MyLib.Core` via `[dependencies].projects` |
| `pascal/monorepo/` | Workspace + `Demo.Greet` library + `Hello` via `[dependencies].workspace` |
| `math/mandelbrot/` | `mandelbrot.fpasprj` depends on `mandelbrot-core.fpasprj`: canonical background-enabled `Std.Tui` app with focused App, Model, Color, Render, and View units |
| `math/julia/` | `julia.fpasprj` depends on `julia-core.fpasprj`: `Julia.Color`, `Julia.Compute`, and `Julia.Render` |
| `math/burning_ship/` | `burning_ship.fpasprj`, program `BurningShipShowcase`, units `BurningShip.Color` / `BurningShip.Render` |
| `math/tricorn/` | `tricorn.fpasprj`, program `TricornShowcase`, units `Tricorn.Color` / `Tricorn.Render` |
| `math/newton/` | `newton.fpasprj`, program `NewtonShowcase`, units `Newton.Color` / `Newton.Render` |

Build the helper units through their project manifests.

## Interactive demos (terminal)

These run until you exit (for example **Escape**). Run from a real terminal if possible.

| Path | Notes |
|------|--------|
| `openai-chat/openai-chat.fpasprj` | Line-oriented chat against a configurable OpenAI-compatible HTTP endpoint |
| `math/mandelbrot/mandelbrot.fpasprj` | Canonical `Std.Tui` MVU example with a bounded typed inbox, cancellable row tasks on the VM pool, atomic image updates, gauges, and overlay |
| `math/julia/julia.fpasprj` | Fullscreen Julia explorer with four cancellable render workers; `WASD` adjusts the constant |
| `math/burning_ship/burning_ship.fpasprj` | Fullscreen terminal Burning Ship explorer |
| `math/tricorn/tricorn.fpasprj` | Fullscreen terminal Tricorn explorer |
| `math/newton/newton.fpasprj` | Fullscreen terminal Newton basins for `z^3-1` |

Custom terminal loops use `Std.Console`; see `docs/pascal/std/console/README.md`.

## Concurrency examples in practice

The worker pipeline feeds and collects concurrently. Both queues have capacity two; a slow
collector blocks workers, which in turn slows the producer. Closing the task group requests
cancellation, so the collector receives all twelve results before starting close. The separate
timeout demo deliberately waits on a non-cancellable cleanup gate: timeout is not termination,
and the owner releases the gate and joins before closing its channel.

Julia starts four workers per view, with a capacity-four row channel. The main task draws at most
four available rows per event-loop iteration; it does not wait for a whole image. Zoom, resize,
palette changes, and exit cancel and join the old render before releasing its queue. Workers
check cancellation between pixels and every 32 fractal iterations, and their result sends are
cancellable. Each replacement owns a fresh queue, so old rows cannot overwrite the new view.
This uses the existing Console loop, not a new `Std.Tui` background-message API. No rendering
speedup is claimed; the change concerns work bounds and event processing.

Run the additional TCP example with an optional port and lifetime in milliseconds:

```sh
fpas run examples/network/tcp_parallel_echo_server.fpas -- 18082 10000
```

It echoes raw bytes as they arrive, without an `Echo:` prefix or line framing. Each of four workers
accepts and owns one connection at a time, processes chunks of at most 4096 bytes, handles partial
writes, and closes the connection even on an I/O error. Idle reads/writes have a five-second timeout.
At lifetime expiry the owner cancels accept and connection I/O, tries a one-second group close,
then retains and joins any unfinished workers before closing the listener. Active requests may be
interrupted: this is a cancellation demo, not graceful draining or a hard process-exit guarantee.
Hosted network calls occupy VM workers; actual parallelism also depends on the runtime pool.
The original single-request TCP tutorial remains unchanged.

`cargo test -p fpas-cli parallel_echo_example` tests the checked-in server with loopback clients,
including a second client while the first is idle, multi-chunk echo, and cancellation during reads
and accepts. Julia's row-equivalence and full-queue cancellation tests are part of
`fpas test tests/suite.fpasprj --filter julia`.

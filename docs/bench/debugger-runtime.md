# Debugger runtime

The debugger still schedules one instruction at a time. Source, function and data
breakpoints, tracing, pause and execution limits keep their instruction boundaries.
Execution access now spans at most 256 scheduler iterations. The runtime publishes
the retained task roots before releasing that access, returning a stop or error,
or waiting for input. Pending maintenance can publish earlier. Newly spawned tasks
use their ordinary ownership boundary until the next group begins.

This removes repeated root snapshots during sustained continue. The supervision
wrapper also leaves publication to its enclosing execution owner instead of
publishing the same worker twice. The ordinary `fpas run` path is unchanged.

## Local DAP gameplay comparison, 2026-10-24

The workload is Into the Dungeon at game commit
`6635dea83210f7f6c50a6632196fcb448cc50054`, seed 25, dungeon region size 1.
The baseline runtime is FPAS commit `5ae070828a30a6664cdc5bfc42d040596604edc1`.
Both versions execute the same copied client image, with SHA-256
`3CE0659E638ECF4317F0AE462684DFAFC947421366CECA599C2A767BB8A8D42F`.
Sources and collection thresholds are unchanged.

The FPAS acceptance controller launches a normal server and the DAP client with a
4-by-4 virtual terminal. Each run starts from a fresh copy of the same save at
`(10912,-10144)`, health 20. The timed command queues eight westward keyboard
events and waits for the final menu breakpoint, completed view and confirmed
position `(10904,-10144)`. It includes protocol traffic, inspection and controller
file polling, and excludes startup and loading the save. No debugger assignments
or direct game-state mutations are used. The controller and server use the same
PATH runtime in both cases; only the debugger child executable changes.

Measurements alternate baseline and changed runtime, with no concurrent builds,
tests or benchmark jobs. Both release executables have the temporary phase timers
removed. These are end-to-end wall times, not CPU samples.

| Eight confirmed steps | Baseline (ms) | Changed (ms) |
|---|---:|---:|
| Run 1 | 17905 | 1472 |
| Run 2 | 17972 | 1521 |
| Run 3 | 17454 | 1570 |
| Median | 17905 | 1521 |

The median falls by 91.5%, a factor of 11.8 for this workload. All six runs reach
the same position with health 20 and close their owned processes normally.
A separate 64-key packet reaches `(10848,-10144)` with health 20 in 9219 ms.
That longer packet is a functional smoke check with one timing, not a paired
performance comparison. These results do not predict the gain at other terminal
sizes or in arbitrary FPAS applications.

## Ordinary execution controls

All 85 configured workloads completed with `cargo bench-fpas compare dap-before`,
using the original IDs and arguments. The first comparison showed 6.9% higher
wall time for Notes, 5.7% for the integer loop, and about 11% for both Mandelbrot
rows. Three alternating executable comparisons checked those deviations:

| Control | Baseline median (ms) | Changed median (ms) |
|---|---:|---:|
| Notes, 250 frames | 2747 | 2762 |
| Integer, 50000000 iterations | 564 | 567 |
| Mandelbrot paint, 12 iterations | 562 | 563 |
| Mandelbrot render, 120 iterations | 5579 | 5491 |

The short render control varied from 423 to 502 ms with the old executable and
422 to 517 ms with the changed one. The separate 120-iteration control uses ten
times the configured render workload, with the same arguments in each old/new
pair and reversed executable order in the middle pair. The configured 85-row
suite remains unchanged. These controls did not reproduce a material slowdown.

## Investigation and reproduction

`fpas run --profile` on the controller establishes end-to-end latency but does
not attribute the debugger child's work. Small scalar, aggregate and TUI debug
probes did not reproduce the sustained gameplay cost. Temporary timers around
the actual debugger scheduler, instruction dispatch and publication showed that
the outer dispatch/readiness root publications alone consumed about 45% of an
instrumented continue. The nested supervision publication added more work.
The timers changed absolute latency and were removed before the comparison.

To reproduce the comparison:

1. Retain the baseline release executable before rebuilding FPAS with `dist.ps1`.
2. Build the client once and keep a copied `.fpascp` image unchanged. Launch both
   debugger versions against that image with the same `--source-root`.
3. Use the game's `tools/dap-acceptance` controller with an isolated copy of the
   seed-25, region-1 spawn save, virtual size 4 by 4 and its menu source breakpoint.
   Select the debugger child executable explicitly; keep the controller and server
   fixed. Continue the save before starting each timer.
4. Measure one `Acceptance.Commands.Batch(Directory, var Id, 'a', 8)` call with
   `Std.Time.MonotonicMillis` and `ElapsedMillis`. Assert the final coordinates
   and health, stop the controller, and repeat from a fresh data directory.
5. Run three alternating pairs and compare medians. Keep normal workload arguments
   unchanged for `cargo bench-fpas compare dap-before`.

The regression test exercises aggressive collection during continued execution,
retained closure aliases across a breakpoint and collection from another thread
while stopped. Existing VM and protocol tests cover stepping, limits, watchpoints,
task ordering, supervision and idle wakeups. For supported target attribution, use
[`fpas debug --profile`](../pascal/tools/profiler.md#debug-targets). Its instrumented
phase timings are separate from the unprofiled latency measurements above.

Validation passes for 916 VM tests (two ignored), the full `fpas-debug` unit and
integration suite, 95 CLI batch-debug tests and VM Clippy with all targets and
warnings denied. After the final `dist.ps1` build, the PATH executable matches
the release binary by SHA-256. All 161 game test programs pass with that
distribution, and a final owned server/client DAP run confirms eight accepted
steps and normal shutdown. That final smoke runs alongside the game tests and
is not included in the paired timing results above.

## Timed terminal input, 2026-10-26

An empty `Std.Console.ReadEventTimeout(50)` returned immediately in the debugger's
queue-only terminal. The TUI input pump therefore repeatedly polled and yielded
while the all-stop scheduler walked the rendering task one instruction at a time.
Positive timed reads now suspend the calling task until an event or deadline.
Polling and result delivery remain on the VM thread, with ordinary task
cancellation and no additional native thread. Zero-timeout reads stay immediate.

The workload is Into the Dungeon's server console at game commit
`c2e3da6e929e1ff96ecf602dd5c3ca956296f110`. The baseline executable is the installed
FPAS distribution at `7c6e886ccdd2db3a8b761b3988d4d628d0fe2c76`, copied unchanged
before rebuilding. Both release runtimes use the same server sources and source
standard library, including the earlier fix that selects due ticks alongside input.
Each run owns a fresh data root and loopback listener; no agent is queried.

A local FPAS controller adapted from the game's `Acceptance.Dap` times F2 followed
by an unbound Ctrl+G marker. Timing starts at the listening view breakpoint and
ends after reading the completed settings view. It includes finishing the pending
dashboard frame, opening and painting settings, routing the marker, protocol
requests, inspection and controller file polling. Compilation and listener startup
are outside this timer. Every run asserts `SettingsOpen = true`, reaches its marker
without a forced pause, disconnects and waits for its owned server process to exit.

Three old/new pairs alternate at each virtual terminal size. No build, test suite
or profiler runs during these measurements; neither debugger nor controller has
profiling enabled. The initial standalone measurements varied more, so this table
uses only the later paired runs under matching conditions.

| Pair | 80x24 before (ms) | 80x24 after (ms) | 144x45 before (ms) | 144x45 after (ms) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 2677 | 1646 | 5812 | 3601 |
| 2 | 2778 | 1653 | 5816 | 3603 |
| 3 | 2678 | 1635 | 5825 | 3618 |
| Median | 2678 | 1646 | 5816 | 3603 |

The medians fall by 38.5% and 38.1%. Larger surfaces still take longer; these
measurements describe this console route rather than arbitrary TUI applications.

The existing `tui` benchmark group checks ordinary execution separately. Its
headless renderer, Notes and styled-cell rows changed from 1694/2867/355 ms to
1427/2690/344 ms, with unchanged IDs and arguments. None regressed. These ordinary
controls do not establish a native-execution gain from the debugger-only repair.
The [history](history.md) records a separate settled control run.

The timeout regression failed before the repair: the read completed before its
deadline and before a peer task. It passes afterward, together with delayed event
delivery, a subsequent zero-timeout poll and cancellation during scope return.
The full Rust workspace passes, including 932 VM tests with three ignored; regular
and release builds and VM Clippy with all targets and warnings denied also pass.

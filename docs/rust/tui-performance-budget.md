# TUI performance budget

Performance targets for the hosted `Std.Tui` dispatch loop.

The public behavior is specified in [`docs/pascal/std/tui-app.md`](../pascal/std/tui-app.md). This document gives Rust implementers a practical budget for changes to the hosted run loop, terminal event pump, redraw scheduling, and callback dispatch.

## Latency target

On a normal development machine, ready input should reach the matching `On*` handler in the same hosted loop turn after any pending paint work has completed.

Reference targets:

| Scenario | Budget |
| -------- | ------ |
| Ready key, mouse, paste, or focus event already queued | Dispatch before any idle wait or timeout. |
| Resize burst followed by input | Dispatch one coalesced `OnResize`, then dispatch the input event on the next process step. |
| No input and no redraw pending | Wait up to the configured idle interval, or the host default timeout when idle is disabled. |
| Handler body runtime | Owned by FPAS user code; host latency measurements should separate this from host dispatch overhead. |

The hosted loop currently uses a bounded process spin count of `64` per run-loop turn. Increasing that value must be justified by a measurable reduction in backlog latency; decreasing it must preserve the ready-input behavior above.

## Regression tests

Use deterministic tests for ordering and starvation rules. Avoid wall-clock assertions in normal unit tests because CI machines and terminal backends vary too much.

Current coverage anchors:

- `tui_host_process_next_dispatches_resize_burst_before_key` verifies resize coalescing at the VM dispatch boundary.
- `tui_application_run_dispatches_ready_key_before_idle_wait` verifies that ready input is processed before the hosted loop waits for idle work.

Manual real-terminal checks live in [`docs/rust/tui-terminal-checklist.md`](tui-terminal-checklist.md).

## Manual measurement

When latency must be measured manually, use a release build and record the OS, terminal, shell, terminal size, and example program. Measure host dispatch separately from handler work whenever possible.

Recommended baseline command:

```sh
cargo build --release -p fpas-cli
```

Then run the relevant TUI example from a real terminal with `target/release/fpas` and capture input-to-visible-update latency with the terminal's own tooling or an external recorder.
# Future Features

Open planning items for Functional Pascal.

## VM implementation (Rust)

[`parallel-vm.md`](../rust/parallel-vm.md) documents the **implemented** parallel task runtime layout in `fpas-vm` (bytecode through shutdown): summary by integration step, paths, and tests. Language rules stay in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md).

## TUI roadmap

Turbo Vision–style direction: Rust-hosted event loop, FPAS `On*` handlers and `RunApp`-style entry, migration away from poll-heavy console usage. See [`tui-application-framework.md`](tui-application-framework.md).

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |
| 10 | [`Native graphics mode`](10-native-graphics-mode.md) | Real windowed 2D graphics for BGI-style APIs and Fractint-like programs |

## Not Yet Planned

| Feature | Description |
|---------|-------------|
| [Libraries](libraries.md) | Project kind `library`, export rules |

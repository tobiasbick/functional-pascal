# Rust implementation docs

Material for **Rust contributors**: where things live in the workspace, how subsystems are wired, and pointers into `crates/*` and tests.

## What belongs here

- **Implemented** runtime or toolchain layout (file paths, phases, test maps), not open-ended product vision.
- Specs and tutorials for **Functional Pascal the language** belong under [`docs/pascal/`](../pascal/).
- **Plans and ideas** for features not yet built belong under [`docs/future/`](../future/).

## Contents

| Document | Purpose |
|----------|---------|
| [`project-loading.md`](project-loading.md) | `fpas-project`: `.fpasprj`, dependencies, workspace, linking. |
| [`parallel-vm.md`](parallel-vm.md) | Parallel task VM in `fpas-vm` / bytecode / compiler touchpoints (through shutdown). |
| [`tui-performance-budget.md`](tui-performance-budget.md) | Latency budget and regression-test anchors for hosted `Std.Tui` dispatch. |
| [`tui-terminal-checklist.md`](tui-terminal-checklist.md) | Real-terminal verification checklist for hosted `Std.Tui` dispatch behavior. |

Add new Rust-facing “atlas” docs here when they describe **current** code structure rather than a roadmap.

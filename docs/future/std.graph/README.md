# `Std.Graph` planning set

**Status:** proposed future feature.

This directory breaks the native graphics work into small, theme-focused documents.
It is the detailed planning set behind [../10-native-graphics-mode.md](../10-native-graphics-mode.md).

## Current decisions

- New standard unit name: `Std.Graph`.
- First backend target: `winit` + `softbuffer`.
- Overall target surface: software-rendered 2D drawing with pixels, lines, shapes, text, and direct input handling.
- First public milestone: one native window, software framebuffer upload, resize/quit/key events.
- Not a goal: BGI compatibility or any retro API emulation target.
- Deliberately deferred: multiple windows, widget/toolkit abstractions, image loading helpers, GPU-first rendering.

## Reading order

1. [01-mvp.md](01-mvp.md) - smallest useful first milestone.
2. [06-use-cases.md](06-use-cases.md) - concrete programs and interaction targets.
3. [02-pascal-surface.md](02-pascal-surface.md) - proposed `Std.Graph` Pascal-facing surface.
4. [03-runtime-architecture.md](03-runtime-architecture.md) - Rust crate boundaries and file layout.
5. [04-implementation-plan.md](04-implementation-plan.md) - incremental implementation slices and verification.
6. [05-backend-selection.md](05-backend-selection.md) - crate selection and rejection rationale.

## Intended outcome

When implementation starts, the user-facing reference should eventually live in `docs/pascal/std/graph.md`.
The files in this directory are the design and rollout plan used to reach that canonical spec and to support programs such as Mandelbrot and Julia explorers.
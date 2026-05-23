# `Std.Graph` Phase 1 MVP

**Status:** proposed first implementation target.

## Goal

Create the smallest native graphics surface that proves all of the following:

- [ ] FPAS can open a real desktop window.
- [ ] FPAS can upload and present a software-rendered frame.
- [ ] FPAS can observe window close, resize, and keyboard input.
- [ ] FPAS can shut the graphics session down cleanly.

This is the foundation slice for a later drawing-oriented `Std.Graph` runtime.
It is not the complete target surface.

## Non-goals

Phase 1 intentionally does **not** include:

- `PutPixel` as the primary rendering path.
- the public drawing primitives for lines, shapes, or text
- Multiple windows.
- GPU-only rendering infrastructure.
- Font loading or font discovery.
- `Std.Tui` integration.
- `go` / task access to the graphics handle.

## MVP rendering model

The first slice should optimize for full-frame software rendering, not single-pixel VM calls.

- [ ] One frame is one contiguous pixel array.
- [ ] Pixels are row-major, top-left to bottom-right.
- [ ] The frame dimensions must match the current window size.
- [ ] After a resize event, the program regenerates and re-presents the frame at the new size.

## Required user-visible capabilities

- [ ] Open a window with initial width, height, and title.
- [ ] Query the current window size.
- [ ] Poll pending events without blocking.
- [ ] Upload and present a full frame.
- [ ] Close the session.

## Foundation use cases

This slice should already be enough for:

- [ ] a solid-color smoke test
- [ ] a simple gradient demo
- [ ] a full-frame Mandelbrot renderer without in-runtime drawing primitives

## Success criteria

A small FPAS program can:

- [ ] Open a window.
- [ ] Query its size.
- [ ] Allocate an `array of integer` with `width * height` entries.
- [ ] Fill that array with a solid color or gradient.
- [ ] Upload the frame.
- [x] React to `Resize`, `Key(Escape)`, and `CloseRequested`.
- [ ] Exit without hanging or leaking host state.

## Acceptance checks

- [ ] `cargo build`
- [ ] `cargo test --workspace`
- [ ] One smoke example run manually on Windows, Linux, and macOS.
- [ ] Manual resize / close test.
- [x] Repeated open / close in one process without stale global state is covered by focused runtime and compiler/VM tests.
- [x] Repeated `Application.Close(App)` on the same handle is covered by a focused runtime test.

## Why bulk frame upload comes first

Fractal workloads and similar software-rendered programs are dominated by bulk frame production.
Using one VM call per pixel would validate the wrong path first and encourage an inefficient API baseline.
The first milestone should therefore prove the native window lifecycle and bulk framebuffer upload path.

## Immediate follow-up after Phase 1

- [x] a runtime-owned backbuffer
- [x] `Clear`
- [x] `PutPixel`
- [x] `DrawLine`
- [x] `DrawRect` / `FillRect`
- [x] `DrawCircle`
- [x] `DrawText`
- [x] `Present`
- [x] Mouse events.
- [x] Wheel events.
- [ ] Blocking or timeout-based event wait.
- [ ] Dirty-rectangle or scanline upload helpers.
- [x] Julia explorer example with interactive controls.
- [ ] Mandelbrot explorer example with interactive controls.
# `Std.Graph` implementation plan

**Status:** proposed rollout plan.

## Strategy

Implement `Std.Graph` as a sequence of small, verifiable slices.
Each slice should end in a working state and touch one cohesive concern.

## Recommended PR slices

### 1. Documentation and opcode reservation

Scope:

- Add this planning set.
- Add `graph.rs` under `crates/fpas-bytecode/src/intrinsic/`.
- Register `Intrinsic::Graph` in `crates/fpas-bytecode/src/intrinsic/mod.rs`.

Verify:

- `cargo build`
- bytecode intrinsic tests still pass

### 2. Semantic surface registration

Scope:

- Register `Std.Graph` in `fpas-sema`.
- Add the Phase 1 types and routine signatures.
- Ensure `uses Std.Graph` resolves cleanly.

Verify:

- `cargo test --workspace`
- new sema tests for valid and invalid `Std.Graph` calls

### 3. Compiler lowering

Scope:

- Add `Std.Graph` lowering in `fpas-compiler`.
- Ensure each `Application.*` call maps to the intended intrinsic.

Verify:

- compiler lowering tests for all Phase 1 routines

### 4. Runtime skeleton in `fpas-std`

Scope:

- Add `graph/` module files.
- Introduce `GraphSession`, event normalization, and framebuffer validation helpers.
- Stub backend integration where necessary, but keep the project building.
- Keep the implementation under `crates/fpas-std/src/graph/`; `src/lib.rs` only re-exports.

Verify:

- `cargo build`
- focused unit tests for size validation and pixel-length checks

### 5. VM bridge

Scope:

- Add graph intrinsic dispatch under `fpas-vm/src/vm/execute/io/graph/`.
- Convert between VM values and `fpas-std` graph runtime types.
- Construct `Std.Graph.Event` records in the VM.

Verify:

- VM tests for `Open`, `Close`, `Size`, `PollEvent`, and `UploadFrame`

### 6. First working window lifecycle

Scope:

- Wire `winit` + `softbuffer` in `fpas-std`.
- Implement a real native window, event polling, resize updates, and frame presentation.

Verify:

- `cargo build`
- `cargo test --workspace`
- manual smoke run on Windows, Linux, macOS

### 7. Runtime-owned drawing surface

Scope:

- Add a persistent backbuffer to `GraphSession`.
- Add `Clear`, `PutPixel`, `DrawLine`, `DrawRect`, `FillRect`, `DrawCircle`, and `Present`.
- Keep drawing logic split by concern in small files.

Verify:

- runtime tests for primitive clipping and raster correctness
- VM tests for each new drawing intrinsic

### 8. Text and richer input

Scope:

- Add bitmap text drawing.
- Add mouse, drag, and wheel events.
- Add any missing event types needed for explorer-style interaction.

Verify:

- runtime tests for text clipping and deterministic glyph output
- VM tests for mouse and wheel event records

### 9. Examples and canonical std docs

Scope:

- Add Mandelbrot and Julia example programs or port the existing showcases.
- Add `docs/pascal/std/graph.md` once the surface is real.
- Update `docs/pascal/std/README.md`.

Verify:

- examples run manually
- docs and implementation names match exactly

## File touchpoints checklist

The first implementation pass should expect to touch at least these paths:

- `crates/fpas-bytecode/src/intrinsic/mod.rs`
- `crates/fpas-bytecode/src/intrinsic/graph.rs`
- `crates/fpas-sema/src/std_registry/loaded/mod.rs`
- `crates/fpas-sema/src/std_registry/loaded/graph/`
- `crates/fpas-compiler/src/compiler/std_calls/mod.rs`
- `crates/fpas-compiler/src/compiler/std_calls/graph.rs`
- `crates/fpas-vm/src/vm/execute/io/mod.rs`
- `crates/fpas-vm/src/vm/execute/io/graph/`
- `crates/fpas-std/src/lib.rs`
- `crates/fpas-std/src/graph/`

For `fpas-std`, new `Std.Graph` runtime logic should be added inside `crates/fpas-std/src/graph/`, not as top-level `graph_*` files.

## First implementation slice to start with

If the goal is to begin coding immediately, start with this exact sequence:

1. Reserve the graph intrinsic family in `fpas-bytecode`.
2. Register the Phase 1 `Std.Graph` symbols in `fpas-sema`.
3. Lower those calls in `fpas-compiler`.
4. Add a stubbed `fpas-vm` / `fpas-std` runtime path that returns clear "not implemented yet" diagnostics.
5. Replace the stubs with the real `winit` + `softbuffer` session.

This keeps the work incremental and lets the compiler surface stabilize before native window integration begins.

## Exit condition for Phase 1

Phase 1 is complete when all of the following are true:

- A real FPAS program can open a native window.
- The program can present a full RGB frame.
- Resize and quit are observable through `Application.PollEvent`.
- Escape-to-exit works through the proposed key event path.
- The docs in `docs/pascal/std/graph.md` match the implementation.

## Exit condition for the first useful graphics release

The first useful graphics release is complete when all of the following are true:

- a program can draw pixels, lines, simple shapes, and text
- keyboard, mouse, wheel, resize, and close events are available
- a Mandelbrot example runs interactively
- a Julia example runs interactively
- the canonical `Std.Graph` docs match the implemented surface
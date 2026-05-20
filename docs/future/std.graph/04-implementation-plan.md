# `Std.Graph` implementation plan

**Status:** proposed rollout plan.

## Strategy

Implement `Std.Graph` as a sequence of small, verifiable slices.
Each slice should end in a working state and touch one cohesive concern.

## Recommended PR slices

### 1. Documentation and opcode reservation

Scope:

- [x] Add this planning set.
- [x] Add `graph.rs` under `crates/fpas-bytecode/src/intrinsic/`.
- [x] Register `Intrinsic::Graph` in `crates/fpas-bytecode/src/intrinsic/mod.rs`.

Verify:

- [x] `cargo build`
- [x] bytecode intrinsic tests still pass

### 2. Semantic surface registration

Scope:

- [x] Register `Std.Graph` in `fpas-sema`.
- [x] Add the Phase 1 types and routine signatures.
- [x] Ensure `uses Std.Graph` resolves cleanly.

Verify:

- [x] `cargo test --workspace`
- [x] new sema tests for valid and invalid `Std.Graph` calls

### 3. Compiler lowering

Scope:

- [x] Add `Std.Graph` lowering in `fpas-compiler`.
- [x] Ensure each `Application.*` call maps to the intended intrinsic.

Verify:

- [x] compiler lowering tests for all Phase 1 routines

### 4. Runtime skeleton in `fpas-std`

Scope:

- [x] Add `graph/` module files.
- [x] Introduce `GraphSession`, event normalization, and framebuffer validation helpers.
- [x] Stub backend integration where necessary, but keep the project building.
- [x] Keep the implementation under `crates/fpas-std/src/graph/`; `src/lib.rs` only re-exports.

Verify:

- [x] `cargo build`
- [x] focused unit tests for size validation and pixel-length checks

### 5. VM bridge

Scope:

- [x] Add graph intrinsic dispatch under `fpas-vm/src/vm/execute/io/graph/`.
- [x] Convert between VM values and `fpas-std` graph runtime types.
- [x] Construct `Std.Graph.Event` records in the VM.

Verify:

- [x] VM tests for `Open`, `Close`, `Size`, `PollEvent`, and `UploadFrame`

### 6. First working window lifecycle

Scope:

- [ ] Wire `winit` + `softbuffer` in `fpas-std`.
- [ ] Implement a real native window, event polling, resize updates, and frame presentation.

Verify:

- [ ] `cargo build`
- [ ] `cargo test --workspace`
- [ ] manual smoke run on Windows, Linux, macOS

### 7. Runtime-owned drawing surface

Scope:

- [ ] Add a persistent backbuffer to `GraphSession`.
- [ ] Add `Clear`, `PutPixel`, `DrawLine`, `DrawRect`, `FillRect`, `DrawCircle`, and `Present`.
- [ ] Keep drawing logic split by concern in small files.

Verify:

- [ ] runtime tests for primitive clipping and raster correctness
- [ ] VM tests for each new drawing intrinsic

### 8. Text and richer input

Scope:

- [ ] Add bitmap text drawing.
- [ ] Add mouse, drag, and wheel events.
- [ ] Add any missing event types needed for explorer-style interaction.

Verify:

- [ ] runtime tests for text clipping and deterministic glyph output
- [ ] VM tests for mouse and wheel event records

### 9. Examples and canonical std docs

Scope:

- [ ] Add Mandelbrot and Julia example programs or port the existing showcases.
- [ ] Add `docs/pascal/std/graph.md` once the surface is real.
- [ ] Update `docs/pascal/std/README.md`.

Verify:

- [ ] examples run manually
- [ ] docs and implementation names match exactly

## File touchpoints checklist

The first implementation pass should expect to touch at least these paths:

- [x] `crates/fpas-bytecode/src/intrinsic/mod.rs`
- [x] `crates/fpas-bytecode/src/intrinsic/graph.rs`
- [x] `crates/fpas-sema/src/std_registry/loaded/mod.rs`
- [x] `crates/fpas-sema/src/std_registry/loaded/graph/`
- [x] `crates/fpas-compiler/src/compiler/std_calls/mod.rs`
- [x] `crates/fpas-compiler/src/compiler/std_calls/graph.rs`
- [x] `crates/fpas-vm/src/vm/execute/io/mod.rs`
- [x] `crates/fpas-vm/src/vm/execute/io/graph/`
- [x] `crates/fpas-std/src/lib.rs`
- [x] `crates/fpas-std/src/graph/`

For `fpas-std`, new `Std.Graph` runtime logic should be added inside `crates/fpas-std/src/graph/`, not as top-level `graph_*` files.

## First implementation slice to start with

If the goal is to begin coding immediately, start with this exact sequence:

- [x] Reserve the graph intrinsic family in `fpas-bytecode`.
- [x] Register the Phase 1 `Std.Graph` symbols in `fpas-sema`.
- [x] Lower those calls in `fpas-compiler`.
- [x] Add a stubbed `fpas-vm` / `fpas-std` runtime path that returns clear "not implemented yet" diagnostics.
- [ ] Replace the stubs with the real `winit` + `softbuffer` session.

This keeps the work incremental and lets the compiler surface stabilize before native window integration begins.

## Exit condition for Phase 1

Phase 1 is complete when all of the following are true:

- [ ] A real FPAS program can open a native window.
- [ ] The program can present a full RGB frame.
- [ ] Resize and quit are observable through `Application.PollEvent`.
- [ ] Escape-to-exit works through the proposed key event path.
- [ ] The docs in `docs/pascal/std/graph.md` match the implementation.

## Exit condition for the first useful graphics release

The first useful graphics release is complete when all of the following are true:

- [ ] a program can draw pixels, lines, simple shapes, and text
- [ ] keyboard, mouse, wheel, resize, and close events are available
- [ ] a Mandelbrot example runs interactively
- [ ] a Julia example runs interactively
- [ ] the canonical `Std.Graph` docs match the implemented surface
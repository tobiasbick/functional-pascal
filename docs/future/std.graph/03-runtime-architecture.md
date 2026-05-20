# `Std.Graph` runtime architecture

**Status:** proposed implementation structure.

## Principles

- [x] `Std.Graph` is a new runtime surface, not an extension of `Std.Console` or `Std.Tui`.
- [x] The implementation should follow the same integration chain already used by other `Std.*` units: sema, compiler lowering, bytecode intrinsics, VM execution, and `fpas-std` runtime support.
- [x] Files should stay small and thematic.
- [x] In `fpas-std`, runtime files should be grouped by the FPAS unit they implement: `Std.Tui` under `src/tui/`, `Std.Graph` under `src/graph/`, and so on.
- [x] Phase 1 should support one active native graphics session per process.

## Proposed crate responsibilities

### `crates/fpas-std`

Owns the host-facing graphics session and the `winit` + `softbuffer` backend integration.
For this crate, the preferred layout follows the Pascal unit boundary directly: unit-owned code for `Std.Graph` should live under `src/graph/`, not as scattered top-level `graph_*` files.

Current stub layout:

```text
crates/fpas-std/src/graph/
  mod.rs           - graph module root and re-exports
  event.rs         - current normalized event model and EventKind names
  framebuffer.rs   - current frame-size and pixel-payload validation helpers
  session.rs       - current GraphSession lifecycle and staged upload state
  stub.rs          - current not-yet-implemented runtime diagnostics
  tests.rs         - current runtime-skeleton tests
```

Planned fuller layout:

```text
crates/fpas-std/src/graph/
  mod.rs           - graph runtime entry points
  color.rs         - packed RGB helpers and validation
  event.rs         - normalized `Std.Graph.Event` payload model
  framebuffer.rs   - frame-size checks and pixel staging
  upload.rs        - bulk `UploadFrame` validation and copy path
  line.rs          - line rasterization into the backbuffer
  rect.rs          - rectangle outline and fill primitives
  circle.rs        - circle rasterization
  text.rs          - bitmap text rendering
  session.rs       - `GraphSession` lifecycle and state
  backend.rs       - `winit` + `softbuffer` bridge
  tests.rs         - runtime-focused graph tests
```

`crates/fpas-std/src/lib.rs` should only re-export the public graph runtime entry points, just as it already does for existing `Std.*` units. The unit-owned implementation should stay inside `crates/fpas-std/src/graph/`.

### `crates/fpas-bytecode`

Defines a dedicated intrinsic family.

```text
crates/fpas-bytecode/src/intrinsic/
  graph.rs         - `GraphIntrinsic`
  mod.rs           - `Intrinsic::Graph` registration
```

Suggested first discriminants:

- [x] `ApplicationOpen`
- [x] `ApplicationClose`
- [x] `ApplicationSize`
- [x] `ApplicationPollEvent`
- [x] `ApplicationUploadFrame`

### `crates/fpas-sema`

Registers the Pascal-facing types and routines for `uses Std.Graph`.

```text
crates/fpas-sema/src/std_registry/loaded/graph/
  mod.rs             - Phase 1 type registration entry point
  application_api.rs - Phase 1 lifecycle, event, and upload routine registration
```

`crates/fpas-sema/src/std_registry/loaded/mod.rs` should register `Std.Graph` exactly once, following the same pattern as the other standard units.

### `crates/fpas-compiler`

Lowers `Std.Graph.Application.*` calls to the new intrinsic family.

```text
crates/fpas-compiler/src/compiler/std_calls/
  graph.rs         - Phase 1 application lifecycle, event, and upload lowering
  mod.rs           - graph module registration
```

### `crates/fpas-vm`

Owns stack manipulation, intrinsic dispatch, record construction, and the call into `fpas-std`.

```text
crates/fpas-vm/src/vm/execute/io/graph/
  mod.rs           - current graph intrinsic dispatch entry point
  application.rs   - current Phase 1 VM bridge for lifecycle, event, and upload calls
  records.rs       - current `Std.Graph.Application`, `Size`, and `Event` record construction
```

`crates/fpas-vm/src/vm/execute/io/mod.rs` should route the new intrinsic family into that graph module.
The VM now also keeps dedicated graph session state beside the TUI state so `Std.Graph` can evolve without reusing the terminal-hosted path.

## Data flow

1. `fpas-sema` registers `Std.Graph` types and routines.
2. `fpas-compiler` lowers those calls to `Intrinsic::Graph` opcodes.
3. `fpas-vm` decodes the graph intrinsic and pops VM arguments.
4. `fpas-vm` calls `fpas-std` graph session helpers.
5. `fpas-std` updates the native window, polls events, or presents the framebuffer.

## Session and threading model

- [ ] Phase 1 should assume one active graphics session.
- [ ] All graph intrinsics should execute on the same host thread that owns the native window.
- [ ] If the current CLI / VM startup model does not guarantee main-thread ownership on macOS, that requirement must be resolved before the feature is enabled there.
- [ ] `go` tasks should not touch `Std.Graph` in Phase 1.

## Event normalization

The backend should convert platform-specific events into a minimal, stable model:

- [ ] `CloseRequested`
- [ ] `Resize`
- [ ] `Key`

That normalization belongs in `fpas-std/src/graph/event.rs`, not in the VM layer.

## Framebuffer contract

- [ ] Phase 1 uses one full-frame upload call.
- [ ] Pixel format is `$00RRGGBB` packed into `integer` / `u32` values.
- [ ] The runtime should validate `Width`, `Height`, and `Length(Pixels)` before presenting.
- [ ] Resize handling should update the expected frame size before the next `UploadFrame` call.

## Planned drawing model after the foundation slice

- [ ] `GraphSession` should own a persistent backbuffer.
- [ ] Drawing intrinsics mutate that backbuffer in place.
- [ ] `Present` flushes the current backbuffer to the native window.
- [ ] `UploadFrame` remains the direct bulk upload path for render-heavy code.
- [ ] Keep raster concerns in separate theme files instead of one large drawing module.

## Deliberate separation from `Std.Tui`

`Std.Tui` is terminal-hosted and should remain terminal-hosted.
`Std.Graph` should not share the TUI session, event loop, or redraw model in Phase 1.
Any future common abstractions should only be extracted after both paths exist and show real duplication.

## Layout preference inside `fpas-std`

For this feature, the intended rule is:

- [ ] if code exists only because of `uses Std.Graph`, it belongs under `crates/fpas-std/src/graph/`
- [ ] `crates/fpas-std/src/lib.rs` may re-export graph items, but should not absorb graph implementation logic
- [ ] avoid new top-level files such as `graph_event.rs`, `graph_line.rs`, or `graph_backend.rs`

This keeps the Rust runtime layout aligned with the FPAS standard-unit surface.
# Future: Graphics and 3D Rendering

> Graphics application case only. No graphics implementation plan or API is
> agreed. Shared foundations and their decision status are maintained
> in [FPAS 2 language directions](fpas2-language-directions.md), including the
> recorded numeric design and the remaining open proposals. Native library
> buffers with existing FPAS scalar types are the preferred graphics-data
> alternative; no buffer API or implementation is agreed.

The [shared terminal and graphical UI plan](shared-ui.md) is intended to proceed
during graphics work after the intended FPAS 2 work. It depends on the graphics
foundations described here but does not select their backend or graphics API.

Public homes follow the binding [namespace specification](../specs/std-namespaces.md):
`Std.Graphics.Image`, `Std.Graphics.Draw2D`, `Std.Graphics.Render3D` and
`Std.Graphics.Gpu`, with independent `Std.Window`, `Std.Input` and `Std.Audio`
domains. Their interfaces and implementations remain open. General byte storage
belongs to `Std.Bytes`; graphics-specific buffers belong to `Std.Graphics`.
References to existing APIs below keep current names until AP53 migrates them.

Functional Pascal has no way to open a window, draw pixels, or use a GPU. This note records the
runtime and language gaps that a graphics capability would expose, and the properties that would
make such a capability practical. It assumes a split in which native Rust code owns the
performance-critical layer (windowing, GPU resources, draw submission) and FPAS code owns
higher-level concerns such as scenes, cameras, animation, and the application loop. This is the
same split that `Std.Ui.Terminal` already uses over `Std.Console`.

The graphics unit itself is out of scope here. This note records rendering needs
for the shared runtime, standard-library, build and language directions, together
with graphics-specific platform and deployment questions. Shared design details
belong in the linked FPAS 2 topics.

## Existing support

These parts of the current runtime already fit a graphics capability:

- **Opaque resource handles.** `Std.Net.Sockets.Connection`, `Std.Net.Sockets.Listener`, and the
  `Std.Tasks` cancellation types show the pattern of opaque values that only the owning intrinsics
  accept. Explicit release is also established, as are cleanup when the VM ends and interruption of
  blocked operations. Windows, meshes, textures, and pipelines can follow the same model.
- **Main-thread execution.** The main task runs on the thread that started execution
  ([scheduling](../pascal/language/concurrency/scheduling.md)). Windowing systems that require the
  main thread, such as macOS, are therefore reachable from FPAS code.
- **Concurrency.** Bounded channels, `TryReceive`, `Select`, and task groups let background work
  feed a frame loop without blocking it.
- **Layered hosts.** `Std.Ui.Terminal` is written in FPAS over a narrow intrinsic layer and provides an
  update/view host with background messages. A rendering host can reuse this structure.
- **Scalar math.** `Std.Math` provides trigonometry, square root, clamping, and related functions.

## Gaps

### Bulk numeric data

Arrays store one tagged runtime value per element. A vertex buffer of `N` floats therefore costs
several times the memory of a packed `f32` buffer, and every upload converts each element at the
native boundary. Bytes are `array of integer` with a range check per element;
`Std.Io.Fs` already supports binary reads and writes using that representation.
Large or frequently changing meshes, image data, and audio samples pay the
tagged-storage and conversion cost. The [standard library roadmap](std-roadmap.md) lists the canonical byte
representation as an open decision.

### Numeric types

`real` is always 64-bit and `integer` is always 64-bit signed. GPU data is mostly 32-bit float,
16-bit or 32-bit unsigned integer, and packed 8-bit color channels. Ordinary FPAS
arrays do not provide these packed layouts. New scalar numeric types are one
possible route; native buffers with explicit storage formats are another and
are currently preferred. They can describe vertex layouts without changing
FPAS scalar arithmetic. Range checks and conversions happen when filling a
buffer rather than being repeated for every compatible native handoff.
The separate numeric design is maintained in
[FPAS 2 numeric types, money and decimal](fpas2-language-directions.md#21-explicit-width-numeric-types-money-and-decimal).

### Small value aggregates

Records are reference-counted, copy-on-write heap values. Vector and matrix math written as records
(`Vec3`, `Mat4`, `Quat`) allocates on every intermediate result. Per-frame camera, transform, and
animation code would put this allocation pressure on the hot path.

### Vector arithmetic notation

There is no operator overloading, so vector math is written as nested function calls
(`Add(Scale(V, 2.0), W)`). This is correct but hard to read in transform-heavy code.

### Event loop integration

The `Std.Ui.Terminal` host reads console input from a blocking call inside a worker task. Native windowing
systems instead deliver events through a loop that must be pumped on the main thread. FPAS has no
general way to wait on "window events or channel messages or a frame deadline" in a single
`Select`. Without one, a frame loop must poll with `TryReceive`.

### Time resolution

`Std.Time` offers monotonic time in milliseconds only. Frame pacing, interpolation, and profiling
benefit from microsecond or nanosecond resolution.

### Optional native dependencies

No crate defines Cargo features, and the VM links every standard capability unconditionally. A GPU
and windowing stack would increase the size and build time of every `fpas` binary, including
headless servers and the test runner.

## Desirable capabilities

### Runtime and standard library

- A shared native binary-buffer foundation under `Std.Bytes`, integrated with
  binary filesystem access, whose target home is `Std.Io.Fs`.
- Specialized pixel, vertex, index and audio buffers with explicit storage
  formats, bulk operations and compatible native handoffs without per-element
  conversion. Existing `integer` and `real` remain the FPAS scalar interface.
- Main-thread event pumping as a selectable source, so `Select` can combine window events, channels,
  task completion, and timeouts.
- A monotonic clock with sub-millisecond resolution.
- Cargo features that make windowing and GPU support optional, while the standard-unit registry
  reports a clear diagnostic when a program uses a capability that the build excludes.
- Image decoding in native code from a path or byte buffer, so pixel data does not pass through
  FPAS arrays.

### Language

- [Numeric types, money and decimal](fpas2-language-directions.md#21-explicit-width-numeric-types-money-and-decimal):
  recorded agreed numeric design with remaining open points; no implementation
  is scheduled. Small scalar numeric types are not a prerequisite for the
  preferred buffer route. `money` and `decimal` are outside this comparison.
- [Packed numeric data](fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers):
  native library buffers are the preferred alternative to language-level packed
  arrays. The API, storage and ownership contracts still need decisions.
- [Small records without heap allocation](fpas2-language-directions.md#23-small-records-without-heap-allocation):
  open proposal for vectors, matrices, colors and rectangles. First evaluate
  optimizing existing records; no separate value-record form is agreed.
- [Record arithmetic operators](fpas2-language-directions.md#24-record-arithmetic-operators):
  open proposal for arithmetic and comparisons. Pure, statically resolved
  operators are a candidate; notation alone does not justify another language form.

The shared host requirements are tracked under
[clock types and resolution](fpas2-language-directions.md#10-distinct-duration-and-clock-types)
and [native event loops](fpas2-language-directions.md#25-native-event-loops-and-selectable-host-events).
Open proposals require explicit design agreement, and every selected topic needs
an implementation decision before work begins.

## Agreed design: numeric types

The [full numeric design](fpas2-language-directions.md#21-explicit-width-numeric-types-money-and-decimal)
is maintained in the FPAS 2 directions, including names, literals, conversions,
arithmetic, bit operations, runtime representation and open rounding and storage
questions. That design remains available for independent numeric language work.
Graphics storage can instead use the preferred native-buffer alternative without
new small scalar types or a rename of `integer` and `real`. The buffer comparison
does not change the status of `money` or `decimal` or imply parser migration.

## Preferred alternative: native buffers

One native binary-buffer core would support specialized pixel, vertex, index
and audio interfaces. A format such as `RGBA8`, `Float32` or `Index16` describes
storage rather than an FPAS scalar type. FPAS supplies ordinary `integer` and
`real` values; native code checks and packs writes and can decode assets directly
into compatible buffers. Bulk operations avoid routing every byte through FPAS.

Rust owns GPU resources, uploads and draw submission. Compatible host buffers
avoid repeated element conversion, but GPU transfers may still copy or require
staging. Layout descriptions, lifetimes and task sharing need explicit contracts;
ordinary FPAS records do not automatically acquire a GPU or C layout.

The shared design, tradeoffs and comparison with packed arrays belong in
[FPAS 2 packed data](fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers).
This preference also applies to the [alternative multimedia foundations](graphics-alternative.md).
Small-record allocation, vector arithmetic notation and native event integration
remain separate topics. The preference does not authorize implementation.

## Non-goals

- A full game engine, scene editor, or entity-component system in the standard library.
- Designing a generic foreign-function interface to arbitrary native libraries;
  see the separate [C FFI proposal](fpas2-language-directions.md#11-c-foreign-function-interface).
- Shader authoring in FPAS.
- Bindings to C or C++ graphics libraries. Native graphics support should use pure Rust crates.

## Open decisions

- Which platforms must graphics support cover (desktop only, or also web through WebAssembly)?
- How should optional capabilities appear in projects: a project-level flag, a separate
  distribution, or detection from `uses`?

# Future: Graphics and 3D Rendering

> Idea only. No implementation plan, unit name, or API decision.

Functional Pascal has no way to open a window, draw pixels, or use a GPU. This note records the
runtime and language gaps that a graphics capability would expose, and the properties that would
make such a capability practical. It assumes a split in which native Rust code owns the
performance-critical layer (windowing, GPU resources, draw submission) and FPAS code owns
higher-level concerns such as scenes, cameras, animation, and the application loop. This is the
same split that `Std.Tui` already uses over `Std.Console`.

The graphics unit itself is out of scope here. This note covers the surrounding runtime, standard
library, build, and language support.

## Existing support

These parts of the current runtime already fit a graphics capability:

- **Opaque resource handles.** `Std.Net.Connection`, `Std.Net.Listener`, and the
  `Std.Tasks` cancellation types show the pattern of opaque values that only the owning intrinsics
  accept. Explicit release is also established, as are cleanup when the VM ends and interruption of
  blocked operations. Windows, meshes, textures, and pipelines can follow the same model.
- **Main-thread execution.** The main task runs on the thread that started execution
  ([scheduling](../pascal/language/concurrency/scheduling.md)). Windowing systems that require the
  main thread, such as macOS, are therefore reachable from FPAS code.
- **Concurrency.** Bounded channels, `TryReceive`, `Select`, and task groups let background work
  feed a frame loop without blocking it.
- **Layered hosts.** `Std.Tui` is written in FPAS over a narrow intrinsic layer and provides an
  update/view host with background messages. A rendering host can reuse this structure.
- **Scalar math.** `Std.Math` provides trigonometry, square root, clamping, and related functions.

## Gaps

### Bulk numeric data

Arrays store one tagged runtime value per element. A vertex buffer of `N` floats therefore costs
several times the memory of a packed `f32` buffer, and every upload converts each element at the
native boundary. Bytes are `array of integer` with a range check per element, and `Std.Fs` reads
and writes UTF-8 text only. Large or frequently changing meshes, image data, and audio samples pay
this cost. The [standard library roadmap](std-roadmap.md) already lists the canonical byte
representation as an open decision.

### Numeric types

`real` is always 64-bit and `integer` is always 64-bit signed. GPU data is mostly 32-bit float,
16-bit or 32-bit unsigned integer, and packed 8-bit color channels. Without fixed-width types,
every boundary call converts and range-checks values, and FPAS code cannot describe a vertex layout
precisely.

### Small value aggregates

Records are reference-counted, copy-on-write heap values. Vector and matrix math written as records
(`Vec3`, `Mat4`, `Quat`) allocates on every intermediate result. Per-frame camera, transform, and
animation code would put this allocation pressure on the hot path.

### Vector arithmetic notation

There is no operator overloading, so vector math is written as nested function calls
(`Add(Scale(V, 2.0), W)`). This is correct but hard to read in transform-heavy code.

### Event loop integration

The `Std.Tui` host reads console input from a blocking call inside a worker task. Native windowing
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

- A canonical packed buffer type for bytes, together with binary file reads and writes in `Std.Fs`.
- Typed numeric buffers (`f32`, `u16`, `u32`, `u8`) that native intrinsics can borrow without
  per-element conversion.
- Main-thread event pumping as a selectable source, so `Select` can combine window events, channels,
  task completion, and timeouts.
- A monotonic clock with sub-millisecond resolution.
- Cargo features that make windowing and GPU support optional, while the standard-unit registry
  reports a clear diagnostic when a program uses a capability that the build excludes.
- Image decoding in native code from a path or byte buffer, so pixel data does not pass through
  FPAS arrays.

### Language

- **Fixed-width numeric types**, for example `real32`, `uint8`, `uint16`, and `uint32`, with
  explicit conversions and defined overflow behavior.
- **Packed arrays** of fixed-width numeric types or of records that contain only such fields, stored
  contiguously and passed to intrinsics without conversion.
- **Value records**: small, fixed-size records without heap allocation and with copy semantics, for
  vectors, matrices, colors, and rectangles.
- **Operator declarations** for records, limited to arithmetic and comparison operators, so vector
  and matrix code can use infix notation. They must remain pure and statically resolved.

Each language item needs its own design note and explicit agreement before implementation.

## Non-goals

- A full game engine, scene editor, or entity-component system in the standard library.
- A generic foreign-function interface to arbitrary native libraries.
- Shader authoring in FPAS.
- Bindings to C or C++ graphics libraries. Native graphics support should use pure Rust crates.

## Open decisions

- Should packed numeric data be a language-level array form or a standard-library buffer type?
- Should value records be a separate declaration form or an inferred property of small records?
- Which platforms must graphics support cover (desktop only, or also web through WebAssembly)?
- How should optional capabilities appear in projects: a project-level flag, a separate
  distribution, or detection from `uses`?

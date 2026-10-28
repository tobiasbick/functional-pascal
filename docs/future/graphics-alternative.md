# Future: Alternative Graphics and Multimedia Foundations

> **⚠️ Disclaimer:** The human overlord is considering more ways to put pixels on
> a screen. FPAS is making really nice progress, even if, as the human overlord
> puts it, it is "so fucking expensive". Apparently the reward for that progress
> is thinking of more things to build.
> These native graphics and multimedia alternatives are considerations only:
> no backend implementation plan, no schedule, no promise.
> The human currently prefers the pure Rust route. That is a preference, not a
> backend decision or authorization to implement anything.

## Purpose and status

The existing [graphics note](graphics.md) describes graphics as an application
case for shared FPAS language and runtime foundations. This alternative explores
how a native graphics and audio layer could be assembled for both 3D games and
applications with custom 2D interfaces.

Neither document specifies an agreed graphics API. This note does not replace
the existing note. Shared designs and their status belong in
[FPAS 2 language directions](fpas2-language-directions.md). For graphics and audio
data, native library buffers without new scalar numeric types are the human's
current preference. This is a preferred alternative, not an agreed buffer API
or authorization to implement it.

Two routes are under consideration:

1. SDL3 as a common native foundation for windows, input, graphics, and audio.
2. Rust libraries assembled into a common FPAS host: `winit`, `wgpu`, `cpal`, and
   selected audio decoders. **This is the human's current preference.**

Using SDL3 would require reconsidering the existing graphics note's preference
for native graphics support built from pure Rust crates. No such change is
agreed here.

## Common shape

Both routes could expose the same kinds of FPAS capabilities:

- Windows, keyboard and mouse input, display scaling, and application events.
- 2D images, rectangles, clipping, colors, text, and rendering to textures.
- 3D buffers, textures, shaders, pipelines, depth testing, and draw submission.
- Audio devices, buffered playback, decoding, and native signal processing.
- Explicit resource ownership, release, and cleanup when execution ends.

FPAS would own application state, interaction, scene descriptions, cameras,
animation decisions, and playback commands. Native code would own GPU resources,
draw submission, decoding, and continuous audio delivery.

The FPAS interfaces should describe these operations without requiring
applications to use backend-specific handles or terminology. This does not
require implementing a general backend-switching framework in advance.

2D and 3D would share host and resource foundations. A 2D application would not
need scene or camera concepts. A 3D renderer would use the GPU interface directly,
rather than being built on top of a simple 2D drawing API.

Audio must continue independently of frame rendering and FPAS update work.
Native audio callbacks must not synchronously depend on arbitrary FPAS execution.
Equalization, mixing, and analysis would need an explicit native processing path.

## Preferred data alternative: native buffers

Both backends can use one shared native binary-buffer foundation with specialized
pixel, vertex, index and audio interfaces. FPAS would continue calculating with
`integer` and `real`. Formats such as `RGBA8`, `Float32`, `Index16` and signed
16-bit PCM describe storage, without introducing matching scalar types into the
language or requiring a rename of the existing ones.

Native code checks ranges and narrows or rounds values when buffers are filled.
Decoders, generators and bulk operations can produce compatible data directly.
Rendering and audio consume supported layouts without repeatedly unpacking
tagged FPAS arrays. GPU uploads may still copy or need staging; this does not
promise zero-copy delivery to a device.

This matches the proposed responsibility split: FPAS controls application state,
scenes and playback; Rust owns native storage, GPU resources and continuous audio
delivery. The buffer preference is independent of choosing SDL3 or the Rust
library route. Pixel, vertex and audio layouts need distinct APIs over a common
binary foundation, not backend-specific handles in application code.

The [shared packed-data proposal](fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers)
owns the comparison with language-level packed arrays and explicit-width scalar
types. API names, supported formats, rounding, layout validation, borrowing,
task sharing and cleanup remain open. Small-record allocation and native event
integration remain separate work; `money` and `decimal` are outside this comparison.

## Route A: SDL3

SDL3 could provide windows, events, input, audio streams, and graphics access.
Its Renderer API offers a simpler 2D path; its GPU API provides modern 3D and
compute operations. Cameras, scenes, materials, widgets, and application behavior
would still need their own layers.

SDL audio accepts decoded PCM. Compressed music formats would still need
additional decoders. Text rendering, image loading, and shader preparation also
need explicit choices; SDL core is not a complete application framework.

The Rust bindings can build SDL from source and link it statically into the
runner. Building that runner would require CMake, a C compiler, and the relevant
platform development dependencies. Ordinary FPAS application builds could use
a prebuilt runner without invoking those tools.

Potential advantages:

- One project supplies much of the native platform infrastructure.
- A convenient 2D entry point alongside lower-level 3D access.
- Less integration between independently maintained window and audio libraries.

Costs and uncertainties:

- A C implementation and Rust bindings add another build and debugging boundary.
- Bindings and SDL versions must remain compatible.
- GPU shaders require backend-appropriate formats and a preparation workflow.
- Successful window or audio initialization does not prove GPU support on the
  same machine. Each capability needs separate target validation.

## Route B: Rust libraries

The candidate composition is:

| Responsibility | Candidate |
| --- | --- |
| Windows, input, and event loop | `winit` |
| GPU rendering for 2D and 3D | `wgpu` |
| Audio device input/output | `cpal` |
| Audio decoding | Selected decoders behind one native interface |
| Convenient 2D drawing and text | An additional rendering/text layer, still undecided |

Here, "pure Rust" means a Rust implementation of the foundation rather than
bindings to SDL. It does not mean eliminating operating-system APIs, graphics
drivers, system libraries, or all internal `unsafe` code.

`wgpu` would support both 2D and 3D drawing. Images and rectangles can be drawn as
textured geometry, but batching, clipping, text shaping, glyph storage, and a
convenient drawing interface remain work to select or implement. WGSL offers a
common shader input, with backend translation handled by the graphics stack.

Individual decoder candidates include `nanomp3` for MP3, `claxon` for FLAC,
`lewton` for Ogg Vorbis, and `hound` for WAV. These are candidates, not an approved
dependency set. Their maintenance, format coverage, seeking behavior, malformed
input handling, and target performance need evaluation. A common playback layer
would have to reconcile decoder differences, buffering, positions, and metadata.

Potential advantages:

- Fits the current Rust runtime and its development tools.
- Avoids building and maintaining an additional SDL C dependency.
- Allows windowing, rendering, and audio choices to evolve separately.
- Connects directly to the surrounding Rust graphics and audio ecosystem.

Costs and uncertainties:

- FPAS must integrate several libraries into one coherent host lifecycle.
- Event-loop ownership, shutdown, resource lifetimes, and error handling cross
  library boundaries.
- More dependency versions and platform combinations need to work together.
- Convenient 2D drawing and unified audio playback require additional layers.

The preference for this route does not establish that it is smaller, faster,
easier to build, or more portable. Those claims would need measurements.

## Application model and selective view updates

Model-Update-View could remain the application model for graphical 2D interfaces:
input and background messages reach a serial host, Update produces the next
model, and View describes the interface. Controlled inputs and explicit commands
could retain their meaning without depending on terminal cells or GPU details.
This is an architectural consideration, not a decision to reuse the current
terminal host unchanged.

The [current TUI host](../pascal/std/tui/application.md#headless-frame-and-routing-order)
still calls View for each render request, then compares trees, reuses stable
layout and repaints damaged rows. Unchanged trees can skip validation, layout
and painting; terminal output also transfers only changed cells. Selective
subview construction remains an open direction. The existing proposals for
[application stores and selectors](fpas2-language-directions.md#19-application-stores-and-selectors)
and [selective TUI updates](fpas2-language-directions.md#20-selective-tui-updates)
could also inform graphical interfaces without changing their current status.

A graphical host could distinguish four kinds of work:

| Work | Possible trigger |
| --- | --- |
| Model update | An input or application message |
| Subview construction | A change to state used by that subview |
| Layout | A change to content measurement, geometry, or constraints |
| Drawing and presentation | Visual invalidation, animation, or an OS repaint request |

A position indicator changing should not require rebuilding and measuring an
unchanged large list. Conversely, an OS repaint request can require drawing
without a model change. Several messages could be processed in order before
presenting one frame, preserving their state transitions without presenting every
intermediate image.

Selectors could identify relevant state changes, but would not alone define
layout invalidation. A changed text width can affect a parent and its siblings.
Cache identity, lifetime, dependency declarations, callback effects, and
invalidation for resize, focus, pointer state, overlays, and theme changes would
need explicit rules. Automatic dependency tracking is not assumed.

An idle interface could wait for events. An active animation or custom drawing
region could request frame deadlines through the proposed
[native event integration](fpas2-language-directions.md#25-native-event-loops-and-selectable-host-events).
High-frequency audio data would stay in native buffers; a visualizer could consume
a bounded snapshot at its display rate instead of sending every sample through
the application model. A custom 2D canvas or 3D viewport could use a similar
separation between application state and continuous rendering.

## Shared UI core for TUI and 2D

The shared UI direction has since been reconsidered. The intended scope and
sequence are now recorded in the [shared UI plan](shared-ui.md): continue FPAS 2
first, then pursue graphics and the shared UI together. That plan includes a
rewrite of existing TUI applications and an initial graphical host; detailed
design and implementation remain open. The tradeoffs below still need validation.

A possible shared core could describe semantic elements such as text, buttons,
lists, input fields, rows, and columns. One adapter would render terminal cells;
another would render graphical elements. Application actions and some model and
background-work conventions could remain common.

| Concern | Potential common ground | Where the representations differ |
| --- | --- | --- |
| State and actions | Models, ordered messages, Update, and explicit commands | Host lifecycle and native events |
| Elements | Semantic text, buttons, lists, and controlled inputs | Styling and available controls |
| Layout | Content sizing, available space, alignment, and flow | Cell rounding, pixel geometry, and font measurement |
| View updates | Dependencies, cached subviews, and invalidation concepts | Measurement, painting, and presentation work |
| Custom drawing | An explicitly specialized region | Terminal cells versus pixels, textures, and GPU resources |

Such a core would need a deliberately limited portable subset and explicit
host-specific extensions. Images, transparency, proportional text, arbitrary
positioning, and 3D content do not have equivalent terminal representations.
Input methods, pointer interaction, selection, and focus also cannot be assumed
identical. Separate Views over shared application state and actions would remain
a possible alternative to sharing the element tree.

The concern is that a common interface could either constrain graphical
applications to terminal capabilities or accumulate exceptions that make both
hosts harder to use. Different measurement and invalidation rules could also
make the abstraction costly to maintain without enough application reuse.
The current `Std.Tui` interface should therefore not be treated as an agreed
universal graphics interface.

The common core needs evidence from actual interfaces in both hosts. Applications
may use separate Views over shared state when that is a better fit. The linked
plan requires a real two-host validation before migrating every application.

## Optional runners and single-executable delivery

Either route could use two prebuilt runner variants: a small runner for terminal
and server applications, and a graphics/audio runner with the native multimedia
dependencies included.

The compiler could record required capabilities from the resolved program and
its unit dependencies. Packaging could then select a compatible runner
automatically. Capability recording, runner selection, and startup validation
would be new work; they are not current behavior.

The current executable packager combines a prebuilt native runner with a compiled
FPAS program. It does not compile a new Rust runtime for every application.
Selecting a suitable prebuilt runner would preserve that model.

The delivery goal would be one executable per application and target, with no
separate multimedia DLLs. Images, fonts, shaders, and other required assets would
also need an embedding mechanism. Operating-system services and graphics/audio
drivers would remain prerequisites. Native dependencies and shader compiler
configuration must be checked for each target before claiming this goal works.

## Shared prerequisites and open questions

Both routes still need the shared foundations recorded in the existing graphics
note: compact binary/numeric storage integrated with binary file access, native
event integration, precise monotonic time, and optional native capabilities.
The preferred buffer route supplies storage formats without making new small
scalar types a prerequisite. Choosing a backend does not settle buffer APIs,
ownership or the remaining language and runtime designs.

Questions for a later decision include:

- Which Windows, Linux, Raspberry Pi, and FreeBSD configurations must work?
- Is browser delivery also a requirement?
- Which 2D text and drawing facilities should FPAS expose first?
- How should native window events participate in task selection and cancellation?
- Which audio formats and playback behaviors belong in the initial scope?
- How should embedded assets be named, loaded, and shared by native resources?

If this idea is pursued later, a small feasibility comparison could exercise
scaled text and images, input, a textured depth-tested 3D scene, continuous audio,
seeking, resource cleanup, and single-executable packaging on the selected targets.
That would provide evidence for a backend decision. It is not scheduled or
authorized by this note.

## References

- [SDL3 GPU API](https://wiki.libsdl.org/SDL3/CategoryGPU)
- [SDL3 Renderer API](https://wiki.libsdl.org/SDL3/CategoryRender)
- [SDL3 audio](https://wiki.libsdl.org/SDL3/CategoryAudio)
- [SDL3 Rust bindings](https://github.com/vhspace/sdl3-rs)
- [SDL3 native build options](https://docs.rs/sdl3-sys/latest/sdl3_sys/)
- [winit](https://github.com/rust-windowing/winit)
- [wgpu](https://github.com/gfx-rs/wgpu)
- [cpal](https://github.com/RustAudio/cpal)
- [nanomp3](https://github.com/robbie01/nanomp3)
- [claxon](https://github.com/ruuda/claxon)
- [lewton](https://github.com/RustAudio/lewton)
- [hound](https://github.com/ruuda/hound)

Library documentation was consulted on 2026-10-21. No graphics/audio prototype,
dependency integration, performance comparison, or target build was performed.

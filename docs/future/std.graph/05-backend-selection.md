# `Std.Graph` backend selection

**Status:** preferred stack selected for the first implementation.

## Required constraints

- Pure Rust crates, or crates that bundle what they need through Cargo.
- No SDL dependency.
- No separate graphics toolkit install for end users.
- Works on Windows, Linux, and macOS.
- Favors software-rendered 2D over GPU-heavy infrastructure.
- Suitable for full-frame fractal rendering.

## Selected first stack

### `winit`

Use for:

- native window creation
- event loop integration
- keyboard, resize, and close events

Why it is the right first choice:

- standard Rust windowing foundation
- broad desktop support
- no SDL-style end-user dependency
- active ecosystem support

### `softbuffer`

Use for:

- presenting a CPU-written pixel buffer to the native window

Why it is the right first choice:

- matches software-rendered 2D well
- integrates with `winit`
- avoids making GPU driver quality the baseline requirement
- fits Mandelbrot-, Julia-, and other software-rendered explorer workloads well

## Likely near-term companion crates

### `font8x8`

Strong candidate for the first text path because it gives deterministic bitmap text without font discovery.
Most likely useful soon after the foundation slice.

### `tiny-skia`

Good candidate when handwritten rasterization of lines, rectangles, circles, fills, and compositing starts to create noise.
It should be evaluated once the foundation slice is complete.

### `fontdue` / `ab_glyph`

Good candidates for scalable text later.
Not required for the Phase 1 MVP.

## Rejected as the first backend

### `pixels`

Useful, but not the preferred baseline because it brings `wgpu` and a GPU-oriented dependency stack into a problem that does not need it yet.

### `minifb`

Acceptable for a throwaway prototype, but not preferred as the long-term base because `winit` + `softbuffer` is a better fit for a durable runtime surface.

## Dependency policy

- Start with stable crate releases.
- Avoid optional backend matrices in Phase 1.
- Do not add a second rendering backend until the first one exposes a real limitation.
- Keep the dependency graph easy to audit from Cargo metadata alone.

## Re-evaluate only if

- `softbuffer` proves unreliable on one of the primary desktop targets,
- `winit` cannot satisfy a hard input or resize requirement, or
- the project deliberately changes course toward GPU-first rendering.
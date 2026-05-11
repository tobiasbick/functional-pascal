# Native graphics mode

> Priority: 10 — evaluate after the current terminal and TUI work has settled.
> Decision pending.

## Goal

Add a real windowed graphics mode for Functional Pascal that can support a
BGI-style API and Fractint-like programs:

- 2D only; no 3D requirement.
- Native window with a pixel buffer.
- Keyboard, mouse, resize, and close events.
- No extra SDK install such as SDL for end users.
- Target platforms: Windows, Linux, macOS. FreeBSD is desirable as a
  best-effort target.

## Current state

The current runtime is terminal-oriented:

- `Std.Console` exposes CRT-style text output, colors, cursor control, raw mode,
  alternate screen, and structured terminal events.
- `Std.Tui` provides a Rust-hosted event loop and redraw model for terminal
  applications.
- The Mandelbrot showcase already proves that FPAS can drive interactive,
  colorful, event-based rendering today, but only through terminal cells.

This means a real graphics mode should be a new runtime surface, not an
extension of the terminal path.

## Selection criteria

- Pure Rust crates or crates that bundle what they need through Cargo.
- No SDL dependency and no separate graphics toolkit installation.
- Works on Windows, Linux, and macOS.
- Prefer software-rendered 2D over GPU-heavy infrastructure for the first
  version.
- Must be suitable for fractal rendering, so bulk frame updates matter more than
  single-pixel calls.

## Candidate crates

### `winit`

Role: native window creation, event loop, keyboard, mouse, resize, close.

Why it fits:

- Standard Rust windowing building block.
- Supports the major desktop backends used by the Rust ecosystem.
- No SDL-style external toolkit requirement.
- Stable line available (`0.30.13`), while `0.31` is currently beta.

Limitations:

- Windowing only. It does not provide drawing.

Verdict: strong yes for the window/event layer.

### `softbuffer`

Role: CPU-written pixel buffer presented to a native window.

Why it fits:

- Explicitly designed for cross-platform software rendering.
- Integrates with `winit`.
- Good match for BGI-style 2D and Fractint-like software rendering.
- Avoids dependence on GPU acceleration and related driver issues.

Platform notes from its documentation:

- Tier-1 desktop support for AppKit, Win32, Wayland, and X11.
- More portable than GPU-backed framebuffers when hardware acceleration is not
  available or is unreliable.

Limitations:

- Only provides the pixel buffer. All drawing primitives must be implemented by
  us or delegated to another crate.

Verdict: best fit for the first graphics backend.

### `pixels`

Role: GPU-backed pixel framebuffer.

Why it fits:

- Well-known crate for pixel-oriented 2D applications.
- Good for scaling and display of a framebuffer.

Limitations:

- Built on `wgpu`, so it depends on a working graphics stack.
- More exposed to driver issues, VM limitations, and older hardware.
- Solves a bigger problem than the project currently needs.

Verdict: viable, but not the preferred first backend.

### `minifb`

Role: simple window + framebuffer crate.

Why it fits:

- Very small and straightforward API.
- Explicitly supports Windows, macOS, and Linux; X11 on Linux/FreeBSD-style
  systems is documented.

Limitations:

- Its own documentation mentions Linux build dependencies.
- `softbuffer` documentation explicitly calls out `minifb` as less desirable for
  long-term window management.
- Less attractive as a foundational runtime layer.

Verdict: acceptable for a throwaway prototype, not recommended as the long-term
backend.

### `tiny-skia`

Role: CPU 2D rasterization primitives.

Why it fits:

- Pure Rust 2D rasterizer.
- Good match for lines, rectangles, circles, fills, clipping, and image
  composition.
- Small and easy to build compared to large native graphics engines.

Limitations:

- No text rendering.
- Not a windowing crate.

Verdict: useful optional companion crate, not a complete solution by itself.

### Text crates

For text output inside a graphics mode, two practical directions exist:

- `font8x8`: embedded bitmap font, minimal and deterministic.
- `fontdue` or `ab_glyph`: real TTF/OTF font parsing and glyph rasterization.

Trade-off:

- `font8x8` is ideal for a first BGI-style text path with zero font discovery.
- `fontdue` / `ab_glyph` are better if scalable text is wanted later.

## Recommended stack

For a first real graphics mode:

1. `winit` for window and input.
2. `softbuffer` for presenting a CPU framebuffer.
3. `font8x8` for initial text drawing.
4. Optional later: `tiny-skia` for higher-level 2D primitives.
5. Optional later: `fontdue` or `ab_glyph` for scalable text.

This combination best matches the project requirements:

- no SDL,
- no mandatory GPU path,
- native desktop windows,
- suitable for BGI-like immediate-mode drawing,
- suitable for Fractint-like full-frame software rendering.

## API implications for FPAS

The graphics mode should likely be a new unit, not part of `Std.Console` or
`Std.Tui`.

Suggested direction:

- Unit name: `Std.Graph`.
- Core operations: `Open`, `Close`, `Width`, `Height`, `Clear`, `Present`,
  `PutPixel`, `Line`, `Rectangle`, `Circle`, `OutTextXY`, `PollEvent`.
- Prefer a framebuffer-oriented runtime API internally, even if Pascal exposes
  BGI-like procedures.

Important performance note:

- Fractal workloads should not be implemented as repeated VM calls to
  `PutPixel` alone.
- The runtime should support efficient frame writes or scanline updates behind
  the Pascal surface.

## Cross-platform notes

- Windows: good expected support with `winit` + `softbuffer`.
- macOS: good expected support with AppKit.
- Linux: good expected support on Wayland and X11.
- FreeBSD: likely possible as a best-effort target through X11 or Wayland, but
  not as strong a promise as the three main desktop platforms.

This still depends on the platform window system being present. That is normal
desktop infrastructure, not an extra multimedia SDK such as SDL.

## Recommendation

Keep this as a future feature and, when implementation starts, prototype it with
`winit` + `softbuffer` + `font8x8`.

Do not start with `pixels` unless a GPU-backed path becomes a deliberate project
goal. Do not build the first version on `minifb` unless the goal is only to
validate the FPAS-facing API very quickly.
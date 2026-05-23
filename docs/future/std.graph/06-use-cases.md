# `Std.Graph` target use cases

**Status:** target capability definition for the first useful graphics release.

## Primary target programs

`Std.Graph` should be good enough to build:

- [ ] a Mandelbrot explorer
- [x] a Julia explorer
- [x] simple pixel-oriented scientific or mathematical visualizers
- [x] 2D viewers with text overlays and direct keyboard / mouse interaction

## Required capability set

The first useful public graphics release should support all of the following:

- [x] open a native window
- [x] draw and present a software framebuffer
- [x] set individual pixels when needed
- [x] draw lines and filled rectangles for overlays and UI hints
- [x] draw simple text for status bars, coordinates, iteration counts, and help
- [x] read keyboard input
- [x] read mouse position, button, drag, and wheel input
- [x] react to resize and close requests

## Interaction model

The target interaction level is closer to an explorer application than to a retained widget toolkit.

- [ ] keyboard for quit, palette changes, parameter tweaks, and movement
- [x] mouse click or drag for re-centering or selection
- [x] mouse wheel for zoom
- [x] text overlays for current state and available controls

## Explicit non-goals

The plan does **not** require:

- BGI compatibility
- Turbo Vision style widgets
- multi-window document interfaces
- 3D rendering

## Release bar for "useful graphics"

The plan should consider `Std.Graph` genuinely useful once the repository can ship:

- [ ] one Mandelbrot example with keyboard and mouse navigation
- [x] one Julia example with parameter changes and redraw
- [x] a small text overlay showing current parameters and controls

That bar is more important than matching any historical graphics API.
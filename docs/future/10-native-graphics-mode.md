# Native graphics mode

> Priority: 10 - evaluate after the current terminal and TUI work has settled.
> Decision pending.

## Goal

Add a real windowed graphics mode for Functional Pascal that can support 2D,
framebuffer-oriented programs with direct drawing and interaction.

## Direction

The native graphics effort now has a dedicated planning set under
[std.graph/README.md](std.graph/README.md).

That split keeps the work traceable by topic instead of growing one large note.

## Current high-level decisions

- [x] The feature should be a new standard unit: `Std.Graph`.
- [x] The first milestone should prove one native window, event polling, and bulk framebuffer presentation.
- [x] The intended release target is a small modern 2D graphics surface with pixels, lines, text, and direct input handling.
- [x] The preferred initial backend is `winit` + `softbuffer`.
- [x] BGI compatibility is not a goal.

## Reading order

1. [std.graph/01-mvp.md](std.graph/01-mvp.md)
2. [std.graph/02-pascal-surface.md](std.graph/02-pascal-surface.md)
3. [std.graph/03-runtime-architecture.md](std.graph/03-runtime-architecture.md)
4. [std.graph/04-implementation-plan.md](std.graph/04-implementation-plan.md)
5. [std.graph/05-backend-selection.md](std.graph/05-backend-selection.md)
6. [std.graph/06-use-cases.md](std.graph/06-use-cases.md)
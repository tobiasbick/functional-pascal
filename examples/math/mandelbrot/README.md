# Mandelbrot Showcase

This project is the current showcase example for Functional Pascal in a real terminal application.

This folder also contains `mandelbrot_graph.fpas`, a single-file native-window `Std.Graph` Mandelbrot explorer.

It demonstrates:

- multi-file projects with `.fpasprj`
- `unit` decomposition across color and render modules
- record methods via the `Complex` helper type
- enum-driven palette selection
- fork-join parallelism with one `go` task per rendered row
- truecolor `Cell` values built with `RgbColor`
- one bulk `WriteCells` call per concurrently computed row
- `BeginFrame` / `Present` redraws so the fractal and HUD are flushed together
- raw-mode event handling with an exhaustive `case` on `EventKind` (key, mouse wheel, resize, paste, focus)
- `EnableMouse` / `EnableFocus` / `EnablePaste` paired with matching `Disable*` on shutdown
- resize awareness, alternate screen usage, and a live HUD

## Run

```sh
cargo run -p fpas-cli -- run examples/math/mandelbrot/mandelbrot.fpasprj
fpas run examples/math/mandelbrot/mandelbrot_graph.fpas
```

The terminal project is also the full-screen example for the
[`Std.Console` cell/frame API](../../../docs/pascal/std/console/cells-frames.md).

## Controls

### `mandelbrot_graph.fpas`

- Arrow keys: pan
- `+` / `-`: zoom in and out
- Mouse wheel: zoom
- Left click on the fractal: center the view on that pixel
- Middle click or `R`: reset the view
- `PageUp` / `PageDown`: increase or decrease iteration depth
- `1`, `2`, `3`: switch color palettes
- `Esc`: quit

### `mandelbrot.fpasprj`

- Arrow keys: pan
- `+` / `-`: zoom in and out
- Mouse wheel: zoom (terminals that report SGR mouse mode)
- Left click on the fractal: center the view on that cell
- Middle click on the fractal: same reset as `R`
- `PageUp` / `PageDown`: increase or decrease iteration depth
- `1`, `2`, `3`: switch color palettes
- `R`: reset the view
- `Esc`: quit
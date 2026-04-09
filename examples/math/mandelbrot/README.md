# Mandelbrot Showcase

This project is the current showcase example for Functional Pascal in a real terminal application.

It demonstrates:

- multi-file projects with `.fpasprj`
- `unit` decomposition across color and render modules
- record methods via the `Complex` helper type
- enum-driven palette selection
- fork-join parallelism with one `go` task per rendered row
- colorful console rendering that works with the current CRT-style screen model
- raw-mode event handling with an exhaustive `case` on `EventKind` (key, mouse wheel, resize, paste, focus)
- `EnableMouse` / `EnableFocus` / `EnablePaste` paired with matching `Disable*` on shutdown
- resize awareness, alternate screen usage, and a live HUD

## Run

```sh
cargo run -p fpas-cli -- examples/math/mandelbrot/mandelbrot.fpasprj
```

## Controls

- Arrow keys: pan
- `+` / `-`: zoom in and out
- Mouse wheel: zoom (terminals that report SGR mouse mode)
- Left click on the fractal: center the view on that cell
- Middle click on the fractal: same reset as `R`
- `PageUp` / `PageDown`: increase or decrease iteration depth
- `1`, `2`, `3`: switch color palettes
- `R`: reset the view
- `Esc`: quit
# Mandelbrot Showcase

This project is the current showcase example for Functional Pascal in a real terminal application.

It demonstrates:

- multi-file projects with `.fpasprj`
- `unit` decomposition across color and render modules
- record methods via the `Complex` helper type
- enum-driven palette selection
- fork-join parallelism with one `go` task per rendered row
- colorful console rendering that works with the current CRT-style screen model
- raw-mode event handling, resize awareness, alternate screen usage, and a live HUD

## Run

```sh
cargo run -p fpas-cli -- examples/math/mandelbrot/mandelbrot.fpasprj
```

## Controls

- Arrow keys: pan
- `+` / `-`: zoom in and out
- `PageUp` / `PageDown`: increase or decrease iteration depth
- `1`, `2`, `3`: switch color palettes
- `R`: reset the view
- `Esc`: quit
# Mandelbrot Showcase

Fullscreen terminal Mandelbrot explorer (`Std.Tui`). Multi-unit `.fpasprj`.

Demonstrates: units, record methods/statics/`Zoom` property, postfix chaining
(`Z.Sq().Add(C)`), fork-join row tasks, MVU state, `TuiCellGrid`, concrete
truecolor cells, panels, rules, gauges, status hints, and a modal overlay.

## Run

```sh
fpas run examples/math/mandelbrot/mandelbrot.fpasprj
```

## Controls

Arrow keys pan; `+`/`-` and wheel zoom; click centers; middle-click/`R` reset;
`PageUp`/`PageDown` iterations; `1`/`2`/`3` palettes; `H` opens the controls
overlay; `Esc` closes the overlay or quits.

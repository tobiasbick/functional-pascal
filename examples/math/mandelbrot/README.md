# Mandelbrot Showcase

Fullscreen terminal Mandelbrot explorer and the canonical interactive `Std.Tui`
example. The program project depends on a separately testable core library.

Demonstrates: units and project dependencies, record methods/statics/`Zoom`
property, postfix chaining (`Z.Sq().Add(C)`), MVU state, `TuiCellGrid`, concrete
truecolor cells, panels, rules, gauges, status hints, and a modal overlay.

The initial frame is blank and remains interactive while a render subscription
uses one cancellable task per row on the VM's CPU-sized worker pool. It publishes
one complete image through the application's bounded typed inbox. While a new image
is computing, the previous image remains visible if the dimensions have not changed.
Pan, zoom, palette, iteration, and resize changes replace the old subscription
with a new generation. Replacement requests cancellation without joining on the UI
path; while the old render stops, only the newest request is retained. Cancellation
is checked between pixels, in iteration chunks of at most 128 steps, and during waits
and blocked sends. Images from obsolete generations are ignored. Only the TUI host
replaces the grid and paints frames. Publishing complete images avoids both row gaps
and repeated layout/paint work for individual rows. The status shows `rendering` or `ready`.
At sixteen-pixel checkpoints, row tasks yield after at least eight milliseconds of work so
Console polling can share a small worker pool without suspending every short row.

## Run

```sh
fpas run examples/math/mandelbrot/mandelbrot.fpasprj
```

## Controls

Arrow keys pan; `+`/`-` and wheel zoom; click centers; middle-click/`R` reset;
`PageUp`/`PageDown` iterations; `1`/`2`/`3` palettes; `H` opens the controls
overlay; `Esc` closes the overlay or quits.

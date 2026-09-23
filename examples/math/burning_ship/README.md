# Burning Ship

Fullscreen terminal Burning Ship explorer (`Std.Console`).

Iteration: `Z.AbsComponents().Sq().Add(C)`.

## Run

```sh
fpas run examples/math/burning_ship/burning_ship.fpasprj
```

## Controls

Same as Mandelbrot (pan, zoom, click, iterations, palettes, `Esc`).

Camera geometry and terminal input/lifecycle are shared through
[`../explorer/`](../explorer/README.md); rendering and palettes remain local.

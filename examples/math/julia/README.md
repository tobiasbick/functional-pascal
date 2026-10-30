# Julia Showcase

Fullscreen terminal Julia-set explorer using `Std.Console` and a separate core library project.

Four row workers feed a bounded channel while the Console loop processes input and
draws available rows. Use `WASD` to adjust the Julia constant; the iteration uses
postfix calls via `Z.Sq().Add(C)`.

## Run

```sh
fpas run examples/math/julia/julia.fpasprj
```

## Controls

Arrows pan; `WASD` change `Jr`/`Ji`; `+`/`-` and wheel zoom; click centers;
`PageUp`/`PageDown` iterations; `1`/`2`/`3` palettes; `R` reset; `Esc` quits.

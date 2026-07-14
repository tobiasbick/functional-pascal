# Julia Examples

This folder contains two Julia-set demos:

- `julia.fpas` - the existing ASCII / CRT version
- `julia_graph.fpas` - the native-window `Std.Graph` version

## Run

```sh
fpas run examples/math/julia/julia.fpas
fpas run examples/math/julia/julia_graph.fpas
```

## `julia_graph.fpas` controls

- Arrow keys: pan
- `+` / `-`: zoom in and out
- Mouse wheel: zoom
- Left click: re-center on the clicked pixel
- Middle click or `R`: reset the view
- `W`, `A`, `S`, `D`: adjust the Julia constant
- `Esc`: quit
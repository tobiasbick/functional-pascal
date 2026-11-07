# Mandelbrot Explorer

Explore the Mandelbrot set in a fullscreen terminal on Windows or Linux.
The canvas uses RGB cells and follows the available space as the terminal
resizes. The status area shows the center, palette, iteration limit, zoom
and whether the image is ready.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application),
build and run from the repository root in a terminal.

```sh
dotnet build examples/tui/mandelbrot/MandelbrotExplorer.fpasproj
dotnet examples/tui/mandelbrot/bin/Debug/net10.0/MandelbrotExplorer.dll
```

| Input | Action |
| --- | --- |
| Arrow keys | Pan |
| Left click | Center on the clicked point |
| `+`, `=`, `-` or mouse wheel | Zoom |
| Page Up / Page Down | Change the iteration limit |
| `1`, `2`, `3` | Select Deep Ocean, Ember or Aurora |
| `R` or middle click | Reset camera, iterations and palette |
| `H` | Show the controls overlay |
| Esc | Close the overlay or exit the explorer |

Each change to the camera, palette, iteration limit or canvas size requests
a new image. A cancellable background command computes rows in parallel and
posts the complete image to the UI. The previous image stays visible while
the replacement is rendering if the size still matches.

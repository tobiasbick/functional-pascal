# Mandelbrot console rendering

This executable sample renders an 80 × 24 Mandelbrot view with an iteration
limit of 50. It uses typed int/double values, mutable locals, a function, numeric
casts, short-circuit conditions, while/counting loops, and an if expression.
Its source is the Mandelbrot program from the language examples.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet build examples/numerics/mandelbrot/Mandelbrot.fpasproj
dotnet examples/numerics/mandelbrot/bin/Debug/net10.0/Mandelbrot.dll
```

After changing packaged compiler tools, use a fresh SDK version/cache entry as
described in the bootstrap instructions. The integration suite packages current
tools under a unique version, builds this exact project/source, and checks every
rendered pixel against a .NET Complex calculation.

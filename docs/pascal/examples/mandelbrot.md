# Mandelbrot

## Example

```pascal
program Mandelbrot;

uses System.Console as Console;

function EscapeIterations(CReal: double; CImaginary: double; Limit: int): int;
begin
  var RealPart: double := 0.0;
  var ImaginaryPart: double := 0.0;
  var Iteration: int := 0;

  while Iteration < Limit and
    RealPart * RealPart + ImaginaryPart * ImaginaryPart <= 4.0 do
    const NextReal: double := RealPart * RealPart - ImaginaryPart * ImaginaryPart + CReal;
    ImaginaryPart := 2.0 * RealPart * ImaginaryPart + CImaginary;
    RealPart := NextReal;
    Iteration := Iteration + 1;
  end while;

  return Iteration;
end function;

begin
  const Width: int := 80;
  const Height: int := 24;
  const Limit: int := 50;

  for Y: int := 0 to Height - 1 do
    var Row: string := '';
    for X: int := 0 to Width - 1 do
      const CReal: double := -2.0 + 3.0 * cast<double>(X) / cast<double>(Width - 1);
      const CImaginary: double := -1.0 + 2.0 * cast<double>(Y) / cast<double>(Height - 1);
      const Count: int := EscapeIterations(CReal, CImaginary, Limit);
      Row := Row + if Count = Limit then '#' else ' ' end if;
    end for;
    Console.WriteLine(Row);
  end for;
end;
end program;
```

The [standalone project](../../../examples/mandelbrot/README.md) runs the same
program. Its MSBuild integration test verifies every pixel against a .NET
Complex calculation. The [Mandelbrot Explorer](../../../examples/tui/mandelbrot/README.md)
adds interactive pan, zoom, and color palettes in a fullscreen terminal.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)

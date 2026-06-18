# Fork-join

The idiomatic way to run parallel work is to spawn one task per unit of work and then wait for all results:

```pascal
program ParallelSum;
uses Std.Console, Std.Task;

function Compute(N: integer): integer;
begin
  return N * N
end;

begin
  var T1: task := go Compute(3);
  var T2: task := go Compute(4);
  WriteLn(Wait(T1) + Wait(T2))
end.
```

The Mandelbrot showcase project in `examples/math/mandelbrot/` demonstrates this pattern: one task per row, all collected in order via `Wait`, combined with a live terminal UI.

## See also

- [`go`](go.md)
- [Task handles](task-handles.md)

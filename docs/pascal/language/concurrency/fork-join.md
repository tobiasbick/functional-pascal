# Fork-join

The idiomatic way to run parallel work is to spawn one task per unit of work and then wait for all results:

```pascal
program ParallelSum;

uses Std.Console as Console, Std.Tasks as Tasks;

function Compute(N: integer): integer;
begin
  return N * N;
end function;

begin
  scope
    const T1: task of integer := go Compute(3);
    const T2: task of integer := go Compute(4);
    Console.WriteLn(Tasks.Wait(T1) + Tasks.Wait(T2));
  end scope;
end.
```

The Mandelbrot showcase project in `examples/math/mandelbrot/` demonstrates this pattern: one task per row, all collected in order via `Wait`, combined with a live terminal UI.

## See also

- [`go`](go.md)
- [Task handles](task-handles.md)

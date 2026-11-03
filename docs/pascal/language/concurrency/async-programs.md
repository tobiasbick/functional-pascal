# Async programs

Mark the program async to await APIs before the process completes.

```pascal
async program AsyncHello;

uses System.Console as Console, System.Threading.Tasks as Tasks;

begin
  await Tasks.Task.Delay(10);
  Console.WriteLine('Ready');
end;
end program;
```

The compiler generates the executable entry-point bridge and waits for the
program task before process completion. A plain `program` cannot await.
User functions do not receive an implicit blocking bridge.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

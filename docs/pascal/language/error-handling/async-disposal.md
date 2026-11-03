# Asynchronous disposal

Await asynchronous resource cleanup before leaving its lexical scope.

```pascal fragment
await using const Stream: IO.FileStream := IO.File.OpenRead('world.json');
const Buffer: byte[] := new byte[128];
const Count: int := await Stream.ReadAsync(Buffer, 0, Buffer.Length);
Console.WriteLine(Count);
```

`await using const` has the same declaration syntax, enclosing scope, reverse
cleanup order, and null handling. It requires an async routine/program and
`System.IAsyncDisposable`. Its cleanup awaits `DisposeAsync()` before completing
each exit. Plain `using const` selects IDisposable; `await using const` selects
IAsyncDisposable, even when the type supports both. It does not infer cancellation
or invoke synchronous Dispose as well. Synchronous and asynchronous resource
declarations may share an async scope; cleanup still follows reverse acquisition
order, awaiting each asynchronous cleanup before proceeding to the next.

Within an async routine, an ordinary synchronous using declaration remains legal.
An `await` is not allowed inside a catch filter. Other supported await locations,
including catch and finally bodies, follow the generated async state machine's
normal CLR behavior.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

# Synchronous disposal

Tie IDisposable cleanup to a lexical scope with using const.

```pascal fragment
using const Reader: IO.StreamReader := new IO.StreamReader('world.json');
Console.WriteLine(Reader.ReadToEnd());
```

`using const Name: T := Expression;` is a local resource declaration, not a block
opener. Both `const` and the type annotation are required; `using var` is not
supported. The binding is immutable and cannot be passed as ref/out. It is visible
after its declaration until the end of the enclosing lexical scope. Resource
declarations are not allowed as top-level bindings or record fields.

Cleanup runs when that enclosing scope exits, including by normal completion,
return, exception, or break/continue leaving the scope. Routine/lambda and program
entry bodies, nested `begin ... end;` blocks, individual branches/case arms,
loop iteration bodies, and try/catch/finally bodies each form scopes. A resource
declared in a loop body is acquired and released separately for each iteration.
There is no separate using-body delimiter. For a shorter lifetime, add an ordinary
nested block:

```pascal fragment
begin
  using const Reader: IO.StreamReader := new IO.StreamReader('world.json');
  Console.WriteLine(Reader.ReadToEnd());
end;
// Reader has been disposed and is no longer in scope.
Console.WriteLine('Read finished');
```

The initializer executes once and must be implicitly convertible to the declared
type. That type must implement `System.IDisposable`; a nullable resource is
allowed and is disposed only when non-null. A failed initializer has no acquired
resource to dispose. The compiler protects the remaining scope with try/finally,
invoking Dispose once for each successfully acquired non-null resource on exit.
Ordinary const/var declarations do not schedule disposal.

Declare multiple resources in consecutive `using const` statements; nested
cleanup releases them in reverse acquisition order. If a later initializer or
cleanup throws, cleanup for earlier acquisitions still runs. A declaration in a
try body is cleaned up before its catch/finally bodies execute; exception filters
retain CLR first-pass ordering and may run before cleanup. A declaration outside
that try remains alive until its enclosing scope exits. Cleanup exceptions
follow ordinary CLR exception replacement rules. There is no pattern-based Dispose
lookup. Imported value-type disposables are supported without replacing their
resource with a copied boxed instance for cleanup.

Using does not establish a linear ownership type. A referenced object may still
escape, including through a closure, or be disposed by other code. Using guarantees
the scheduled cleanup call; it neither extends the lifetime for escaping aliases
nor ensures those aliases remain usable afterward.

## Example

```pascal
program ResourceLifetime;

uses System as Sys, System.IO as IO, System.Console as Console;

begin
  const Stream: IO.MemoryStream := new IO.MemoryStream();
  begin
    using const Owned: IO.MemoryStream := Stream;
    Owned.WriteByte(65);
  end;
  if Stream.CanRead then raise new Sys.InvalidOperationException('Resource was not disposed'); end if;
  Console.WriteLine('Resource released');
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

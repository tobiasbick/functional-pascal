# Errors and resources

## Exceptions

CLR exceptions propagate normally through FPAS and .NET calls. `try`/`catch`,
`raise`, and `finally` use the CLR exception mechanism directly.

```pascal
try
  Console.WriteLine(IO.File.ReadAllText('world.json'));
catch Failure: IO.IOException do
  Console.WriteLine(Failure.Message);
catch Failure: Sys.Exception when Failure.Message.Length > 0 do
  Console.WriteLine('Another failure');
  raise;
finally
  Console.WriteLine('Read attempt finished');
end try;
```

Catch clauses are ordered. Their types must derive from `System.Exception`.
`when` supplies a CLR exception filter; throwing from a filter counts as a false
filter according to CLR behavior. `catch do` catches any ordinary managed
exception without a binding. Unreachable catches are errors. A `try` statement
requires a catch or finally clause, or both.

`raise Expression;` throws a non-null exception. `raise;` is valid only within
the lexical body of a catch and rethrows while preserving the original stack.
It is not allowed in a nested routine merely declared within that catch.
`finally` runs on normal and exceptional exits, including return.
Control transfers out of `finally` are forbidden. If cleanup throws during
another exception, ordinary CLR exception replacement rules apply.

Use standard exception types and catch at the boundary that can meaningfully
handle the failure. There is no `panic` primitive, automatic exception-to-union
conversion, or language-defined error wrapper.

False `requires`/`ensures` conditions throw System.InvalidOperationException.
Exceptions raised while evaluating their expressions propagate unchanged.
Postconditions run only on successful exit after body cleanup; they do not run
on a fault/cancellation. See [contracts](contracts.md) for the entry/exit boundary
and async Task behavior. These checks use the same CLR exception mechanism.

## Absence and expected outcomes

Use CLR nullability (`T?`) for a value that may be absent. Use the API's existing
Try-pattern where provided:

```pascal
var Count: int := 0;
if int.TryParse('42', out Count) then
  Console.WriteLine(Count);
else
  Console.WriteLine('Expected an Int32 value');
end if;
```

Application unions describe normal domain alternatives, such as a move that
succeeds or is blocked. They are ordinary return values, not another exception
mechanism. Each application chooses its meaningful cases and handles them
explicitly; see [types](types.md#absence-failures-and-domain-outcomes).

There is no built-in Option/Result convention or postfix failure-propagation
operator. A union returned by a .NET or FPAS call is never automatically unwrapped.
An exception thrown while computing any return value still propagates normally.

## Synchronous disposal

```pascal
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

```pascal
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

## Asynchronous disposal

```pascal
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

# Exceptions

CLR exceptions propagate normally through FPAS and .NET calls. `try`/`catch`,
`raise`, and `finally` use the CLR exception mechanism directly.

```pascal fragment context=body
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
`when` uses CLR exception-filter semantics; throwing from a filter counts as a
false filter. `catch do` catches any ordinary managed
exception without a binding. Unreachable catches are errors. A `try` statement
requires a catch or finally clause, or both.

`raise Expression;` throws a non-null exception. `raise;` is valid only within
the lexical body of a catch and rethrows while preserving the original stack.
It is not allowed in a nested routine merely declared within that catch.
`finally` runs on normal and exceptional exits, including return.
Control transfers out of `finally` are forbidden. If cleanup throws during
another exception, ordinary CLR exception replacement rules apply.

Use standard exception types and catch at the boundary that can meaningfully
handle the failure. There is no `panic` primitive, automatic exception-to-variant
conversion, or language-defined error wrapper.

False `requires`/`ensures` conditions throw System.InvalidOperationException.
Exceptions raised while evaluating their expressions propagate unchanged.
Postconditions run only on successful exit after body cleanup; they do not run
on a fault/cancellation. See [contracts](../functions/contracts.md) for the entry/exit boundary
and async Task behavior. These checks use the same CLR exception mechanism.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

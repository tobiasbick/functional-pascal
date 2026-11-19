# FPAS for agents

FPAS is a function-first Pascal language with selected Ada conventions that
compiles directly to CLR assemblies. Use the topic guides for FPAS and the
[CLI reference](cli.md) for every public command. The [handbook](../README.md)
owns detailed semantics and the [grammar](../../specs/syntax.md) defines syntax.

## A complete program

```pascal
program AgentStart;

uses System as Sys, System.Console as Console;

function Double(Value: int): int;
requires Value >= int.MinValue div 2 and Value <= int.MaxValue div 2;
ensures (Answer: int): Answer = Value * 2;
begin
  return Value * 2;
end function;

begin
  const Answer: int := Double(21);
  if Answer <> 42 then
    raise new Sys.InvalidOperationException('Incorrect answer');
  end if;

  Console.WriteLine(Answer);
end;

end program;
```

Save sources as UTF-8 `.fpas`. Projects use `.fpasproj` and solutions `.slnx`.
Read [CLI](cli.md) for compilation and execution.

## Rules before writing

- Keywords and identifiers are case-sensitive. Keywords are lowercase. Escape
  foreign names that match an FPAS keyword with `@`, as in `Value.@end`.
- Declare each `const` or `var` with a type and initializer, as in `const Count: int := 3;`.
  Parameters, function results, fields, loop variables, and pattern bindings also need types.
- Use CLR aliases such as `int`, `long`, `double`, `decimal`, `bool`, and `string`.
  A `char` is one UTF-16 code unit. Strings use apostrophes, doubled inside text.
- `const` prevents rebinding. Arrays and FPAS records are references. Imported
  collections follow their CLR type. A const reference can still point to a mutable object.
- Import namespaces, types, and units explicitly with aliases. Call real .NET
  APIs through those aliases. Mark declarations and record members `public` to
  expose them. Unmarked record members are private to the declaring record.
- End statements with `;`, including before `else`. Close constructs by name.
  A program's entry block ends with `end;`, then the declaration ends with
  `end program;`. A unit ends with `end unit;`.
- Parameters use semicolons; arguments use commas. Consume function results or
  write `discard Call();`. Spell `ref`, `out`, and `in` in both declarations and calls.
- Construct records with `new Point(X := 1, Y := 2)`. Variants have qualified cases
  and exhaustive matching. Use CLR nullability and exceptions for absence/failure.
- Tasks and cancellation use ordinary .NET APIs. Resources use `using const` or
  `await using const`; cleanup runs when the enclosing lexical scope exits.
- At entry routines, state caller obligations with `requires` and guarantees on
  successful return with `ensures`. Examples include a program's main routine
  and public library operations.
- Use `Fpas.Workflows` for deferred composition of asynchronous work with typed
  outcomes, dependencies, or shared execution policies.

## Read for the task

| Task | Read |
| --- | --- |
| Values, arithmetic, branches, loops, arrays, and strings | [Basics](basics.md) |
| Records, domains, variants, generics, units, and global state | [Types and units](types.md) |
| Routines, parameters, delegates, closures, inference, and contracts | [Functions](functions.md) |
| Matching, nullability, exceptions, and resource lifetime | [Patterns, errors, and resources](patterns-errors.md) |
| .NET APIs, interfaces, events, attributes, and CLR representation | [Interop](interop.md) |
| Async functions, cancellation, shared state, and composable effects | [Concurrency](concurrency.md) |
| Build, test, refactor, inspect APIs, debug, profile, or publish | [CLI](cli.md) |
| Change the compiler or run repository tests | [Development workflows](tools.md) |
| Update examples, check links, or verify the token budget | [Maintenance](maintenance.md) |
| Change CUI integration or prepare an upstream contribution | [CUI workflow](../../cui-contributions.md) |

Use the owning handbook/specification for details. Check implementation status
before using a feature from a proposal.

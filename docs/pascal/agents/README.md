# FPAS for agents

FPAS is a function-first Pascal language with selected Ada conventions, compiling
directly to CLR assemblies. Read this page before writing FPAS, then load the
topics needed for the task. The [handbook](../README.md) owns detailed semantics;
the [grammar](../../specs/syntax.md) defines syntax. This reference describes
implemented behavior. Deferred proposals stay in [language gaps](../../future/language-gaps.md).

## A complete program

```pascal
program AgentStart;

uses System as Sys, System.Console as Console;

function Double(Value: int): int;
begin
  return Value * 2;
end function;

begin
  const Answer: int := Double(21);
  if Answer <> 42 then raise new Sys.InvalidOperationException('Incorrect answer'); end if;
  Console.WriteLine(Answer);
end;
end program;
```

Save sources as UTF-8 `.fpas`. Projects use `.fpasproj` and solutions `.slnx`.
Read [tools](tools.md) for compilation, execution, and the development workflow.

## Rules before writing

- Keywords and identifiers are case-sensitive. Keywords are lowercase. Escape
  foreign keyword names with `@`, as in `Value.@event`.
- Every binding needs a type and initializer: `const Count: int := 3;`.
  Parameters, results, fields, loop variables, and pattern bindings also need types.
- Use CLR aliases such as `int`, `long`, `double`, `decimal`, `bool`, and `string`.
  A `char` is one UTF-16 code unit. Strings use apostrophes, doubled inside text.
- `const` prevents rebinding; `var` permits assignment. Arrays, collections, and
  records retain reference semantics. Const does not freeze referenced objects.
- Import namespaces, types, and units explicitly with aliases. Call real .NET
  APIs through those aliases. Top-level declarations are private unless public;
  record members are public unless private.
- End statements with `;`, including before `else`. Close constructs by name.
  A program's entry block ends with `end;`, then the declaration ends with
  `end program;`. A unit ends with `end unit;`.
- Parameters use semicolons; arguments use commas. Consume function results or
  write `discard Call();`. Writable caller storage uses explicit ref/out/in modes.
- Construct records with `new Point(X := 1, Y := 2)`. Unions have qualified cases
  and exhaustive matching. Use CLR nullability and exceptions for absence/failure.
- Tasks and cancellation use ordinary .NET APIs. Resources use `using const` or
  `await using const`; cleanup runs when the enclosing lexical scope exits.

## Read for the task

| Task | Read |
| --- | --- |
| Values, arithmetic, branches, loops, arrays, and strings | [Basics](basics.md) |
| Records, domains, unions, generics, units, and global state | [Types and units](types.md) |
| Routines, parameters, delegates, closures, inference, and contracts | [Functions](functions.md) |
| Matching, nullability, exceptions, and resource lifetime | [Patterns, errors, and resources](patterns-errors.md) |
| .NET APIs, interfaces, events, attributes, and CLR representation | [Interop](interop.md) |
| Async functions, tasks, cancellation, and shared state | [Concurrency](concurrency.md) |
| Build, test, inspect APIs, debug, profile, or change the compiler | [Tools](tools.md) |
| Update examples, check links, or verify the token budget | [Maintenance](maintenance.md) |

Use the owning handbook/specification for details. Check implementation status
before using a feature from a proposal. The two tracked baseline gaps and source
interface proposal remain deferred; this reference does not enable them.

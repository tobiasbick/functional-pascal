# Matching, absence, errors, and resources

Read this when handling union cases, nullable values, exceptions, or cleanup.

## Patterns and coverage

A union has named cases, with optional immutable payloads. Construct with
Outcome.Ready(42) or fieldless Outcome.Missing. A generic case may infer its owner
arguments from payloads and expected type. Null is not an implicit union case.

Case statements/expressions evaluate their selector once and try arms in source
order, without fallthrough. Bind payloads explicitly: when Outcome.Ready(const
Value: int). Each annotation exactly matches the selected static type; it is not
a cast/filter. Cover every payload position, using _ for ignored values. Named
payloads use := and cannot mix with positional payloads. Nested patterns and
comma-separated alternatives are supported; alternatives must bind the same
names/types. A guard follows the pattern, as in when Outcome.Ready(const N: int)
if N > 0:.

Unions require exhaustive nested coverage, and bool needs true/false or a
fallback. Integer/string/CLR-enum selectors require a catch-all or else; CLR enum
metadata cannot exclude unnamed values. A subrange can prove complete interval
coverage with constant/range patterns. Guards never prove coverage. Duplicate,
unreachable, and out-of-domain arms are errors. A fallback after full coverage
is also unreachable.

Use is T as Name for a type pattern with an arm-local non-null binding. The
ordinary expression Value is T tests CLR type compatibility without binding a
name. Case expressions require one expression and semicolon per arm.

## Absence and exceptions

Use T? for absence: nullable reference annotations or actual Nullable<T> for value
types. Null cannot initialize a non-null reference. Stable local/parameter null
tests refine applicable branches and short-circuit operands. Repeated property
reads and mutable captures cannot provide the same guarantee. A foreign overloaded
equality call does not prove nullness; a runtime is T test can validate the value.
Coalescing ?? evaluates its fallback only for null. Nullable<T>.Value can throw
when absent. There is no null-forgiving suffix.

Use ordinary CLR exceptions and existing Try-pattern APIs for failures.
Raise Expression throws a non-null exception; raise; rethrows only inside the
lexical catch body, preserving the original stack. Catch clauses are ordered,
may use when filters, and can omit the binding with catch do. A throwing filter
counts as false. Finally executes on normal/exceptional exits; control cannot
transfer out of it. An exception from cleanup replaces an earlier exception
according to CLR rules. Application unions model normal domain outcomes, without
implicit exception conversion or an Option/Result prelude.

## Resource lifetime

Declare using const Name: T := Expression for IDisposable, or await using const
for IAsyncDisposable in an async body. These declarations protect the remainder
of the enclosing lexical scope; they have no separate closer. Use begin/end for
a shorter lifetime. Successful acquisitions are cleaned up in reverse order on
normal completion, return, exceptions, and loop exits. Null resources are skipped;
failed acquisition schedules no cleanup. Earlier resources are still released
when a later acquisition or disposal fails. A resource in a try body is disposed
before its catch/finally bodies; filters can run before stack unwinding.

CLR GC owns managed memory; using provides scheduled external-resource cleanup.
Escaping references can outlive the scope but do not postpone disposal. The
language does not create ownership/borrowing guarantees for those aliases.

```pascal
program AgentPatterns;

uses System as Sys, System.IO as IO, System.Console as Console;

type Outcome = union
  Ready(Value: int);
  Missing;
end union;

begin
  const Result: Outcome := Outcome.Ready(4);
  const Amount: int := case Result of
    when Outcome.Ready(const Value: int) if Value > 0: Value;
    when Outcome.Ready(_): 0;
    when Outcome.Missing: 0;
  end case;
  var Parsed: int := 0;
  begin
    using const Reader: IO.StringReader := new IO.StringReader('42');
    const Text: string? := Reader.ReadLine();
    if Text <> null then discard int.TryParse(Text, out Parsed); end if;
  end;
  var Handled: bool := false;
  var Cleaned: bool := false;
  try
    raise new Sys.ArgumentException('Expected test failure');
  catch Failure: Sys.ArgumentException when Failure.Message.Length > 0 do
    Handled := true;
  finally
    Cleaned := true;
  end try;
  if Amount <> 4 or Parsed <> 42 or not Handled or not Cleaned then
    raise new Sys.InvalidOperationException('Incorrect pattern or cleanup behavior');
  end if;
  Console.WriteLine('Patterns verified');
end;
end program;
```

Read [control flow](../language/control-flow.md) for matching/coverage,
[nullable types](../language/types.md#nullable-types) for flow restrictions, and
[errors/resources](../language/errors-and-resources.md) for cleanup ordering.

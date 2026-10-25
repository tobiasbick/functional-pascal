# FPAS for agents

FPAS is a functional-first Pascal language with selected Ada conventions. Use
this reference when writing FPAS without prior language knowledge. Read this
page first, then load the topics needed for the task. The [handbook](../README.md)
contains detailed semantics and APIs; the [grammar](../../specs/grammar.ebnf)
defines syntax. Describe implemented behavior; proposals belong in `docs/future/`.

## A complete program

```pascal id=first-program run
program AgentStart;

uses Std.Console as Console, Std.Test as Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
  const Answer := Double(21);
  Test.AssertEquals(42, Answer);
  Console.WriteLn(Answer);
end.
```

Save a program as `.fpas` and run `fpas run file.fpas`. `Std.Test` is an ordinary
import used here for the assertion; console output requires `Std.Console`.
Use a `.fpasprj` when the program imports application units or project dependencies.

## Rules to know before writing

- Keywords and identifiers are case-insensitive. Strings use single quotes,
  doubling an embedded quote: `'it''s'`. Comments start with `//`.
- Repeat `type`, `const`, or `var` for each declaration. Bindings are initialized;
  `const` is immutable, and `var` permits reassignment. Local bindings can infer
  a fully determined type; parameters, results, fields and globals state types.
- Routines declare parentheses even with no parameters. Separate declared
  parameters with semicolons and supplied arguments with commas.
- Statements end with `;`, including before `else`. Close constructs by name:
  `end function;`, `end if;`, `end for;`, `end record;`, `end unit;`.
  A program's main block ends with `end.`. Functions return with `return`.
- Import every unit with an alias: `uses Std.Parse as Parse;`. Access its public
  names through that alias. Unit declarations and record members are private by default;
  each export needs its own `public`.
- Value parameters and loop variables are read-only. A `var` parameter changes
  caller storage; its argument also needs `var`, as in `Increase(var Counter)`.
- Arrays and strings start at index zero. String indexing returns one Unicode
  scalar as a string. Collection transformations return values; `InPlace` methods
  change writable storage.
- Construct records with named fields: `Point(X := 1, Y := 2)`. Qualify user enum
  variants: `Shape.Circle(3)`. Built-in `Some`, `None`, `Ok`, and `Error` keep
  their language spellings.
- `case` binds payloads with explicit `const`: `when Some(const Value):`.
  Closed variants require exhaustive cases. `Value is Some(_)` only tests a
  pattern. `try` propagates Result/Option failure from a compatible function.
- Tasks started with `go` need an owning `scope` in the same routine. Pass data
  and channels between tasks; mutable captures stay with their owning task.

## Read for the task

| When you need to… | Read |
| --- | --- |
| Declare values, compute, branch, loop, or use collections | [Basics](basics.md) |
| Write routines, change caller storage, store callbacks, or capture values | [Functions](functions.md) |
| Define records, enums, generics, domain types, or imported units | [Types and units](types.md) |
| Match variants, handle missing values, propagate or define errors | [Patterns and errors](patterns-errors.md) |
| Start tasks, coordinate channels, cancel, or manage child lifetimes | [Concurrency](concurrency.md) |
| Look up an API, navigate sources, validate, debug, or profile | [Tools](tools.md) |

Use `fpas doc <name>` for exact declarations and built-in operations instead of
guessing from another Pascal dialect. Format with `fpas fmt`, check with
`fpas check`, and run the relevant tests. [Tools](tools.md) explains structured
reports, project selection, and bounded runtime inspection.

Reference upkeep and the total token budget are defined in
[maintenance](maintenance.md).

# Declarations

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_decl`, `procedure_decl`).

## Declaration shape

```text
function Name [<T>] ( [ params ] ) : RetType ;
  // nested function | nested procedure
begin
  ...
end function;

procedure Name [<T>] ( [ params ] ) ;
  // nested function | nested procedure
begin
  ...
end procedure;
```

- The header ends with `;` before the body. Functions close with `end function;`
  and procedures with `end procedure;`, including nested and record routines.
- Every body statement ends with `;`. Use `null;` when a procedure performs no action.
- Use `()` when there are no parameters: `function Pi(): real;`.
- Parameter lists use `;` between individually typed parameters; call sites use
  `,`. Write `A: integer; B: integer`, rather than grouped `A, B: integer`.
- Calls always include parentheses, including calls without arguments: `Pi()` or
  `SayHello()`. A bare routine name is not call syntax.

## Functions

A function returns a value using `return`. Its caller must consume the result
or explicitly write `discard Function(...);`; see
[result consumption](first-class.md#result-consumption-and-discard).

```pascal
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;

```

## Procedures

A procedure performs an action but returns no value:

```pascal
uses Std.Console as Console;

procedure SayHello(Name: string);
begin
  Console.WriteLn(('Hello, ' + Name) + '!');
end procedure;
```

Procedures use bare `return` to exit early without a value:

```pascal
uses Std.Console as Console;

procedure LogIfPositive(InitialCount: integer; Value: integer);
begin
   var Count: integer := InitialCount;
  if Value <= 0 then
    return;
  end if;

  Count := Count + 1;
  Console.WriteLn('logged ', Value);
end procedure;
```

## See also

- [Parameters](parameters.md)
- [Early return](early-return.md)

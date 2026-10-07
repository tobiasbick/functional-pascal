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

- The header ends with `;` before the body. A function body ends with `end function;`, and a procedure body ends with `end procedure;`. Methods and nested routines use the same matching endings.
- Use `()` when there are no parameters: `function Pi(): real;`.
- Parameter lists use `;` between parameters; call sites use `,`.
- Calls always include parentheses, including calls without arguments: `Pi()` or
  `SayHello()`. A bare routine name is not call syntax.

## Functions

A function returns a value using `return`:

```pascal
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;
```

Consume every function result by assigning, returning, passing, or inspecting
it. To ignore a permitted result deliberately, write `discard Add(1, 2);`.
A plain `Add(1, 2);` statement is an error. See [discarding values](discard.md)
for task-handle and callable-capture restrictions.

## Procedures

A procedure performs an action but returns no value:

```pascal
procedure SayHello(Name: string);
begin
  WriteLn('Hello, ' + Name + '!');
end procedure;
```

Procedures use bare `return` to exit early without a value:

```pascal
procedure LogIfPositive(Count: integer; Value: integer);
begin
  var LocalCount: integer := Count;
if Value <= 0 then
    return;
  end if;

  LocalCount := LocalCount + 1;
  WriteLn('logged ', Value);
end procedure;
```

## See also

- [Parameters](parameters.md)
- [Early return](early-return.md)

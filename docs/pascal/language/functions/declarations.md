# Declarations

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`function_decl`, `procedure_decl`).

## Declaration shape

```text
function Name [<T>] ( [ params ] ) : RetType ;
  { nested function | nested procedure }
begin
  ...
end;

procedure Name [<T>] ( [ params ] ) ;
  { nested function | nested procedure }
begin
  ...
end;
```

- The header ends with `;` before the body. The body ends with `end;` (including top-level declarations in a program or unit).
- Use `()` when there are no parameters: `function Pi(): real;`.
- Parameter lists use `;` between parameters; call sites use `,`.

## Functions

A function returns a value using `return`:

```pascal
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end;
```

## Procedures

A procedure performs an action but returns no value:

```pascal
procedure SayHello(Name: string);
begin
  WriteLn('Hello, ' + Name + '!');
end;
```

Procedures use bare `return` to exit early without a value:

```pascal
procedure LogIfPositive(mutable Count: integer; Value: integer);
begin
  if Value <= 0 then
    return;
  Count := Count + 1;
  WriteLn('logged ', Value);
end;
```

## See also

- [Parameters](parameters.md)
- [Early return](early-return.md)

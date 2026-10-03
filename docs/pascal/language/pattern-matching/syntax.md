# Case syntax

```text
case <expression> of
  when <label> { , <label> } [ if <boolean> ] :
    <statement> ; { <statement> ; }
  ...
[ else
    <statement> ; { <statement> ; } ]
end case;
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

- Each arm starts with `when` and contains a nonempty statement list.
- Every statement ends with `;`, including the last one before `when`, `else`,
  or `end case;`. The formatter preserves these required terminators.
- Use `null;` for a body that performs no action. A standalone `;` is invalid.
- Each arm has its own local declaration scope. An explicit `begin ... end;`
  adds a nested scope (see [Scalar labels — block arms](scalar-labels.md#block-arms)).

## Example

```pascal
program StatusExample;

uses Std.Console as Console;

begin
  var Status: integer := 1;
  case Status of
    when 0:
      var Message: string := 'ok';
      Console.WriteLn(Message);
    when 1, 2:
      var Message: string := 'retry';
      Console.WriteLn(Message);
    else
      null;
  end case;
end program;
```

## See also

- [Scalar labels](scalar-labels.md)

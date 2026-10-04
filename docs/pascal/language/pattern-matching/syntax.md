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
  const Status: integer := 1;
  case Status of
    when 0:
      const Message: string := 'ok';
      Console.WriteLn(Message);
    when 1, 2:
      const Message: string := 'retry';
      Console.WriteLn(Message);
    else
      null;
  end case;
end program;
```

## Case expressions

In a required value position, `case` selects one expression:

```pascal
const Message: string := case Status of
  when 0: 'ok';
  when 1, 2: 'retry';
  else 'unknown';
end case;
```

Each arm and fallback contains exactly one expression followed by `;`. The
surrounding declaration or statement has its own terminator after `end case`.
There is no implicit value from a statement list. The scrutinee runs once; guards
and selected values run in written order, and unselected values do not run.

All branches must agree on one ordinary type. An expected type supplies context
for empty collections and generic constructors; otherwise branch inference uses
all values and does not depend on their order. Incompatible branch types and
unresolved empty values require a compatible annotation or corrected values.
A procedure call cannot be a branch value; a stored procedure value can.

Pattern bindings are local to their arm and visible in its guard and value.
Grouped patterns must introduce identical binding names and types. Coverage and
fallback rules are described in [Exhaustiveness](exhaustiveness.md).

## See also

- [Scalar labels](scalar-labels.md)

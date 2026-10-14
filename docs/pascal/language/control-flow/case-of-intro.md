# Case of intro

A `case` statement selects the first arm whose label and optional guard match.
The expression is evaluated once. Supported expression types are:

- ordinal types: `integer`, `boolean`, an `enum`, or an integer subrange
- `string`
- `Result of (T, E)` or `Option of T`
- a [distinct type](../types/distinct-types.md#case) over `integer`, `string`,
  or `boolean`, with labels of the same distinct type

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_arm`, `case_label`, `case_expression`).

Each arm starts with `when Labels [if Guard]:` and contains a nonempty list of
statements. The next `when`, optional final `else`, or `end case` ends the list.
Every statement ends with `;`, including the last statement in each arm; the
whole case ends with `end case;`. Use `null;` for an arm with no action.
A case requires at least one `when` arm. Arms do not fall through.

Statements and expressions must cover every possible input. Use `if` for
selective handling. The same rules apply in both positions:

| Selector | Required coverage | `else` |
|----------|-------------------|--------|
| Enum, Option, Result | Every variant explicitly, including nested patterns | Forbidden |
| Boolean | Both true and false | Forbidden |
| Subrange | All declared values, using values or ranges | Forbidden |
| Integer, string | Explicit labels or guarded arms plus a fallback | Required |
| Distinct | The rules of its supported underlying type, with typed labels | Follows the underlying type |

Guards never contribute coverage, even constant-true guards. Duplicate labels
within an arm and labels fully covered by earlier unguarded arms are errors.
Partial overlaps are allowed and follow written order.

```pascal
case Day of
  when 'Monday':
    Console.WriteLn('Start of week');
  when 'Friday':
    Console.WriteLn('Almost weekend');
  when 'Saturday', 'Sunday':
    Console.WriteLn('Weekend');
  else
    Console.WriteLn('Midweek');
end case;
```

Ranges use `..`; comma-separated labels share one body:

```pascal
case Score of
  when 0..59:
    Grade := 'F';
  when 60..69:
    Grade := 'D';
  when 70..79:
    Grade := 'C';
  when 80..89:
    Grade := 'B';
  when 90..100:
    Grade := 'A';
  else
    Grade := '?';
end case;
```

## Arm scopes and guards

Each arm, including `else`, has its own local scope. Body declarations cannot
be used in its guard, another arm, or after the case. Pattern bindings are
available in the guard and selected body. An explicit `begin ... end;` inside
an arm retains an additional scope; its ending closes only that inner block.

```pascal
case Items of
  when Some(const Value) if Value > 0:
    const Doubled: integer := Value * 2;
    Console.WriteLn(Doubled);
    Console.WriteLn('positive');
  when Some(const Value):
    null;
  when None:
    Console.WriteLn('missing');
end case;
```

Guarded arms do not count toward coverage. Missing coverage reports FP3011;
`else` on a finite selector reports FP3035, even after complete coverage.
Use an [`is` test](../pattern-matching/is-test.md) for selective variant checks.
See [Pattern matching](../pattern-matching/README.md) for guards, scalar guard
bindings, data-enum patterns, and exhaustiveness rules. Destructuring keeps
forms such as `Ok(const Value)`, `Error(const Reason)`, `Some(const Value)`,
and `None`.

## Case expressions

Where a value is expected, `case` selects the value of the first matching arm:

```pascal
function Area(Value: Shape): real;
begin
  return case Value of
    when Shape.Circle(const R): 3.14159 * R * R;
    when Shape.Rectangle(const W, const H): W * H;
  end case;
end function;

const Label: string := case Count of
  when 0: 'none';
  when 1: 'one';
  else 'many';
end case;
const State: string := case Enabled of
  when true: 'enabled';
  when false: 'disabled';
end case;
```

- Each arm is `when Labels [if Guard]: Value;` with exactly one expression
  ended by `;`. An `else` arm is `else Value;`. `end case` closes the
  expression; the enclosing statement or declaration supplies its terminator.
- Labels, guards, pattern bindings, and arm scopes follow the statement. Only
  the selected arm value is evaluated.
- Every input must produce a value, using the same coverage and `else` matrix
  as a statement. Boolean and subrange expressions use explicit finite coverage;
  integer/string expressions require `else`. Distinct follows its underlying type.
- All arm values have one type, with the same rules as
  [`if` expressions](if-then-else.md#if-expressions): no numeric widening, and
  `None`, `[]`, or unresolved generic call results take context from the other
  arms. The enclosing expected result also supplies context. Pattern bindings
  remain available only in their own arm during inference.
- When the selector, every label, and every arm value are compile-time constants
  and no arm uses a guard, binding, or destructuring pattern, the expression is
  a compile-time constant.

A `case` at the start of a statement is always the `case` statement. The
expression form appears after `:=`, `return`, or `discard`, as an argument, as
an operand, or inside a condition. The formatter puts each arm on its own line,
indented one level, and `end case` at the indentation of the enclosing
statement.

## See also

- [Pattern matching](../pattern-matching/README.md)
- [Error handling](../error-handling/README.md)

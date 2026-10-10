# AP21: Decision expressions

Status: complete (AP21.1, AP21.2). Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Every accepted decision expression has an unambiguous result type and complete
value-producing paths.

## Decisions

- An `if` expression has the form `if C then A elsif D then B else E end if`,
  with the named closer like every other block form. `else` is required; each
  branch is a single expression.
- A `case` expression has the form `case Value of when Label: Expression; ...
  end case`. Each arm is a single expression ended by `;`; open domains may
  have an `else` arm. AP03 coverage rules apply.
- All branches must have a compatible type. There is no implicit return from a
  final statement and no statement list inside an expression branch.

```pascal
const Noun: string := if Count = 1 then 'item' else 'items' end if;

const Grade: string :=
  if Points >= 90 then 'A'
  elsif Points >= 50 then 'B'
  else 'C'
  end if;

return case Shape of
  when Shape.Circle(const R): Pi * R * R;
  when Shape.Rectangle(const W, const H): W * H;
end case;
```

### Branch types, position, conditions, and constants (AP21.1)

- All branches have one type, checked like an assignment. There is no numeric
  widening: `if C then 1 else 2.5 end if` is an error whose hint names `1.0`
  or `IntToReal`. Context-typed branches such as `None` and `[]` take the type
  of the other branches. With an expected type (a declared binding, parameter,
  field, or return type), every branch is checked against it.
- `if` (and `case`) at the start of a statement is always the statement form.
  The expression form appears only in expression positions: after `:=`,
  `return`, or `discard`, as an argument, an operand, or a condition. Because
  `end if` closes it, it is a primary operand and needs no parentheses, for
  example `1 + if C then 2 else 3 end if`.
- Conditions may use `is` pattern tests with the same rules as the `if`
  statement; bindings are visible only in the branch they guard.
- An `if` expression is a compile-time constant when its condition and every
  branch are compile-time constants (AP16 rules); otherwise it is computed.

AP21.2 applies the same branch-type, position, and constant rules to `case`
expressions.

## Open decisions

None.

## Dependencies

- AP03 (closed-enum coverage for `case` expressions).
- AP07 (boolean conditions and precedence).
- AP13 (`end if`, `elsif`, `when` arms).

## Order

The `if` expression comes first (AP21.1), then the `case` expression (AP21.2).

## Work packages

- [x] [AP21.1: if expressions](01-if-expressions.md)
- [x] [AP21.2: case expressions](02-case-expressions.md)

## Acceptance

Every accepted decision expression has an unambiguous result type and complete
value-producing paths.

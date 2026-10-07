# AP21: Decision expressions

Status: agreed direction. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
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

## Open decisions

Before AP21.1, specify branch-type compatibility, including numeric widening
and expected-type propagation. Also specify how an expression `if` is
distinguished from a statement `if` at the start of a statement. These details
were already required by AP21.1; the agreed forms above do not settle them.

## Dependencies

- AP03 (closed-enum coverage for `case` expressions).
- AP07 (boolean conditions and precedence).
- AP13 (`end if`, `elsif`, `when` arms).

## Order

The `if` expression comes first (AP21.1), then the `case` expression (AP21.2).

## Work packages

- [ ] [AP21.1: if expressions](01-if-expressions.md)
- [ ] [AP21.2: case expressions](02-case-expressions.md)

## Acceptance

Every accepted decision expression has an unambiguous result type and complete
value-producing paths.

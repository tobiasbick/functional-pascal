# AP13: Explicit block boundaries

Status: agreed direction (Q08, Q09). Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

The statement-ending and block-closing rules, including the complete block
table, are agreed (Q08, Q09). Task-scope semantics remain subject to the
decisions in AP26.

## Goal

Every block kind has one canonical form; every statement and declaration ends
with `;` except the program's final `end.`; expressions have no terminating `;`
of their own; branch ownership is structural, not inferred; and all repository
sources, docs, and tests use the new syntax.

## Decisions

### Reserved keyword names

The JSON null variant is `JsonValue.NullValue`. This confirmed API spelling
allows `null` to be reserved as a keyword while JSON text remains unchanged.

### Statement endings and closers

- Every statement and declaration ends with `;`, including the last one before
  a block closer, `else`, or `until`. The only exception is the final `end.` of
  a program's main block.
- Named block constructs use a named ending, and the `;` after it
  terminates the statement or declaration it closes: `end if;`, `end for;`,
  `end while;`, `end case;`, `end function;`, `end procedure;`, `end record;`,
  `end enum;`. A `repeat` loop keeps `repeat ... until Condition;`.
- `elsif` and `else` stand inside the `if`, after the terminated last
  statement of the previous branch, and share one `end if;`. A branch is a
  statement list and needs no `begin`/`end`, so an `else` never binds to an
  open inner statement. `else` followed by `if` always starts a nested `if`
  with its own `end if;`.
- Routine bodies keep `begin`, closed by the routine's named ending
  (`end function;`, `end procedure;`). The program's main block stays
  `begin ... end.`. A plain `begin ... end;` remains as a compound statement
  with its own scope for local declarations (Q08). Those declarations are not
  visible outside the block.
- A compound statement may appear in a branch or loop body. Its `end;` closes
  only that scope; the enclosing control structure still needs its named
  closer, such as `end if;` or `end while;`.
- Expressions take no terminating `;` of their own. Anonymous routines,
  record updates, and `if`/`case` expressions close with their named ending;
  any following `;` terminates the enclosing statement or declaration, as in
  `return case Value of ... end case;` or
  `const Moved: Point := P with X := 1; end with;`.
- An anonymous routine passed as an argument ends before the call's closing
  parenthesis; no `;` separates its ending from that parenthesis. Its body
  statements still require semicolons.
- Existing two-space indentation stays; statements inside a branch, loop, or
  `case` arm are indented one level below their owning clause.

```pascal
function Classify(Points: integer): Grade;
begin
  if Points >= 90 then
    Bonus := Bonus + 1;
    return Grade.Excellent;
  elsif Points >= 50 then
    return Grade.Good;
  else
    return Grade.Failed(Points);
  end if;
end function;
```

### Block table (Q09)

| Construct | Form |
|-----------|------|
| Program | `program Name;` ... `begin` ... `end.` |
| Unit | `unit Name;` ... `end unit;` |
| Function | `function F(...): T;` ... `begin` ... `end function;` |
| Procedure | `procedure P(...);` ... `begin` ... `end procedure;` |
| Anonymous function | `function(...): T begin ... end function` (expression) |
| Plain scoping block (Q08) | `begin ... end;` with block-local declarations |
| Record / enum type | `record ... end record;`, `enum ... end enum;` |
| `if` | `if C then ... elsif D then ... else ... end if;` |
| `case` | `case V of when L: ... else ... end case;` |
| `for` | `for I: integer := A to B do ... end for;`, `for X: T in Items do ... end for;` |
| `while` | `while C do ... end while;` |
| `repeat` | `repeat ... until C;` |
| Record update | `P with X := 1; end with` (expression) |
| `if`/`case` expression | `if C then A else B end if`, `case V of when L: E; end case` |
| Task scope (AP26) | `scope ... end scope;` |

Methods and nested routines follow the function/procedure rules. Anonymous
procedures use the corresponding `procedure ... end procedure` expression form.
Record literals are removed by AP10. `if`/`case` expressions are implemented by
AP21 and task scopes by AP26.

The following draft passes an anonymous function to a routine named `Apply`.
The final `;` terminates the call statement, not the anonymous function:

```pascal
Apply(
  function(Value: integer): integer
  begin
    return Value * 2;
  end function
);
```

### Details

- **`elsif` continues an `if` chain.** It replaces the earlier `else if`
  continuation, which made `else` followed by `if` ambiguous. When a missing
  `end if;` follows `else if`, the diagnostic suggests `elsif`.
- **Case arms start with `when`.** Each arm is `when Labels [if Guard]:`
  followed by a statement list; the next `when`, the catch-all, or `end case`
  ends the arm. No lookahead is needed to find an arm boundary.
- **Empty branches contain `null;`.** A `case` arm, `if` branch, or loop body
  with no action is written as the statement `null;`; an empty statement list
  is an error. `null` becomes a reserved keyword.
- **The catch-all arm is `else`.** It follows the last `when` arm, holds a
  statement list, and is ended by `end case`. Under AP03 it is valid only for
  open domains such as `integer` or `string`, not for closed enums.
- **Record updates close with `end with`:** `P with X := 10; end with`.
- **Units close with `end unit;`** after their last declaration. Units still
  have no main block; the program keeps `begin ... end.`.

```pascal
case Shape of
  when Shape.Circle(const R):
    Area := Pi * R * R;
    Log('circle');
  when Shape.Point:
    Area := 0.0;
end case;

case Command of
  when 'help':
    ShowHelp();
  when 'quit', 'exit':
    Running := false;
  else
    Log('unknown: ' + Command);
end case;

const Moved: Point := P with
  X := 10;
end with;
```

## Open decisions

There are no open decisions for AP13.1. Task-scope decisions belong to AP26.

## Common requirements for every work package

- Parser diagnostics show the expected closer or terminator, for example
  "expected `end if;`, found `end function;`", and recover at the statement
  boundary.
- The formatter emits every `;` and named closer, keeps comments attached
  across closers, indents bodies one level, and stays idempotent; CLI and
  editor formatting are identical.
- Each work package migrates every repository consumer (see the
  [migration rules](../development-process.md#migration-rules)) and updates
  `docs/specs/grammar.ebnf`, `docs/pascal/tools/fmt-style.md`, the affected
  `docs/pascal/` pages, editor snippets and highlighting, CLI templates, and the
  FPAS authoring skill for its constructs.
- Tests: positive parser and formatter cases for each affected block-table
  row; negative cases for missing `;` and mismatched closers; parse, format,
  parse equality and identical second formatting; unchanged FPAS suite results.
- No legacy parser mode ships; conversion tools stay on the branch.

## Dependencies

- AP01 (reference style), AP02 (diagnostic codes).

AP17, AP20, AP21, AP23, and AP26 depend on this package; AP28 is transferred
after it.

## Order

AP13.1 reserves the new keywords. AP13.2 switches to statement terminators;
the closer work packages build on it. AP13.3 (declarations) and AP13.4
(conditionals and loops) are independent of each other. AP13.5 (case arms)
reuses the statement-list branches of AP13.4. AP13.6 (expression closers)
follows AP13.3 so anonymous routines match named routines.

## Work packages

- [ ] [AP13.1: Reserve elsif, when, and null](01-reserve-block-keywords.md)
- [ ] [AP13.2: Statement and declaration terminators](02-statement-terminators.md)
- [ ] [AP13.3: Named closers for declarations](03-declaration-closers.md)
- [ ] [AP13.4: Conditional, loop, and scoping blocks](04-conditional-and-loop-blocks.md)
- [ ] [AP13.5: Case arms](05-case-arms.md)
- [ ] [AP13.6: Expression closers](06-expression-closers.md)

## Acceptance

Every block kind has one canonical form, including retained plain scoping
blocks, the program's `end.`, and `repeat ... until`; every statement and
declaration ends with `;` except the program's final `end.`; expressions have
no terminating `;` of their own; branch ownership is structural, not inferred;
and all repository sources, docs, and tests use the new syntax.

## Reference

The reference branch `codex/syntax-changes` delivered this syntax in one
change with a focused parser module `fpas-parser/src/parser/blocks.rs`,
declaration-list emission in the formatter, and an end-to-end conversion
regression `crates/fpas-cli/src/main_tests/fmt/source_conversion.rs`. Reserving
`null` required renaming the `JsonValue.Null` variant (the reference chose
`NullValue`). A program-entry name collision (a program and a routine sharing a
name) surfaced during that work; recheck it on `main`.

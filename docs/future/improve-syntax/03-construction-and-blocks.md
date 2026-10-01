# Construction and blocks

See the [steering document](README.md) for dependencies and approval gates.
Examples describe proposed forms, not runnable current-language programs.

## AP09: Named arguments

### Decided rules

- Named arguments use `Name := Value`, for example
  `CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false)`.
- A call is fully positional or fully named. Names are the public parameter
  names; there are no separate internal/external labels, so renaming a
  parameter changes the API.
- Parameters have no default values; every parameter is passed. Optional
  settings use an options record with field defaults (AP10).
- Named arguments apply to declared routines, methods, and enum variant
  constructors. Function values (closures, variables or parameters of a
  function type) are called positionally only, because function-type
  compatibility ignores parameter names.
- Arguments are evaluated in written left-to-right order, even when named
  arguments reorder parameters.

```pascal
CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false);
Start(Host := 'localhost', Options := ServerOptions(Port := 9000));

var F: function(Value: integer): integer := Double;
F(3);            // valid
F(Value := 3);   // error: named arguments need a declared routine
```

### Tasks

- [ ] Diagnose unknown, duplicate, and missing names, mixed positional/named
  calls, and named calls on function values; list the expected parameter names.
- [ ] Test same-typed argument roles, reordered evaluation, and invalid labels.

Acceptance: named calls have explicit role mapping and predictable evaluation;
invalid mappings receive concrete diagnostics.

## AP10: Typed record construction

### Decided rules

- Records are constructed as `TypeName(Field := Value, ...)`. This replaces the
  contextually typed `record ... end` literal; one construction form remains.
- Record construction is named only. `Point(10, 20)` is an error that shows the
  named form.
- Omitted fields take their declared default; fields without a default are
  required.
- Enum variants with data are constructor calls and follow AP09: fully
  positional or fully named. Patterns stay positional.
- A record with private fields can still be constructed only inside its
  declaring unit; importers use public factory functions.

```pascal
const P: Point := Point(X := 10, Y := 20);
const C: Config := Config(Port := 9000);   // other fields: defaults
const Q: Point := Point(10, 20);            // error: use Point(X := 10, Y := 20)

const A: Shape := Shape.Circle(2.0);
const B: Shape := Shape.Rectangle(Width := 3.0, Height := 4.0);
```

### Tasks

- [ ] Verify whether a type and a routine may share a name today; specify that a
  record type name used as a call target always means construction.
- [ ] Diagnose `record ... end` literals with the typed replacement.
- [ ] Detect swapped types, duplicate/unknown/missing fields.
- [ ] Migrate existing record literals (about 590 sites in `.fpas` sources at
  planning time) while preserving their data; the migration must take the
  record type from the literal's expected type.

Acceptance: one canonical structural-construction rule exposes the type and
field mapping without guessing between literals and factories.

## AP11: Individual declarations

### Decided rules (Q07)

- All types declared in the same unit or program are visible to each other
  regardless of declaration order, including mutually recursive types.
- No explicit forward type declaration is required.
- Constants and variables remain subject to declaration order.

### Tasks

- [ ] Require a separate `type`, `const`, or `var` keyword for each declaration;
  remove groups that inherit a declaration category from a preceding entry.
- [ ] Apply the rule only where declarations are already permitted. Do not
  implicitly introduce local type declarations.
- [ ] Resolve type references across all type declarations in the same unit or
  program, preserving scope and visibility.
- [ ] Test forward type references and mutually recursive types with separate
  `type` keywords; verify that constants and variables still obey declaration
  order.
- [ ] Split existing groups while preserving scope and visibility.

Acceptance: a local edit cannot accidentally inherit the wrong declaration
category, and migration preserves exported names, scopes, and recursive types.

## AP12: Callable expressions

- [ ] Allow any correctly typed function expression as a call target, including
  returned functions such as `MakeAdder(3)(5)`.
- [ ] Keep call parentheses explicit. Do not add automatic partial application,
  currying, or a second lambda shorthand; retain existing anonymous functions.
- [ ] Specify argument order and test returned functions, selected function
  values, function fields, and non-callable values.

Acceptance: all function-valued call targets follow the same rules; non-callable
targets receive an actionable diagnostic.

## AP13: Explicit block boundaries

Agreed direction. The statement-ending and block-closing rules, including the
complete block table, are agreed (Q08, Q09). Task-scope semantics remain subject
to the decisions in AP26.

### Agreed rules

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

### Decided block table (Q09)

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
Record literals are removed by AP10.

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

### Decided details

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

### Tasks

- [ ] Implement the agreed block table, covering methods, anonymous routines,
  nested declarations, and comments at closing boundaries as well as the
  explicitly listed forms.
- [ ] Parser: require `;` after every statement and declaration and the named
  closer for each named block, while retaining `end;` for plain scoping blocks.
  Reject missing final `;` and plain `end;` used to close a control structure
  with diagnostics that show the expected closer, for example
  "expected `end if;`, found `end function;`", and recover at the statement
  boundary.
- [ ] Reserve `elsif`, `when`, and `null` as keywords; diagnose their use as identifiers
  with a rename hint.
- [ ] Formatter: emit every `;` and named closer, indent branch and loop bodies
  one level, keep comments attached across closers, and make CLI and editor
  formatting identical and idempotent.
- [ ] Migration: convert every repository consumer — `.fpas` sources under
  `lib/`, `examples/`, `tests/`, `apps/`, Rust-embedded fixtures, generated-source
  templates, and documentation examples. Use a temporary tool that reads the old
  syntax and emits the new one; remove it before completion and ship no
  permanent legacy mode.
- [ ] Update `docs/specs/grammar.ebnf`, `docs/pascal/tools/fmt-style.md`, the
  affected `docs/pascal/` pages, and the FPAS authoring guidance that teaches the
  old separator rules.
- [ ] Tests: positive parser and formatter cases for every row of the table;
  negative cases for missing `;`, mismatched closers, and a stray `;` before
  `else`; nested `if`/`case`, loops inside branches, and empty bodies; parse →
  format → parse equality and a second formatting pass with identical output;
  `fpas test tests/suite.fpasprj` with unchanged runtime results.
- [ ] Test nested plain blocks, visibility of local declarations within their
  block, rejection of access outside it, and plain blocks inside branches and
  loops with separate closing tokens for the enclosing control structure.
- [ ] Test expression endings in declarations, returns, and call arguments;
  reject an extra terminating `;` between an anonymous routine expression and
  the call's closing parenthesis.

Acceptance: every block kind has one canonical form, including retained plain
scoping blocks, the program's `end.`, and `repeat ... until`;
every statement and declaration ends with `;` except the program's final
`end.`; expressions have no terminating `;` of their own; branch ownership is structural,
not inferred; and all repository sources, docs, and tests use the new syntax.

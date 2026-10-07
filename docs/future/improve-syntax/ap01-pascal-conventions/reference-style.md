# Pascal-oriented reference style

This is the reference for [AP01](README.md). The ten reference examples
distinguish implemented forms from changes owned by open packages. Examples
that combine planned forms remain uncompiled drafts. The program, unit,
conditional, loop, and callback examples use only implemented forms. A draft
does not approve an open decision or claim that the current compiler accepts
it. AP13 statement endings, block closers,
`elsif`, `when`, and `null;`, AP11 individual declarations, and AP16
computed `const` and writable `var` bindings are implemented.

## Writing rules

- Use two spaces per indentation level, lowercase keywords and primitive type
  names, and descriptive PascalCase identifiers. Names stay case-insensitive.
- Use `:=` for assignment, initialization, named arguments, and named record
  fields; use `=` for equality and type definitions.
- Parenthesize calls. Separate declared parameters with `;`, each with its
  own type annotation; separate call arguments with `,`
  ([AP08, Q06](../ap08-comma-separated-parameter-lists/README.md)).
- Repeat the declaration keyword for each declaration
  ([AP11](../ap11-individual-declarations/README.md)). Prefer `const`; use `var`
  for reassignment ([AP16](../ap16-immutable-and-mutable-bindings/README.md)).
  Keep explicit binding types in these examples; local inference is limited
  and has open details in [AP22](../ap22-limited-local-inference/README.md).
- End statements and declarations with `;`, including the last statement in
  a body. Keep the program's final `end.`. Use the owning construct's named
  closer, while a plain scoping block keeps `end;`
  ([AP13, Q08 and Q09](../ap13-explicit-block-boundaries/README.md)).
- An expression has no terminating `;` of its own. The enclosing statement or
  declaration supplies it; an anonymous routine used as an argument closes
  before the argument separator or closing parenthesis (AP13.6).
- Consume function results. A procedure call may stand alone; deliberate
  discard of a value uses `discard Expression;`
  ([AP04](../ap04-discarded-function-values/README.md)).
- Type applications use `of`: one argument without parentheses, multiple
  arguments in parentheses. Keep `dict of K to V`. Generic routines retain
  angle brackets only when declaring type parameters; their calls infer the
  type arguments ([AP24](../ap24-generic-data-structures/README.md)).
- Declarations and record members remain private by default; `public` exports
  them. Do not add a visibility keyword for the default.

## 1. Program, bindings, and assignment

Current forms: the program heading and final `end.`, explicit types,
initializers, `:=` assignment, immutable `const` and writable `var` (AP16),
and the required semicolon before `end.` (AP13.2)
([variables](../../../pascal/language/basics/variables.md)).
The example uses only implemented forms.

```pascal
program Counting;

const Step: integer := 2;
var Count: integer := 0;

begin
  Count := Count + Step;
end.
```

## 2. Unit and public routine

Current forms: units, explicit `public` visibility, routine headings,
semicolon-separated parameters, `begin`, `return`, the final body semicolon,
`end function;`, and `end unit;` (AP13.2 and AP13.3;
[grammar](../../../specs/grammar.ebnf), `unit` and `function_heading`).
The example uses only implemented forms.

```pascal
unit Arithmetic;

public function Add(Left: integer; Right: integer): integer;
begin
  return Left + Right;
end function;

end unit;
```

## 3. Records, enums, and construction

Current forms: record fields and defaults, enum payload declarations, and
positional variant construction, named type closers, and the main-body
terminator (AP13;
[records](../../../pascal/language/types/records.md),
[enums](../../../pascal/language/types/enums.md)). Payload declarations also
separate individually typed parameters with `;`.

One keyword per declaration (AP11) and computed `const` bindings (AP16) are
implemented. Draft forms: named record construction (AP10) and named variant
arguments (AP09.2).

```pascal
program Shapes;

type Point = record
  X: integer;
  Y: integer := 0;
end record;

type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
end enum;

const Origin: Point := Point(X := 0);
const Round: Shape := Shape.Circle(2.0);
const Box: Shape := Shape.Rectangle(Width := 3.0, Height := 4.0);

begin
  null;
end.
```

The `null;` no-op statement is implemented by AP13.4. `Point` fields are
accessible within this program; construction across units follows AP10's
visibility rule.

## 4. Ordinary calls and visible caller mutation

Current forms: explicit parameter and result types, positional calls, and
assignment, statement terminators, and routine closers (AP13;
[parameters](../../../pascal/language/functions/parameters.md)).

Writable `var` bindings are implemented by AP16. Draft forms: `var` parameters
and arguments (AP17.1), and the fully named call (AP09 and AP17.2).

```pascal
program Calls;

function Add(Left: integer; Right: integer): integer;
begin
  return Left + Right;
end function;

procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

var Counter: integer := 0;

begin
  Counter := Add(1, 2);
  Increase(var Counter);
  Increase(Value := var Counter);
end.
```

The `Add` result is consumed by assignment. Both calls to `Increase` are
procedure statements; neither silently discards a function result (AP04).
AP17's function-type modes and writes retained on failure are agreed; its
reference-parameter implementation is still pending.

## 5. Conditional branches and a plain scoping block

Current forms: comparisons, assignment, explicit local declarations,
statement-list branches, `elsif`, `null;`, `end if;`, and the retained plain
block's local scope (AP13.1, AP13.2, and AP13.4;
[grammar](../../../specs/grammar.ebnf), `if_stmt` and `block`).

Binding keywords follow the implemented AP16 rules. The `end;` closes only
the plain block; `end if;` closes the conditional. The example uses only
implemented forms.

```pascal
program Branches;

const Score: integer := 75;
var Grade: integer := 0;

begin
  if Score >= 90 then
    Grade := 3;
  elsif Score >= 50 then
    begin
      const PassingGrade: integer := 2;
      Grade := PassingGrade;
    end;
  else
    null;
  end if;
end.
```

## 6. Loops

Current forms: counted `for` headings, `while` conditions, `repeat ... until`,
statement terminators, statement-list bodies, and `end for;` / `end while;`
(AP13.2 and AP13.4; [grammar](../../../specs/grammar.ebnf), `for_stmt`,
`while_stmt`, and `repeat_stmt`).

Writable `var` and immutable per-iteration `for` variables are implemented
under AP16.3; the loop variable has no binding keyword. `repeat` keeps `until`.
The example uses only implemented forms.

```pascal
program Loops;

var Total: integer := 0;

begin
  for I: integer := 1 to 3 do
    Total := Total + I;
  end for;

  while Total > 0 do
    Total := Total - 1;
  end while;

  repeat
    Total := Total + 1;
  until Total = 2;
end.
```

## 7. Case arms and explicit pattern bindings

Current forms: enum payloads, qualified variants, `case ... of`, declaration
closers and statement terminators, `when` arms, and `null;` (AP13;
[enums](../../../pascal/language/types/enums.md)).

Computed `const` and writable `var` are implemented by AP16. Draft forms:
`const` in payload patterns (AP20.1), and mandatory explicit coverage without
`else` for the closed enum (AP03).

```pascal
program Matching;

type Choice = enum
  Present(Value: integer);
  Missing;
end enum;

const Item: Choice := Choice.Present(3);
var Total: integer := 0;

begin
  case Item of
    when Choice.Present(const Value):
      Total := Value;
    when Choice.Missing:
      null;
  end case;
end.
```

## 8. Generic types and generic routines

Current forms: `array of T`, `Option of T`, `dict of K to V`, and generic
routine declarations with inferred call-site type arguments, statement
terminators, and named closers (AP13;
[grammar](../../../specs/grammar.ebnf), `type_expr`;
[generic routines](../../../pascal/language/functions/generic-routines.md)).

Draft forms: parenthesized multi-argument type applications and user-defined
generic records/enums (AP24), and named record construction (AP10). Computed
`const` bindings are implemented by AP16.

```pascal
program Generics;

type Pair of (K, V) = record
  Key: K;
  Value: V;
end record;

type Lookup of T = enum
  Found(Value: T);
  Missing;
end enum;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

const Entry: Pair of (string, integer) := Pair(Key := 'count', Value := 3);
const Item: Lookup of string := Lookup.Found('x');
const Count: integer := Identity(3);

begin
  null;
end.
```

`Identity<T>` declares a routine type parameter; `Identity(3)` infers it.
It is not an angle-bracket type application. Built-in examples keep the same
rules: `array of string`, `Option of string`, `Result of (integer, string)`,
and `dict of string to integer`. Only the multi-argument `Result` spelling
changes from today's `Result of integer, string` in AP24.1.

## 9. Anonymous routine as an argument

Current forms: function types, annotated anonymous routines, passing them as
arguments, positional calls through function parameters, named routine and
expression closers, body terminators, and `null;` (AP13.2 through AP13.6;
[closures](../../../pascal/language/functions/closures.md)).

Computed `const` is implemented by AP16. The example uses only implemented
forms.

```pascal
program Callbacks;

function Apply(Operation: function(Value: integer): integer;
  Value: integer): integer;
begin
  return Operation(Value);
end function;

const Doubled: integer := Apply(
  function(Value: integer): integer
  begin
    return Value * 2;
  end function,
  3
);

begin
  null;
end.
```

The anonymous routine has no `;` before `,`. The final `);` terminates the
binding declaration, not the routine expression. The function result is
consumed, and the function-valued parameter is called positionally (AP09).

## 10. Subranges and the recorded distinct-type exception

Current forms: type aliases, primitive types, integer literals, the main-body
terminator, and `null;` (AP13; [grammar](../../../specs/grammar.ebnf),
`type_def` and `integer_literal`).

Draft forms: integer subrange declarations and checked conversion (AP18, Q10
and Q11), and `distinct` declarations and explicit construction (AP19, Q12
and Q13). Computed `const` is implemented by AP16.

```pascal
program DomainTypes;

type Percent = 0..100;
type UserId = distinct integer;

const Level: Percent := Percent(50);
const Id: UserId := UserId(42);

begin
  null;
end.
```

AP19 explicitly selects `distinct` instead of Delphi's repeated `type` form.
This example does not choose its unresolved constraint, dictionary-key, or
case-label rules. See the [spelling review](spelling-review.md) for decisions
that remain with the other packages.

# Pascal-oriented reference style

This is the reference for [AP01](README.md). All ten examples are uncompiled
drafts of the planned language. Each example distinguishes forms supported by
the current handbook from changes owned by another package. A draft does not
approve an open decision or claim that the current compiler accepts it.

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
initializers, and `:=` assignment
([variables](../../../pascal/language/basics/variables.md)).

Draft forms: mutable `var` and the preferred immutable `const` form (AP16),
and the required semicolon before `end.` (AP13.2).

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
semicolon-separated parameters, `begin`, and `return`
([grammar](../../../specs/grammar.ebnf), `unit` and `function_heading`).

Draft forms: the required final body semicolon (AP13.2), `end function;`,
and `end unit;` (AP13.3).

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
positional variant construction
([records](../../../pascal/language/types/records.md),
[enums](../../../pascal/language/types/enums.md)). Payload declarations also
separate individually typed parameters with `;`.

Draft forms: enforcing one keyword per declaration (AP11), named type closers
and the main-body terminator (AP13), computed `const` bindings (AP16), named
record construction (AP10), and named variant arguments (AP09.2).

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

The `null;` no-op statement belongs to AP13.4. `Point` fields are accessible
within this program; construction across units follows AP10's visibility rule.

## 4. Ordinary calls and visible caller mutation

Current forms: explicit parameter and result types, positional calls, and
assignment ([parameters](../../../pascal/language/functions/parameters.md)).

Draft forms: required statement terminators and routine closers (AP13), mutable
`var` bindings (AP16), `var` parameters and arguments (AP17.1), and the fully
named call (AP09 and AP17.2).

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
This example does not decide AP17's open rules for function types or failure.

## 5. Conditional branches and a plain scoping block

Current forms: comparisons, assignment, and explicit local declarations
([grammar](../../../specs/grammar.ebnf), `if_stmt` and `block`).

Draft forms: statement-list branches, `elsif`, `null;`, `end if;`, and the
retained plain block's local scope (AP13.1, AP13.2, and AP13.4); binding keywords
follow AP16. The `end;` closes only the plain block; `end if;` closes the
conditional.

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

Current forms: counted `for` headings, `while` conditions, and
`repeat ... until` ([grammar](../../../specs/grammar.ebnf), `for_stmt`,
`while_stmt`, and `repeat_stmt`).

Draft forms: mutable `var` (AP16), statement terminators (AP13.2), statement-list
bodies and `end for;` / `end while;` (AP13.4). The `for` variable is immutable
per iteration under AP16.3; it has no binding keyword. `repeat` keeps `until`.

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

Current forms: enum payloads, qualified variants, and `case ... of`
([enums](../../../pascal/language/types/enums.md)).

Draft forms: declaration closers and statement terminators (AP13), computed
`const` and mutable `var` (AP16), `when` arms and `null;` (AP13.5), `const` in
payload patterns (AP20.1), and explicit coverage without `else` for the closed
enum (AP03).

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
routine declarations with inferred call-site type arguments
([grammar](../../../specs/grammar.ebnf), `type_expr`;
[generic routines](../../../pascal/language/functions/generic-routines.md)).

Draft forms: parenthesized multi-argument type applications and user-defined
generic records/enums (AP24), named record construction (AP10), computed
`const` (AP16), and statement terminators and named closers (AP13).

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
arguments, and positional calls through function parameters
([closures](../../../pascal/language/functions/closures.md)).

Draft forms: computed `const` (AP16), named routine and expression closers,
required body terminators, and `null;` (AP13.2, AP13.3, AP13.4, and AP13.6).

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

Current forms: type aliases, primitive types, and integer literals
([grammar](../../../specs/grammar.ebnf), `type_def` and `integer_literal`).

Draft forms: integer subrange declarations and checked conversion (AP18, Q10
and Q11), `distinct` declarations and explicit construction (AP19, Q12 and
Q13), computed `const` (AP16), and the main-body terminator and `null;` (AP13).

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

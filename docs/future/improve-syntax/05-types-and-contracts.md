# Types and contracts

See the [steering document](README.md) for dependencies and approval gates.
All example declarations are draft syntax.

## AP18: Subrange types

### Decided base types (Q10)

- Initially support integer subranges only, such as `type Percent = 0..100;`.
- The base type follows from the bounds; no explicit base type may be written.
- Enum subranges are deferred until a concrete use case justifies them.

### Decided conversions and violations (Q11)

- A subrange value widens implicitly to `integer`; arithmetic results are
  `integer`.
- Conversion from `integer` to a subrange requires an explicit checked
  conversion, such as `Percent(X)`, including in assignments and returns.
- An out-of-range constant is a compile-time error. An out-of-range dynamic
  value causes a runtime panic that reports the value and the allowed range.
- Membership testing with `X in Percent` checks the range before conversion.
  Conversion does not return `Option` or `Result`.

```pascal
if Input in Percent then
  Show(Percent(Input));
else
  Reject(Input);
end if;
```

### Tasks

- [ ] Specify static integer bounds with the Pascal subrange form, for example
  `type Percent = 0..100;` and `type RetryCount = 0..5;`, using the decided
  implicit base type.
- [ ] Test accepted integer bounds and rejection of non-integer bounds and
  explicit base-type annotations.
- [ ] Implement explicit checked conversion, compile-time diagnostics for
  invalid constants, and runtime panics for invalid dynamic values.
- [ ] Implement implicit widening and integer arithmetic results; require
  checked conversion when placing those results back into a subrange.
  Distinguish range restrictions from AP19's distinct type identity.
- [ ] Implement range membership checks without requiring arbitrary proof
  solving or automatic validation of external data.
- [ ] Test both endpoints, values immediately outside the range, invalid
  constants, dynamic conversion, panic details, membership, implicit widening,
  arithmetic result types, and rejection of implicit narrowing in assignments,
  arguments, and returns.

Acceptance: unchecked input cannot bypass the range restriction; invalid
constants produce compile-time diagnostics and invalid dynamic conversions
panic with the value and range; widening and arithmetic produce integers.

## AP19: Distinct domain types

### Decided spelling (Q12)

- Use `type UserId = distinct integer;` for a distinct domain type.
- Introduce `distinct` as a keyword. This is an explicitly agreed exception to
  the Pascal/Delphi spelling preference: it makes the distinction from the
  alias `type UserId = integer;` visible without a repeated `type` keyword.

### Decided operations (Q13)

- Construction and unwrapping require explicit conversions: `UserId(42)` and
  `integer(Id)`.
- Equality and ordering are inherited from the underlying type for operands
  of the same distinct type. Different domain types are not interchangeable.
- Arithmetic is not inherited; `Id + 1` is invalid. Use records or explicit
  functions for quantities that need arithmetic.
- Do not add a separate declaration form for hiding the representation.
  Records with non-public fields provide that encapsulation.

### Tasks

- [ ] Implement the `distinct` declaration form and separate type identity
  from aliases; diagnose the unselected `type UserId = type integer;` form
  with the canonical spelling.
- [ ] Test distinct declarations, unchanged aliases, and diagnostics for the
  unselected declaration form and use of `distinct` as an identifier.
- [ ] Implement explicit construction and unwrapping, same-type equality and
  ordering, and rejection of inherited arithmetic.
- [ ] Test explicit conversions, valid same-type comparisons, rejected implicit
  conversions and mixed-domain comparisons, swapped domain arguments, and
  arithmetic rejection. Distinctness alone does not validate a URL, path, or
  duration.

Acceptance: domain identities cannot be interchanged accidentally; construction
and unwrapping are explicit, same-type comparisons work, and arithmetic is not
inherited. Representation hiding uses existing record visibility.

## AP20: Nested patterns and explicit bindings

Current enum patterns bind plain identifiers only, and scalar labels never bind,
so binding versus comparison is not ambiguous today. It becomes ambiguous once
patterns nest and may contain literals or constants; the explicit binding form
is introduced together with nesting.

### Decided rules

- `const Name` binds a field, for example `Some(const User)`; literals and
  named constants compare; `_` ignores a field. `_` never ignores a whole
  closed-enum variant.
- Patterns nest: `when Ok(Some(const User)):`.
- `Value is Pattern` tests one variant and binds its fields. It is valid as the
  condition of `if`, `elsif`, and `while`; the bindings are visible only in the
  branch or loop body. It may be followed by `and` conditions that use the
  bindings; it cannot appear under `or` or `not`. It is not a `case` and has no
  exhaustiveness check. `is` is not a free boolean value.

```pascal
case Response of
  when Ok(Some(const User)):
    Greet(User);
  when Ok(None):
    null;
  when Error(_):
    Retry();
end case;

if Msg is TuiMsg.Resize(const W, const H) and W > 0 then
  Relayout(W, H);
end if;

while Queue.Next() is Some(const Job) do
  Run(Job);
end while;

const Hit: boolean := X is Some(_);   // error: 'is' only in if, elsif, while
```

### Tasks

- [ ] Resolve constants and constructors explicitly; a nearby constant must not
  turn a binding into a comparison.
- [ ] Check typing, reachability, guards, duplicate arms, and exhaustiveness for
  nested patterns; retain explicit top-level variant handling.
- [ ] Reserve `is`; diagnose `is` outside its permitted positions.
- [ ] Migrate existing plain-identifier bindings to `const Name`.
- [ ] Test shadowing, constructor lookup, comparison, explicit bindings, `_`,
  `is` in each position, and binding scope.

Acceptance: nested patterns have stable resolution and concrete coverage
diagnostics without a catch-all loophole for closed enums; adding a surrounding
name cannot silently alter a pattern's meaning.

## AP21: Decision expressions

### Decided rules

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

### Tasks

- [ ] Implement the `if` expression first, then the `case` expression.
- [ ] Specify branch-type compatibility and how an expression `if`/`case` is
  distinguished from the statement form at the start of a statement.
- [ ] Diagnose a missing `else`, a missing `end if`/`end case`, and statements
  inside an expression branch.
- [ ] Test nested expressions, missing branches/values, and incompatible results.

Acceptance: every accepted decision expression has an unambiguous result type
and complete value-producing paths.

## AP22: Limited local inference

### Decided rules (Q14)

- Allow omitted type annotations on local bindings only for literals, typed
  construction, and explicit conversions. The type must be evident from the
  initializer itself.
- Ordinary function calls and other initializer forms require an explicit
  type annotation; do not introduce general local inference.
- Explicit local annotations remain valid. Public parameters, results, and
  fields remain fully annotated.
- AP27 may rely on this limited form after its implementation; it must not
  broaden inference implicitly. AP27 is retained as optional (Q21).

```pascal
const P := Point(X := 1, Y := 2);
const Limit := 100;
const Level := Percent(Input);
const Name: string := Text.Trim(Input);   // ordinary call: annotation required
```

### Tasks

- [ ] Implement local inference for the three permitted initializer forms.
- [ ] Keep public parameters, results, and fields fully annotated. Allow explicit
  local annotations and require them for ambiguous cases.
- [ ] Reject arbitrary guesses for empty collections or underconstrained constructors.
- [ ] Verify that changing a private body cannot change a public signature.
- [ ] Test inferred literals, typed construction, and conversions; accepted
  explicit annotations; rejection of unannotated ordinary calls and other
  initializer forms; and ambiguous or underconstrained initializers.

Acceptance: only the three permitted local initializer forms infer types;
ordinary calls and ambiguous cases require annotations, and public signatures
remain explicit and stable.

## AP23: Preconditions and postconditions

### Decided keywords (Q15)

- Use `requires` for preconditions and `ensures` for postconditions.
- Both become reserved words.
- Multiple clauses of the same kind are combined with logical `and`.

### Decided return-value naming (Q16)

- An `ensures` clause may name the return value explicitly, for example
  `ensures (Clamped) Clamped >= Lower and Clamped <= Upper;`.
- The name denotes the routine's return value and is visible only in that
  clause. It does not introduce a binding in the body or in other clauses.
- Do not use the routine name or `return` as a special return-value reference.
  `Result` remains reserved for the existing result type.

### Decided checking and violations (Q17)

- Contracts are always checked, including release builds. No build mode
  disables them.
- Check `requires` on routine entry and `ensures` on every return.
- A violation causes a panic identifying the routine, clause text, and
  parameter values; it is not a `Result` error.
- Initially provide no `old` values or entry-state snapshots.
- Contract expressions may use parameters, constants, operators, and built-in
  length/membership checks; postconditions may also use their named return
  value. Further function calls require a `pure` annotation once AP25 is
  implemented. AP23 does not depend on that later extension.

### Tasks

- [ ] Implement boolean routine contracts with the agreed `requires` and
  `ensures` keywords and conjunction of repeated clauses.
- [ ] Test repeated clauses, boolean typing, and diagnostics for using either
  reserved keyword as an identifier.
- [ ] Implement the clause-local return-value binding with the routine's
  return type; test valid references, type errors, and rejection of references
  from the body or other clauses.
- [ ] Enforce the agreed expression restrictions, including the named return
  value in postconditions; reject unsupported calls and `old` references.
- [ ] Implement mandatory entry/return checks and violation panics with the
  agreed diagnostic details; do not add automatic proofs.
- [ ] Test entry checks, every return path, repeated clauses, release-build
  checks, panic details, expression restrictions, and missing return values.
  Verify that violations do not become `Result` errors.

Draft contract fragment using the agreed keywords and return-value naming;
the body is omitted:

```pascal
function Clamp(Value: integer; Lower: integer; Upper: integer): integer;
requires Lower <= Upper;
ensures (Clamped) Clamped >= Lower and Clamped <= Upper;
```

Acceptance: contracts are typed and restricted to side-effect-free expressions;
checks remain active in release builds, violations panic with the agreed
details, and contracts cannot substitute for a missing return.

## AP24: Generic data structures

### Decided rules

- Every type application uses `of`. One type argument is written without
  parentheses; several are always parenthesized: `array of string`,
  `Option of User`, `Result of (User, LoadError)`,
  `Pair of (Lookup of string, integer)`. `dict of K to V` keeps its form.
- User-defined generic types declare their parameters with the same word:
  `type Lookup of T = enum`, `type Pair of (K: Comparable, V) = record`.
  Routines keep `function Max<T: Comparable>(...)`, because routine type
  arguments are always inferred and never written at a use site.
- Angle-bracket applications such as `Lookup<string>` are errors whose
  diagnostic shows the `of` form.

```pascal
type Lookup of T = enum
  Found(Value: T);
  Missing;
  Failed(Message: string);
end enum;

type Pair of (K: Comparable, V) = record
  Key: K;
  Value: V;
end record;

const L: Lookup of string := Lookup.Found('x');
const R: Result of (User, LoadError) := Load(Id);
```

### Tasks

- [ ] Specify and implement one user-defined generic record, then a payload enum.
- [ ] Migrate `Result of T, E` to `Result of (T, E)` (about 890 sites in `.fpas`
  sources at planning time), docs, and fixtures.
- [ ] Settle recursive types, constructor lookup, equality, and constructor
  type-argument inference: from constructor arguments, otherwise from the
  expected type; if neither determines a parameter (`Lookup.Missing` without
  context), report an error asking for an annotation.
- [ ] Start constraints from the existing routine constraints, not a broad
  typeclass model.
- [ ] Test multiple concrete type arguments, nesting, recursion, invalid types,
  and nested patterns.

Acceptance: generic records and enums work across concrete types with explainable
rules, including recursion and nested matching, rather than per-type exceptions,
and every type application uses the same `of` form.

# Implemented expressions and control flow

This extends the [direct compiler subset](first-program.md) using the existing
language syntax. Programs still compile directly to CIL and run through .NET;
no Fpas.Core runtime dependency is needed. The [Mandelbrot example](../../examples/mandelbrot/README.md)
builds and executes through its ordinary `.fpasproj`.

## Types, literals, and expressions

- All [primitive numbers](scalar-types.md), string, and bool are supported, including source
  parameters/results and referenced CLR methods.
- Decimal/hexadecimal integral literals, L/U/UL suffixes, real F/D/M suffixes,
  digit separators, true/false, and single-line strings are implemented.
  A directly negated literal can express a signed minimum. Floating literals must be
  finite; runtime floating operations retain IEEE infinity/NaN behavior.
- Unary +, -, not; binary +, -, *, /, div, mod; comparison =, <>, <, <=, >, >=;
  and Boolean/integer and, or, xor plus integer shl/shr follow the baseline
  precedence. Comparisons require parentheses when nested.
- Integral arithmetic and numeric casts check overflow by default.
  `unchecked(Expression)` enables wrapping integer arithmetic and unchecked
  conversion only within that expression. Integer division retains CLR errors.
  `div` truncates toward zero; `/` needs floating operands after promotion.
- C# numeric widening applies in expressions, bindings, arguments, and returns.
  `cast<int>` narrows a double with truncation toward zero; `cast<double>`
  widens an int explicitly. Decimal remains checked inside unchecked expressions.
  No Boolean/numeric or string/numeric conversions
  are inserted.
- Metadata overload selection prefers identity over numeric widening for each
  argument. A candidate must be at least as good for every argument and better
  for one; competing incomparable candidates are ambiguous.
- Boolean and/or short-circuit; xor and integer bitwise operators evaluate both
  operands. Shift counts use CLR masking. String + and equality use actual
  System.String methods, preserving ordinal content equality.
- if/elsif/else expressions require a common branch type, including int-to-double
  promotion, and evaluate only the selected branch. Function/argument operands
  evaluate left to right.

## Statements, scopes, and flow

Every const/var still requires an explicit type and initializer. A local var may
be assigned; const bindings, parameters, and counting-loop variables are read-only.
`discard Expression;` consumes an intentionally unused value.

`begin ... end;`, each if branch, and each loop body create lexical scope.
Locals may shadow outer locals in inner scopes; declarations in the same scope
cannot duplicate a name. Routine parameters cannot be redeclared in the routine's
outer body. Import aliases cannot be shadowed. Names declared inside a branch,
block, or loop are unavailable after it. Every declaration initializes its own
storage before that binding can be read.

if/elsif/else statements, while loops, and int counting loops with to/downto are
implemented. Counting bounds evaluate once in written order; endpoints are
inclusive, reversed ranges are empty, and the final iteration ends before the
counter could overflow at int.MinValue/int.MaxValue. A counter is scoped to its
loop body. break/continue affect the innermost loop; continue in a counting loop
still performs the guarded step.

Functions must return a compatible value on every normal path. A constant-true
while loop with no reachable break has no normal fallthrough. Statements after
unconditional return/break/continue or exhaustive terminating branches are
diagnosed at the unreachable statement. A procedure/program may return without
a value. Emission uses typed locals and branch labels with a shared return point.

## Verification and limits

The xUnit suite executes emitted programs to check precedence, signed arithmetic,
bitwise operations, widening/casts, bool/double metadata, floating NaN comparisons,
overflow/zero-division exceptions, short-circuit/branch effects, recursive calls,
scopes, nested loops, and extreme loop endpoints. Diagnostic tests cover invalid
types/assignments, missing annotations/returns, unreachable statements, and loop
exits outside loops. The mixed-project suite verifies exported bool/double methods
from C#, and the SDK builds the checked-in Mandelbrot program and checks all
1,920 output characters against an independent System.Numerics.Complex calculation.

[Global values](global-values.md) and [named nested routines](nested-routines.md)
are implemented. Repeat loops remain later work. [XML documentation](xml-documentation.md) is
implemented. [Multiple FPAS sources](multiple-sources.md)
[CLR objects, nullable types, and exceptions](clr-objects-and-resources.md), and
[arrays, collections, and enumeration](arrays-and-collections.md), and
[callable values, closures, and generic CLR calls](functions-and-closures.md),
[records](records.md), [unions/case matching](unions-and-matching.md), and
[generic source routines](generic-routines.md), [generic data types](generic-types.md),
and [type argument inference](type-argument-inference.md) are now supported.
[Async and portable PDBs](overloads-interfaces-debugging-and-async.md) are implemented.
Runtime arithmetic exceptions already
use ordinary CLR exception types; no additional exception layer is introduced.

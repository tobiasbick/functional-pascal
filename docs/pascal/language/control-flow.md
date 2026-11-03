# Control flow and patterns

## Conditions

```pascal
if Health <= 0 then
  Console.WriteLine('Defeated');
elsif Health < 5 then
  Console.WriteLine('In danger');
else
  Console.WriteLine('Ready');
end if;
```

Conditions have type `bool`. `elsif` is one keyword. Each statement, including
the one before `elsif` or `else`, ends with `;`. An `if` expression requires
`else` and one expression per branch:

```pascal
const Label: string := if Health > 0 then 'Alive' else 'Defeated' end if;
```

Branch results must have a common type through identity, expected type, or an
applicable implicit conversion. Incompatible results require explicit conversion.
Conditions and branches execute only as selected.

## Loops

```pascal
for Index: int := 0 to Values.Length - 1 do
  Console.WriteLine(Values[Index]);
end for;

for Item: string in Items do
  Console.WriteLine(Item);
end for;

while Ready do
  Work();
end while;

repeat
  Work();
until Done;
```

Repeat executes its body at least once, then evaluates a bool condition: false
repeats and true exits. Each iteration has a body-local scope; those declarations
are unavailable in the until condition and after the loop. The condition runs
after body cleanup. Continue reaches this condition after cleanup; break exits
without evaluating it. Return/raise and failed cleanup also skip the condition.

Counting loops have primitive integral (including char and native integers) or integral-subrange variables and inclusive
bounds. For a subrange, endpoints must already have that type or use explicit
checked construction. Increment/decrement stays within its bounds and stops
before stepping beyond the final value. Distinct scalars are not loop counters.
Both bounds evaluate once in written order. `downto` counts downward. A reversed range runs
zero times. The loop terminates before incrementing past its final value, even
at the integral type's minimum/maximum. The loop variable is a new read-only
binding scoped to the loop body; closures capture that iteration's value.

`for Item: T in Source` requires the explicit element type. Omitting `: T` is an
error. The source evaluates once. Vectors and strings use their ordinary
CLR elements (`char` for strings). Other sources must provide a supported
`IEnumerable<T>` or non-generic `IEnumerable` view; non-generic elements are
`object?`. Each element must be implicitly convertible to the declared loop
variable type. Use an explicit projection or cast for other conversions. Dispose
an acquired disposable enumerator on every exit.
Rectangular arrays follow CLR enumeration order and retain their known element
type. There are no user-defined duck-typed enumeration patterns in the baseline.

Enumeration preserves collection behavior. Modifying a collection can invalidate
its enumerator; the compiler does not insert a snapshot or prohibit all aliasing.

`break;` and `continue;` affect the innermost loop. `return` and `raise` may also
leave loops. All these exits honor applicable `finally` and
resource cleanup. Empty bodies are legal; `null;` is not an empty statement.

Async enumeration is outside the initial scope. Consume channels through their
ordinary async APIs; see [concurrency](concurrency.md).

## Case statements and expressions

```pascal
const Area: double := case ShapeValue of
  when Shape.Dot: 0.0;
  when Shape.Circle(const Radius: double): Math.PI * Radius * Radius;
  when Shape.Rectangle(const Width: double, const Height: double): Width * Height;
end case;
```

The selector evaluates once. Arms are tested in source order. `case` expressions
have one expression followed by `;` per arm; statements have a statement list.
Their delimiters are otherwise identical. `else` introduces a fallback arm.
There is no fallthrough.

Supported patterns:

| Pattern | Meaning |
| --- | --- |
| `42`, `'ready'`, `true`, `null`, `Direction.North` | Constant match |
| `0..9` | Inclusive integral constant range |
| `Shape.Circle(const Radius: double)` | Union case and typed immutable payload binding |
| `Shape.Rectangle(Height := const H: double, Width := const W: double)` | Named union payloads |
| `Tree.Leaf(Tree.Leaf(const Value: int))` | Nested union pattern for `Tree<Tree<int>>` |
| `_` | Any value |
| `const Value: T` | Any value of the selected static type T, with a new immutable binding |
| `is Sys.Exception as Failure` | CLR type test and non-null binding |

Payload patterns must name/position every payload, using `_` for ignored values.
Every `const Name: T` pattern requires its annotation to match the selected
payload/selector's static type, after resolving transparent aliases. The annotation
does not perform a conversion or introduce a runtime type filter. To narrow a
CLR type, use `is T as Name`, which already states the binding's type explicitly.
Do not mix named and positional payloads. Use `if Condition` after a complete
pattern as an optional guard. Guards may read its bindings. A false guard tries
the next arm. Comma-separated alternative patterns may share an arm only when
they introduce the same names with identical types.

Matching constants does not invoke arbitrary user operators. Primitive/string
constant patterns have their specified value comparison; enums compare underlying
values. A type pattern uses CLR type compatibility, not constructor execution.

## Coverage

- Language unions require exhaustive coverage in both
  statement and expression forms. Coverage descends into nested union patterns.
- `bool` requires true and false, or a fallback.
- Integer, string, and CLR enum selectors require an unguarded catch-all pattern
  or `else`. Enum metadata does not rule out unnamed runtime values.
- Subrange selectors use their declared base for constant/range patterns. These
  tests do not construct a new domain value. Coverage may prove the complete
  declared interval; otherwise a fallback is required. Unreachable/out-of-domain
  patterns are errors, and an annotation on a whole-value binding names the
  subrange type, not its primitive base.
- Reference-type matching must account for null when the selector is nullable
  and otherwise requires a fallback unless it is a closed language union.
- Guards do not prove coverage. Unreachable arms and duplicate constant patterns
  are errors. A catch-all after already exhaustive cases is unreachable.

An `is T as Name` pattern binds `Name` only inside its arm/guard. `Value is T`
outside patterns is a Boolean type test without introducing a binding.

## Record updates

```pascal
const Moved: Point := Position with X := Position.X + 1.0; end with;
```

`with` is a postfix expression for language records. It produces an updated
record, never a dynamic member-import scope. The operand evaluates once. See
[records](types.md#records) for sharing and equality rules.

# Exhaustiveness

Every `case` is exhaustive, with the same rules for statements and expressions.
Use `if` for selective handling. A missing value or variant reports FP3011;
a forbidden `else` reports FP3035, including after complete explicit coverage.

| Selector | Required unguarded coverage | `else` |
|----------|-----------------------------|--------|
| Enum, Result, Option | Every variant explicitly, including nested payloads | Forbidden |
| Boolean | Both true and false | Forbidden |
| Subrange | Every value within the declared bounds | Forbidden |
| Integer, string | Labels or guarded arms plus a fallback | Required |
| Distinct | Same as its supported underlying type, with typed labels | Same as the underlying type |

Aliases preserve their type's rules. Subrange coverage uses interval boundaries,
so large ranges do not require enumerating their values. Supported ranges may
overlap partially; only completely covered labels are unreachable. Ordinary
integer selectors still require `else`, even with a machine-wide range label.

## Enum exhaustiveness

```pascal
type
  Light = enum
    Red;
    Yellow;
    Green;
  end enum;

// ERROR: non-exhaustive match — missing Light.Yellow

case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Green:
    WriteLn('Go');
end case;
```

Fix by covering all variants:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Yellow:
    WriteLn('Caution');
  when Light.Green:
    WriteLn('Go');
end case;
```

Variants with the same behavior can share an arm:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Yellow, Light.Green:
    WriteLn('Proceed with caution');
end case;
```

## Closed-enum catch-alls

An enum catch-all hides the cases that need attention when the enum grows.
FP3035 rejects `else` and names a missing pattern for each uncovered variant.
Replace it with explicit arms, grouping variants with the same behavior:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Yellow, Light.Green:
    null;
end case;
```

When all values already have explicit coverage, remove the redundant `else`.
For handling just one variant, use an [`is` test](is-test.md):

```pascal
if L is Light.Red then
  WriteLn('Stop');
end if;
```

## Result and Option exhaustiveness

`Result` requires both `Ok` and `Error`. `Option` requires both `Some` and `None`:

```pascal
// ERROR: non-exhaustive — missing Error
case R of
  when Ok(const V):
    WriteLn(IntToStr(V));
end case;

// OK: both variants covered
case R of
  when Ok(const V):
    WriteLn(IntToStr(V));
  when Error(const E):
    WriteLn('Error: ' + E);
end case;
```

## Nested coverage

Coverage is checked recursively. Every value of the case type must match some
unguarded arm, including the payloads of nested patterns:

```pascal
// ERROR: non-exhaustive — missing Ok(None)
case Response of
  when Ok(Some(const User)):
    Greet(User);
  when Error(_):
    Retry();
end case;
```

The diagnostic names one missing pattern per uncovered variant, such as
`Ok(None)` or `Some(Shape.Rect(_, _))`.

## Unreachable labels

A label is unreachable when earlier unguarded arms already match every value
it matches, for scalar and variant cases alike. It is an error (FP3033):

```pascal
case Item of
  when Some(_):
    Use();
  when Some(0): // ERROR: unreachable after Some(_)
    Skip();
  when None:
    null;
end case;
```

Guarded arms never make later labels unreachable.

Duplicate labels within one arm are errors even when that arm has a guard.
Separate guarded arms may repeat a label: a failed guard permits later matches.
Fully covered labels can be covered by the union of several earlier ranges.
Empty ranges and labels outside a subrange's domain match no possible value
and are also rejected. Partial overlap remains useful for special cases followed
by a more general pattern, such as `Ok(0)` followed by `Ok(const Value)`.

Repeated comparisons are also unreachable: a second unguarded `Some(1)` or
`Error('timeout')` matches no new value. Named compile-time constants and
constant expressions are compared by their evaluated values, so `Some(Limit)`
and `Some(1 + 2)` overlap completely when `Limit` is `3`.

The same value comparison applies to known fields of static record constants,
including nested fields and constants derived from them. For example:

```pascal
type Flags = record
  Enabled: boolean;
end record;
const Settings: Flags := Flags(Enabled := true);
const Enabled: boolean := Settings.Enabled;
case Candidate of
  when Some(Enabled):
    null;
  when Some(false):
    null;
  when None:
    null;
end case;
```

For `Candidate: Option of boolean`, these arms are exhaustive. Replacing
`Some(false)` with `Some(Settings.Enabled)` produces a duplicate label (FP3033).
Boolean and simple-enum field values also contribute coverage after record copies
or updates and across compiled-unit imports.

## Rules

- Enum, `Result`, and `Option` types: every value must match an **unguarded**, explicit variant arm. `else` is rejected, including a redundant branch after complete coverage. This includes nested payloads; `when Ok(Some(_)):` does not cover `Ok(None)`.
- `_` and `const Name` cover every value of their payload position. Enum members, `true`/`false`, and nested variants cover their own values. Named compile-time constants contribute the same coverage as their values, including when parenthesized and across units. Subrange values cover their declared finite domain; open integer/string literals alone never complete coverage.
- Boolean and subrange cases require explicit finite coverage and reject `else`. Integer/string cases require `else`. Distinct Boolean follows Boolean rules; distinct integer/string follows the open scalar rules. Distinct real remains unsupported.
- A [`case` expression](../control-flow/case-of-intro.md#case-expressions) must produce a value for every input and follows the statement's same coverage matrix.
- Guard clauses do not count toward exhaustiveness — `when Shape.Circle(const R) if R > 0:` does not cover variant `Circle`; add an unguarded `when Shape.Circle(const R):` arm.
- Duplicate or fully covered labels are rejected for every supported selector
  type (FP3033). Partial overlaps are legal and select the first successful match.

## See also

- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
- [Guards](guards.md)

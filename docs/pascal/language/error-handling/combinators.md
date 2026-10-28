# Combinators

`Result operations` and `Option operations` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Result operations`](../types/result-operations.md) and [`Option operations`](../types/option-operations.md) for full API details.

```pascal id=example-1 fragment context=header
uses Std.Text.Format as Conv;
function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

const R: result of (integer, string) := Ok(21);
const M: result of (string, string) := R.Map(DoubleToString);
// M = Ok('42')

begin end.
```

```pascal id=example-2 fragment context=header
uses Std.Text.Format as Conv;
function PositiveToString(V: integer): option of string;
begin
  if V > 0 then
    return Some(Conv.IntToStr(V));
  else
    return None;
  end if;
end function;

const O: option of integer := Some(5);
const M: option of string := O.AndThen(PositiveToString);
// M = Some('5')

begin end.
```

| Combinator | Result | Option |
|------------|--------|--------|
| `V.Map(F)` | Transform `Ok` value | Transform `Some` value |
| `V.AndThen(F)` | Chain fallible operation | Chain optional lookup |
| `V.OrElse(F)` | Recover from `Error` | Provide fallback for `None` |

Receiver and callback argument expressions evaluate eagerly in source order.
The callback body runs only for the selected variant: Map and AndThen on Ok/Some,
OrElse on Error/None. An expression that creates an OrElse callback therefore runs
even when the receiver succeeds; the callback body still does not run.

Use `case` for payload binding, local handling, or a plain replacement value.
A case expression evaluates only its selected arm. OrElse returns a Result or
Option wrapper; its callback provides a wrapped recovery value.

## See also

- [Result](result.md)
- [Option](option.md)

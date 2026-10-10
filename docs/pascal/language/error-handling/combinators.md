# Combinators

`Result operations` and `Option operations` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Result operations`](../types/result-operations.md) and [`Option operations`](../types/option-operations.md) for full API details.

```pascal
uses Std.Conv;

function DoubleToString(V: integer): string;
begin
  return IntToStr(V * 2);
end function;

const R: Result of (integer, string) := Ok(21);
const M: Result of (string, string) := R.Map(DoubleToString);
// M = Ok('42')
```

```pascal
uses Std.Conv;

function PositiveToString(V: integer): Option of string;
begin
  if V > 0 then
    return Some(IntToStr(V));
  else
    return None;
  end if;
end function;

const O: Option of integer := Some(5);
const M: Option of string := O.AndThen(PositiveToString);
// M = Some('5')
```

| Combinator | Result | Option |
|------------|--------|--------|
| `Map(V, F)` | Transform `Ok` value | Transform `Some` value |
| `AndThen(V, F)` | Chain fallible operation | Chain optional lookup |
| `OrElse(V, F)` | Recover from `Error` | Provide fallback for `None` |

## See also

- [Result](result.md)
- [Option](option.md)

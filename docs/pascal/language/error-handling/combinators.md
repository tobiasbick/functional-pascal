# Combinators

`Std.Results` and `Std.Options` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Std.Results`](../../std/result/result.md) and [`Std.Options`](../../std/result/option.md) for full API details.

```pascal
uses Std.Results, Std.Conv;

function DoubleToString(V: integer): string;
begin
  return IntToStr(V * 2)
end;

var R: Result of integer, string := Ok(21);
var M: Result of string, string := Map(R, DoubleToString);
// M = Ok('42')
```

```pascal
uses Std.Options, Std.Conv;

function PositiveToString(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;

var O: Option of integer := Some(5);
var M: Option of string := AndThen(O, PositiveToString);
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

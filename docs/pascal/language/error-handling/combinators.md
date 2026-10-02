# Combinators

`Std.Results` and `Std.Options` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Std.Results`](../../std/result/result.md) and [`Std.Options`](../../std/result/option.md) for full API details.

```pascal
program Example;

uses Std.Results as Results;
uses Std.Conv as Conv;

function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

var R: result of integer, string := Ok(21);
var M: result of string, string := Results.Map(R, DoubleToString);

// M = Ok('42')
begin
  null;
end program;
```

```pascal
program Example;

uses Std.Options as Options;
uses Std.Conv as Conv;

function PositiveToString(V: integer): option of string;
begin
  if V > 0 then
    return Some(Conv.IntToStr(V));
  else
    return None;
  end if;
end function;

var O: option of integer := Some(5);
var M: option of string := Options.AndThen(O, PositiveToString);

// M = Some('5')
begin
  null;
end program;
```

| Combinator | Result | Option |
|------------|--------|--------|
| `Map(V, F)` | Transform `Ok` value | Transform `Some` value |
| `AndThen(V, F)` | Chain fallible operation | Chain optional lookup |
| `OrElse(V, F)` | Recover from `Error` | Provide fallback for `None` |

## See also

- [Result](result.md)
- [Option](option.md)

# Combinators

`Std.Results` and `Std.Options` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Std.Results`](../../std/result/result.md) and [`Std.Options`](../../std/result/option.md) for full API details.

Their callbacks must be `pure function` values. Use `case` or `try` to sequence
ordinary operations such as network requests, or to inspect resource-bearing
results and options.

```pascal
program Example;

uses Std.Results as Results;
uses Std.Conv as Conv;

pure function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

const R: result of (integer, string) := Result.Ok(21);
const M: result of (string, string) := Results.Map(R, DoubleToString);

// M = Ok('42')
begin
  null;
end program;
```

```pascal
program Example;

uses Std.Options as Options;
uses Std.Conv as Conv;

pure function PositiveToString(V: integer): option of (string);
begin
  if V > 0 then
    return Option.Some(Conv.IntToStr(V));
  else
    return Option.None;
  end if;
end function;

const O: option of (integer) := Option.Some(5);
const M: option of (string) := Options.AndThen(O, PositiveToString);

// M = Some('5')
begin
  null;
end program;
```

| Combinator | Result | Option |
|------------|--------|--------|
| `Map(V, F)` | Transform `Result.Ok` value | Transform `Option.Some` value |
| `AndThen(V, F)` | Chain fallible operation | Chain optional lookup |
| `OrElse(V, F)` | Recover from `Result.Error` | Provide fallback for `Option.None` |

## See also

- [Result](result.md)
- [Option](option.md)

# Result and Option patterns

Destructuring `case` arms for `Result of (T, E)` and `Option of (T)`:

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

case Success of
  when Result.Ok(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Result.Error(const Message):
    Console.WriteLn(Message);
end case;

case Console.Present of
  when Option.Some(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Option.None:
    Console.WriteLn('empty');
end case;
```

Grouped patterns in one arm must introduce the same binding names and types:

```pascal
uses Std.Console as Console;

case R of
  when Result.Ok(const Msg), Result.Error(const Msg):
    Console.WriteLn(Msg);
end case;
```

Because all labels share one body, they must expose the same binding names with
compatible payload types. Binding names are case-insensitive. Use separate arms
when labels expose different names or payload types.

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Error handling](../error-handling/README.md)
- [Exhaustiveness](exhaustiveness.md)

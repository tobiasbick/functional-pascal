# Result and Option patterns

The code fragments use `uses Std.Conv as Conv, Std.Console as Console;`.

`Some`, `Ok`, and `Error` match one explicit payload, either by position or
with the case-insensitive label `Value`: `Some(Value := const N)`,
`Ok(Value := _)`, or `Error(Value := 'timeout')`. Error payloads use their
declared error type. Nested patterns choose their own positional or named
field list. Unknown, repeated or missing fields are errors; `_` explicitly
ignores a payload. `None` has no payload and uses no parentheses.

Destructuring `case` arms for `Result of (T, E)` and `Option of T`:

```pascal
case Success of
  when Ok(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Error(const Message):
    Console.WriteLn(Message);
end case;

case Present of
  when Some(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when None:
    Console.WriteLn('empty');
end case;
```

Payloads may hold nested patterns and comparisons:

```pascal
case Lookup of
  when Ok(Some(const User)):
    Greet(User);
  when Ok(None):
    Console.WriteLn('not found');
  when Error('timeout'):
    Retry();
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

Multiple destructure labels in one arm may reuse one binding name:

```pascal
case R of
  when Ok(const Msg), Error(const Msg):
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

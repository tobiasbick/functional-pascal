# Result and Option patterns

Destructuring `case` arms for `Result of T, E` and `Option of T`:

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

case Success of
  when Ok(Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Error(Message):
    Console.WriteLn(Message);
end case;

case Console.Present of
  when Some(Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when None:
    Console.WriteLn('empty');
end case;
```

Multiple destructure labels in one arm may reuse one binding name:

```pascal
uses Std.Console as Console;

case R of
  when Ok(Msg), Error(Msg):
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

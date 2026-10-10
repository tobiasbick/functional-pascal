# Result and Option patterns

Destructuring `case` arms for `Result of (T, E)` and `Option of T`:

```pascal
case Success of
  when Ok(const Value):
    WriteLn(IntToStr(Value));
  when Error(const Message):
    WriteLn(Message);
end case;

case Present of
  when Some(const Value):
    WriteLn(IntToStr(Value));
  when None:
    WriteLn('empty');
end case;
```

Payloads may hold nested patterns and comparisons:

```pascal
case Lookup of
  when Ok(Some(const User)):
    Greet(User);
  when Ok(None):
    WriteLn('not found');
  when Error('timeout'):
    Retry();
  when Error(const Message):
    WriteLn(Message);
end case;
```

Multiple destructure labels in one arm may reuse one binding name:

```pascal
case R of
  when Ok(const Msg), Error(const Msg):
    WriteLn(Msg);
end case;
```

Because all labels share one body, they must expose the same binding names with
compatible payload types. Binding names are case-insensitive. Use separate arms
when labels expose different names or payload types.

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Error handling](../error-handling/README.md)
- [Exhaustiveness](exhaustiveness.md)

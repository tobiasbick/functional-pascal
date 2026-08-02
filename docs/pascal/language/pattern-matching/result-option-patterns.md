# Result and Option patterns

Destructuring `case` arms for `Result of T, E` and `Option of T`:

```pascal
case Success of
  Ok(Value): WriteLn(IntToStr(Value));
  Error(Message): WriteLn(Message)
end;

case Present of
  Some(Value): WriteLn(IntToStr(Value));
  None: WriteLn('empty')
end;
```

Multiple destructure labels in one arm may reuse one binding name:

```pascal
case R of
  Ok(Msg), Error(Msg):
    WriteLn(Msg)
end;
```

Because all labels share one body, they must expose the same binding names with
compatible payload types. Binding names are case-insensitive. Use separate arms
when labels expose different names or payload types.

## See also

- [Types — Result and Option](../types/result-option-types.md)
- [Error handling](../error-handling/README.md)
- [Exhaustiveness](exhaustiveness.md)

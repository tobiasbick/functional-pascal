# Record updates

Use with to create a shallow updated copy of a source record.

```pascal fragment context=point-update
const Moved: Point := Position with X := Position.X + 1.0; end with;
```

`with` is a postfix expression for language records. It produces an updated
record, never a dynamic member-import scope. The operand evaluates once. See
[records](records.md) for sharing and equality rules.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

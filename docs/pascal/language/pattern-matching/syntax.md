# Case statements and expressions

Case evaluates one selector and tests ordered arms with typed patterns.

```pascal fragment context=shape
const Area: double := case ShapeValue of
  when Shape.Dot: 0.0;
  when Shape.Circle(const Radius: double): Math.PI * Radius * Radius;
  when Shape.Rectangle(const Width: double, const Height: double): Width * Height;
end case;
```

The selector evaluates once. Arms are tested in source order. `case` expressions
have one expression followed by `;` per arm; statements have a statement list.
Their delimiters are otherwise identical. `else` introduces a fallback arm.
There is no fallthrough.

Supported patterns:

| Pattern | Meaning |
| --- | --- |
| `42`, `'ready'`, `true`, `null`, `Direction.North` | Constant match |
| `0..9` | Inclusive integral constant range |
| `Shape.Circle(const Radius: double)` | Union case and typed immutable payload binding |
| `Shape.Rectangle(Height := const H: double, Width := const W: double)` | Named union payloads |
| `Tree.Leaf(Tree.Leaf(const Value: int))` | Nested union pattern for `Tree<Tree<int>>` |
| `_` | Any value |
| `const Value: T` | Any value of the selected static type T, with a new immutable binding |
| `is Sys.Exception as Failure` | CLR type test and non-null binding |

Payload patterns must name/position every payload, using `_` for ignored values.
Every `const Name: T` pattern requires its annotation to match the selected
payload/selector's static type, after resolving transparent aliases. The annotation
does not perform a conversion or introduce a runtime type filter. To narrow a
CLR type, use `is T as Name`, which already states the binding's type explicitly.
Do not mix named and positional payloads. Use `if Condition` after a complete
pattern as an optional guard. Guards may read its bindings. A false guard tries
the next arm. Comma-separated alternative patterns may share an arm only when
they introduce the same names with identical types.

Matching constants does not invoke arbitrary user operators. Primitive/string
constant patterns have their specified value comparison; enums compare underlying
values. A type pattern uses CLR type compatibility, not constructor execution.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

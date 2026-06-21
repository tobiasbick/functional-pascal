# Primitive types

| Type      | Description                  | Example              |
|-----------|------------------------------|----------------------|
| `integer` | 64-bit signed integer        | `42`, `-7`, `0`      |
| `real`    | 64-bit floating point        | `3.14`, `-0.5`       |
| `boolean` | Boolean                      | `true`, `false`      |
| `string`  | Immutable text sequence      | `'Hello'`, `'A'`     |

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (literals, `type_expr` built-ins).

Single-character text uses the same `string` type as longer text:

```pascal
var
  Letter: string := 'A';
  Word: string := 'Hello';
```

Strings use single quotes with doubled apostrophes for escaping: `'It''s Pascal'`.

Strings may span multiple lines:

```pascal
var
  Poem: string := 'Roses are red
Violets are blue';
```

## Character codes

The `#` prefix denotes a character by its ASCII code (decimal, range **0..255**). These can be concatenated directly with string literals:

```pascal
var
  LineBreak: string := #13#10;                  { CR+LF }
  Greeting: string := 'Hello'#13#10'World';     { Hello\r\nWorld }
  Tab: string := #9;                            { tab character }
  Letter: string := #65;                        { 'A' }
```

## See also

- [Number literals](number-literals.md)
- [Operators — string indexing](operators.md#string-indexing)

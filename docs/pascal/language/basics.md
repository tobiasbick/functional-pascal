# Basics

## Lexical rules

Sources are UTF-8 `.fpas` files. LF, CRLF, and bare CR terminate lines; an initial
UTF-8 BOM is allowed. Whitespace is otherwise insignificant. Comments start with
`//` and end at the next line ending or EOF. Only line comments are supported;
brace, parenthesis-star, and slash-star forms are not comments. Consecutive
standalone comments immediately before a declaration supply its Markdown
documentation; a blank source line detaches them. See
[source comments and API documentation](comments.md) for attachment, structured
parameter/return descriptions, and the planned .NET XML export.

Keywords and identifiers are **case-sensitive**. Keywords are lowercase;
`Count` and `count` are different identifiers. Identifiers use ASCII letters,
digits, and `_`, with no leading digit. `_` alone is a pattern wildcard, not a
value binding. Prefix a keyword with `@` to use it as an identifier, including
in an imported CLR name: `Value.@event`. The `@` is not part of its metadata name.

Primitive aliases are predefined type names, not keywords. They cannot be
redeclared in a type namespace. See [types](types.md).

### Literals

| Form | Meaning |
| --- | --- |
| `42`, `1_000`, `$FF` | Integral literal; default type `int` |
| `42L`, `42U`, `42UL` | `long`, `uint`, `ulong` |
| `1.5`, `1e3`, `1.5D` | `double` |
| `1.5F`, `1.5M` | `float`, `decimal` |
| `'Hello'`, `'It''s ready'` | `string`; doubled apostrophe escapes an apostrophe |
| `#65`, `#10` | `char`, one UTF-16 code unit, range 0..65535 |
| `true`, `false` | `bool` |
| `null` | Null literal for nullable CLR types |

Numeric suffixes accept either case. `_` may occur singly between digits.
Hexadecimal literals may have integral suffixes. Negative numbers use unary `-`;
a directly negated literal may represent the minimum signed value.
An `int` constant may be contextually converted to a smaller integral type,
an unsigned integral type, or a native integral type when representable;
a representable `long` constant may also convert to `ulong`. Char requires a
character literal or an explicit cast. Otherwise, an unsuffixed integral literal must fit `int`;
use an explicit suffix for larger values. Floating literals must fit their
chosen representation; use CLR APIs to obtain infinity or NaN.

Strings do not span source lines. Use concatenation and character literals:
`'First' + #10 + 'Second'`. There is no adjacent-literal concatenation. A `char`
may be a surrogate code unit; it is not a Unicode scalar. Use `System.Text.Rune`
when scalar processing is needed.

## Bindings and assignment

```pascal
const Name: string := 'Mara';
var Health: int := 10;
const Limit: int := 100;
Health := Health - 1;
```

Every `const` or `var` declaration introduces one name and requires both an
explicit type and an initializer, at every scope. The syntax is
`const Name: T := Expression;` or `var Name: T := Expression;`. Omitting `: T` is
an error even when the initializer's type is obvious. The initializer must be
implicitly convertible to the declared type; the binding retains that type.

The same explicit-type rule applies to loop variables, using resources, and
pattern bindings. Parameters, function results, and record fields also require
types. A named ensures result uses `ensures (Returned: T): Condition;` and also
requires its annotation. A CLR type pattern states its binding's type before `as`; a wildcard
introduces no binding. Compiler-provided `self` and `args` have the fixed types
defined by their enclosing record and program.

The declared type supplies context to empty array literals, `null`, and generic
union constructors. Expression typing, overload resolution, and generic argument
inference still operate; they never remove a declaration's type requirement.

`const` means an immutable **binding**, not necessarily a compile-time constant.
Its initializer may execute at runtime. A `const` reference can still refer to a
mutable object; calling `Items.Add(1)` is legal on a `const` list. `var` allows
rebinding. Parameters are read-only bindings unless declared `ref` or `out`.

Assignment evaluates the receiver and indexes once, then the right side, in
source order. It can target a local `var`, a writable CLR field/property/indexer,
a writable array element, or a `ref`/`out` parameter. Properties may have effects.
It can also target an accessible top-level var, emitted as a CLR static field.
It cannot target a language record's read-only field or a domain's read-only payload.

There are no implicit declarations or implicit conversions from values to `bool`.
All statements end with `;`, including the last statement before `else` or a named
`end`. An empty statement list is allowed. A nested `begin ... end;` creates scope.
Programs close with `end program;` and units with `end unit;`; neither has a
terminal period. The program's entry block first closes with its own `end;`.
See [program structure](../program-structure/README.md).
Local resource bindings use `using const Name: T := Expression;` (or
`await using const` for async disposal), without a separate block closer.
Their cleanup runs on leaving the enclosing lexical scope; see
[errors and resources](errors-and-resources.md).

## Operators

Precedence, high to low:

1. Calls, member/index access, record `with` updates.
2. Unary `+`, `-`, `not`, `await`.
3. `*`, `/`, `div`, `mod`.
4. `+`, `-`.
5. `shl`, `shr`.
6. `<`, `<=`, `>`, `>=`, `is`.
7. `=`, `<>`.
8. `and`, then `xor`, then `or`.
9. `??` (right associative).

Other binary operators associate left. Comparison/equality operators do not
chain. Parenthesize expressions combining `await` and coalescing when their
grouping would be unclear: `(await Load()) ?? 'Fallback'`.

- `and`/`or` on `bool` short-circuit. `xor` is eager.
- On integral values, `and`, `or`, `xor`, `not`, `shl`, and `shr` are bitwise,
  with CLR/C# numeric promotion and shift-count rules. Same-type enum bitwise
  operations retain the enum type.
- `/` accepts promoted `float`, `double`, or `decimal` operands. Two integral
  operands require `div` or an explicit floating conversion.
- `div` and `mod` accept integral operands; division truncates toward zero.
- Numeric widening and binary promotion follow C# rules. For example,
  `int + long` produces `long`, and `short + short` produces `int`.
  `decimal` does not implicitly mix with `float` or `double`.
- Integral arithmetic and narrowing conversions check overflow by default.
  `unchecked(Expression)` enables wrapping integral arithmetic/conversions only
  inside that expression. Division and decimal arithmetic retain CLR exceptions.
- Floating operations follow IEEE 754 and may produce infinity or NaN.
- `string + string`, `string + char`, and `char + string` concatenate.
  Arbitrary objects are not implicitly formatted into strings.
- `Value ?? Fallback` evaluates the fallback only if the nullable value is null.
  For `Nullable<T>`, the non-null result is the contained `T`.

`=` and `<>` use primitive equality or an applicable CLR equality operator.
Strings compare ordinal contents; arrays use reference identity. Generated
record/union equality is described in [types](types.md). Other reference types
without an applicable equality operator use reference identity, not an implicit
call to virtual `Equals`. Ordering requires an applicable primitive/operator rule.

## Casts and runtime types

`cast<T>(Expression)` performs an explicit numeric or CLR reference conversion.
Numeric casts are checked unless enclosed by `unchecked`. An incompatible
reference cast throws `InvalidCastException`; it is not an optional conversion.
`Value is T` tests runtime type compatibility, yielding false for null.
`typeof(T)` produces `System.Type`. Casts do not parse strings or translate
application-defined union values into nullable values.
Compiler-defined casts wrap/unwrap [distinct types and subranges](data-domains.md).
Subrange bounds remain checked inside unchecked expressions.

`typeof(T)` uses a statically resolved type, not a value expression. It produces
a non-null Type without constructing an instance or running the operand type's
static constructor/unit initializer. Transparent aliases identify their CLR target;
nominal source types retain their emitted identity. Reference nullable annotations
do not create a distinct runtime type, while nullable values retain Nullable<T>
identity. Closed generic types and in-scope type parameters follow the existing
type/constraint rules. See the [implemented typeof milestone](../../implementation/typeof.md).

## Keywords

```text
async await begin break case catch const continue discard distinct div do downto else
elsif end ensures enum false finally for function if implements in is mod new not null
of or out params private procedure program public raise range record ref repeat return
requires shl shr static subscribe then to true try type union unit unsubscribe until uses using var
when where while with and xor
```

`as` is also a keyword, used in imports and type patterns. `class` and `struct`
are constraint keywords. `cast`, `typeof`, and `unchecked` are reserved
compiler-recognized forms with the syntax above; escape them with `@` to refer
to foreign identifiers with those names. `self` and `args` are reserved implicit bindings in
their applicable scopes. There are no predefined `integer`, `real`, `go`, `dict`,
`channel`, or task-type keywords.

# Local variables

Variables can be declared inline inside statement lists. Every binding starts
with its own `const` or `var` keyword. The same rule
applies in routines and nested scopes. Local type declarations are not permitted.
`const` bindings are immutable; `var` allows reassignment.
Prefer `const` for immutable bindings in application and library code.
See [Constants](constants.md) for runtime initialization and static classification.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_stmt` and `var_stmt` in statement position).

```pascal
function FullName(First: string; Last: string): string;
begin
  const Space: string := ' ';
  return First + Space + Last;
end function;
```

## Limited type inference

Explicit [subrange conversions](../types/subranges.md), such as
`const Level := Percent(Value);`, infer the range type and retain its checked
bounds. Ordinary routines returning a subrange still need an annotation.

In statement position, a `const` or `var` binding can omit its type annotation
for scalar literals, non-empty array and dictionary literals, typed record or
enum construction, and explicit type conversions. Parentheses and a numeric
literal's sign do not change this rule. The inferred type is fixed for the
lifetime of the binding.

```pascal
type UserId = distinct integer;

function Sample(): integer;
begin
  const Count := 42;
  var Offset := -2;
  const Names := ['Ada', 'Grace'];
  const Scores := ['Ada': 42, 'Grace': 7];
  const Id := UserId(Count);
  const Raw := integer(Id);
  Offset := Offset + 1;
  return Raw + Offset + Names.Length() + Scores.Length();
end function;
```

Collection elements must provide complete, matching types; dictionary keys and
values are checked separately. Nested empty collections and unresolved generic
arguments require an annotation. Mixed numeric literals such as `[1, 2.0]`
are invalid even with an annotation. Import `Std.Conv` and convert explicitly,
for example `[IntToReal(1), 2.0]`; inference does not choose a numeric promotion.
Expressions inside a collection literal can supply known element types.

Typed construction uses the resolved record type or enum variant, including
imported types and type aliases. Generic arguments must be fully determined by
supplied fields or payloads, or already fixed by a type alias. Record defaults
do not infer arguments. An uninstantiated generic routine payload does not
supply a complete type. A non-generic payloadless enum variant also provides
its type; a generic payloadless variant needs an annotation or a fixed alias.
See [Records](../types/records.md), [Enums](../types/enums.md), and
[constructor inference](../types/generics.md#generic-records).

```pascal
type Point = record X: integer; Y: integer; end record;
type Box of T = record Value: T; end record;
type Lookup of T = enum Found(Value: T); Missing; end enum;
type IntLookup = Lookup of integer;

function Construct(): integer;
begin
  const P := Point(X := 1, Y := 2);
  var B := Box(Value := P);
  const Found := Lookup.Found(B);
  const Missing := IntLookup.Missing;
  const Present := Some(1);
  const Unknown: Lookup of string := Lookup.Missing;
  return Present.Unwrap();
end function;
```

`Some(Value)` supplies `Option of T` when its payload has a complete type.
`None` leaves that type undetermined. `Ok(Value)` and `Error(Value)` leave one
of the two `Result` arguments undetermined and require an annotation.

Ordinary function and method calls, references to bindings, operator expressions,
closures, conditional expressions, record updates, and field or index access
require an annotation. This includes a factory returning a record or enum and
access following a constructor, such as `Point(X := 1, Y := 2).X`.
Conversion functions such as `IntToReal(...)` are ordinary calls; explicit type
conversions use `Type(Value)`, such as `UserId(42)`.
[FP3036](../../tools/diagnostics.md#semantic-analysis) asks for the missing annotation.

```pascal
const Empty: array of integer := [];
const Mixed: array of real := [IntToReal(1), 2.0];
const Converted: real := IntToReal(42);
const Computed: integer := ReadValue();
```

Declarations outside statement lists, including program and unit declarations
and routine declarations before `begin`, remain explicitly typed. Parameters,
routine results, and record fields also require annotations.

## See also

- [Variables](variables.md)
- [Runnable inference example](../../../../examples/pascal/basics/local_inference.fpas)

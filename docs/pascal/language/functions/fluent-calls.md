# Dot calls and function chaining

A dot call selects a declared record member or a fixed operation of a built-in
receiver type. Strings, arrays, dictionaries, `Option`, and `Result` provide
these operations without imports. Each receiver type has one signature per
case-insensitive operation name. Your own free functions are called ordinarily.

```pascal
program Chaining;

function IsEven(Value: integer): boolean;
begin
  return Value mod 2 = 0;
end function;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

function Sum(Acc: integer; Value: integer): integer;
begin
  return Acc + Value;
end function;

begin
const Values: array of integer := [1, 2, 3, 4];
const Total: integer := Values.Filter(IsEven).Map(Double).Reduce(0, Sum);
end.
```

Every step receives the previous result. Results may change type, for example
`Text.Split(',').Map(ParseNumber).Find(IsPositive).UnwrapOr(0)`. Processing is
eager. Receivers and explicit arguments are evaluated once; the receiver is
first, followed by arguments in written order. There is no implicit unwrap or
conversion.

## Fixed targets and names

Record methods retain their declared `Self` parameter and can return `Self`,
a new record, or another type to continue a chain. Callable fields and
properties take only their explicit arguments. Invalid record members do not
fall through to free functions.

Built-in operations use the static receiver type and catalog name alone.
Local or imported functions with the same name cannot redirect a dot call.
Wrong argument names, counts, or types produce errors on that selected entry.

```pascal
function Length(Text: string): integer;
begin
  return 100;
end function;

const A: integer := Length('hi');    // Your free function: 100
const B: integer := 'hi'.Length();   // Built-in string operation: 2
```

`Length()` and `IsEmpty()` are shared by strings, arrays, and dictionaries.
Strings and arrays both use `Slice(Start, Len)`; string indexes count Unicode
scalars. Generic containers retain their operations when their shape is known.
An unconstrained type parameter, scalar, channel, or task handle has no native
instance catalog. A free function returning a record or built-in type can
start a chain: `Build().Next()` or `MakeText().Trim()`.

## Arguments and factories

Fixed-signature operations accept positional or fully named explicit arguments.
Names match case-insensitively; all parameters are required. Mixed, unknown,
duplicate, or missing names are rejected. The receiver cannot be named.

```pascal
const Text: string := 'hello';
const Part: string := Text.Slice(Len := 3, Start := 1);
const Letter: string := string.Chr(N := 65);
const Copies: array of string := array.Fill(Count := 2, Value := 'hi');
const Message: string := '%s: %d'.Format('items', 3);
```

Named arguments still evaluate in written order, then map to declaration order.
`Format` is the sole variadic native operation and accepts only positional,
heterogeneously typed arguments. `string.Chr(N)` and `array.Fill(Value, Count)`
construct values without a receiver. `Fill` infers its element type from
`Value`, including when `Count` is zero. Binding type annotations remain
required. Explicit factory generic arguments and `(array of T).Fill` are invalid.

The former helper units `Std.Str`, `Std.Arrays`, `Std.Dictionaries`,
`Std.Options`, and `Std.Results`, their free calls, and their free routine
references have been removed. Use a named or anonymous wrapper when a callback
is needed, such as `function(S: string): integer begin return S.Length(); end function`.
Other standard-library units still require `uses`.

## Procedures, mutation, and tasks

A procedure may finish a statement chain; it produces no value for a later step.
`Items.Push(Value)` and `Items.Pop()` require writable array storage without a
receiver marker or additional parentheses. Fields, array elements, and forwarded
`var` parameters use the shared storage checks. Constants, read-only parameters,
properties, dictionary entries, and computed receivers are rejected (FP3028).
`Items.Push(Value := 3)` applies the same checks. Explicit written `var` arguments
retain their markers; `Push`'s `Value` parameter is read-only.

The receiver's root and indices are determined once before arguments. Mutation
reads and writes that storage after argument evaluation, and copy-on-write
preserves other values sharing it. Completed writes survive failures and early
`try` exits. `Pop` returns the removed element, which can continue a chain.

`go` can spawn an eligible read-only native call and evaluates its receiver and
arguments before spawning. Writable receivers cannot cross a task boundary
(FP3030), so `go Items.Push(Value)` and `go Items.Pop()` are rejected.

## See also

- [Postfix chaining](postfix-chaining.md)
- [Record methods](../types/record-methods.md)
- [String operations](../types/string/README.md)
- [Array operations](../types/array/README.md)
- [Dictionary operations](../types/dictionary-operations.md)
- [Option operations](../types/option-operations.md)
- [Result operations](../types/result-operations.md)

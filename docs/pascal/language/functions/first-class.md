# First-class functions

The code fragments use `uses Std.Console as Console;`.

Functions can be assigned to variables and passed as arguments:

```pascal id=example-1 fragment context=header-console
function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value);
end function;

function Double(X: integer): integer;
begin
  return X * 2;
end function;

begin
  const R: integer := Apply(Double, 5); // 10
  const Op: function(X: integer): integer := Double;
  Console.WriteLn(Op(7)); // 14
end.
```

Call sites pass a **named** function or procedure, a **closure expression**, a
**bound record method**, a [payload variant constructor](function-types.md#payload-constructors-as-functions), or a **variable** whose type is a function or procedure
type. Imported routines are called through their explicit import alias, for
example `Console.WriteLn(...)` after `uses Std.Console as Console;`.

Array elements and dictionary entries with callable types are also called
through their designators. The indexed callable is selected once before its
arguments are evaluated:

```pascal id=example-2 fragment context=first-class-collections
const Operations: array of function(X: integer): integer := [Double];
const ByName: dict of string to function(X: integer): integer := ['double': Double];
const First: integer := Operations[0](7); // 14
const Named: integer := ByName['double'](7); // 14
```

These calls accept positional or fully named arguments from their visible
static signature, for example `Operations[0](X := 7)`. Each call uses one
argument style throughout, like other first-class callable values.
Procedure elements may be called as statements. Their `var` parameter modes
and the restrictions on [task calls](../concurrency/go.md) remain the same.

```pascal id=example-3 fragment context=first-class-bound
type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

begin
  const C: Counter := Counter(
    Base := 10
  );
  const AddTen: function(Value: integer): integer := C.Add;
  Console.WriteLn(AddTen(5)); // 15
  Console.WriteLn(Apply(AddTen, 7)); // 17
end.
```

`C.Add` captures `C` by value once; calling `AddTen` supplies only the remaining
parameters. See [Record methods](../types/record-methods.md#bound-methods-as-values).

## Native operation values

A callable name without argument parentheses denotes a function or procedure
value; parentheses invoke it. The same rule applies to source routines, Rust
intrinsics, factory functions and payload constructors. Each operation keeps its
canonical name; passing it as a callback does not require another helper API.

A fixed signature such as `Math.Sqrt` can be inferred. Polymorphic operations
such as `Math.Abs` need a receiving callable signature. An annotation or typed
callback parameter supplies the argument and result types:

```pascal id=native-callable-values run
program NativeCallables;

uses Std.Math as Math, Std.Test as Test;

begin
  const Abs: function(N: integer): integer := Math.Abs;
  Test.AssertEquals(7, Abs(N := -7));
  const Positive := [-3, 1, -2].Map(Math.Abs);
  Test.AssertEquals(3, Positive[0]);
  const Fill: function(Value: integer; Count: integer): array of integer := array.Fill;
  Test.AssertEquals(7, Fill(7, 3)[2]);
end.
```

A native receiver reference such as `Text.Trim` evaluates the receiver once and
captures its current value. Later changes to the original binding do not replace
that value. Arrays and dictionaries retain value semantics; shared-resource
fields keep their existing shared-handle behavior. Computed and indexed receivers
follow the same rule, for example `ReadText().Trim` or `Texts[Index].Length`.
The callable takes only the operation's remaining parameters. Operations returning
no value, such as `Items.ForEach`, produce procedure values.

```pascal id=native-receiver-values run
program NativeReceiverValues;

uses Std.Math as Math, Std.Test as Test;

begin
  var Text := '  hello  ';
  const Trim := Text.Trim;
  Text := 'changed';
  Test.AssertEquals('hello', Trim());
  const Values := [-3, 1];
  const Map: function(F: function(X: integer): integer): array of integer := Values.Map;
  Test.AssertEquals(3, Map(Math.Abs)[0]);
end.
```

Generic operations need enough context to resolve every remaining type.
Variadic operations specialize to one fixed callable arity and parameter list:
`const Write: procedure(Text: string; N: integer) := Console.WriteLn;` requires
`uses Std.Console as Console;`. Likewise,
`const Format: function(N: integer): string := '%d'.Format;` captures the template.
Argument types and comparison constraints remain those of the selected operation.
Labels at later calls come from the visible callable signature, including explicit
receiving annotations.

Native InPlace operations require writable source storage and cannot capture
their receiver by value. Call them directly, or declare an ordinary routine with
an explicit `var` parameter. Callable values preserve their existing `var` modes,
capture restrictions, task-handle lifetimes and worker-transfer checks. Binding a
function does not execute its body. Type names remain types: record construction
and scalar conversions do not become function values.

## Optional handlers

A callback can be an ordinary record field of type `Option of HandlerType`.
Its `None` default represents the absence of a handler; `Some(...)` stores a
named routine, closure, or bound method. Use a `case` arm when the callback
should run only when present:

```pascal id=example-4
program OptionalHandlers;

uses Std.Console as Console;

type ClickHandler = procedure(Sender: integer);

type Button = record
  Id: integer;
  OnClick: option of ClickHandler := None;

  procedure Click(Self: Button);
  begin
    case Self.OnClick of
      when Some(const Handler):
        Handler(Self.Id);
      when None:
        null;
    end case;
  end procedure;
end record;

procedure HandleClick(Sender: integer);
begin
  Console.WriteLn(Sender);
end procedure;

begin
  var B: Button := Button(Id := 1);
  B.OnClick := Some(HandleClick);
  B.Click(); // Prints 1.
  B.OnClick := None;
  B.Click(); // No handler, so no call.
end.
```

Ordinary field rules apply: setting or clearing a handler requires a mutable
record binding; a `const` record can read the field and call an installed
handler. Construction and record updates can supply the field, and copies
retain their own handler values. Unit fields are private by default; `public`
exposes a field for ordinary reads, assignments, and calls from consumers.
The `None` default is preserved in imported records and transparent aliases.

When absence is an error, select the callable with `Value.OnClick.Unwrap()`
and call the resulting value. `Unwrap` panics for `None`; it does not supply a
default result. A full `case` may instead handle both `Some` and `None`
explicitly. Calls retain the normal callable parameter and capture rules.

## See also

- [Function types](function-types.md)
- [Capturing closures](closures.md)
- [Record methods](../types/record-methods.md)
- [`Array operations`](../types/array/README.md) — `Map`, `Filter`, and other higher-order helpers

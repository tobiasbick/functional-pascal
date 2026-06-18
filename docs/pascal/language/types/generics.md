# Generics

Functions and procedures declare type parameters in angle brackets (`<T>`). Record methods may declare type parameters on the method itself.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`generic_params`, `constraint`).

## Generic functions and procedures

```pascal
function Identity<T>(Value: T): T;
begin
  return Value
end;

procedure PrintValue<T>(Value: T);
begin
  WriteLn(Value)
end;
```

Type arguments are inferred from the call-site arguments — no explicit instantiation is needed:

```pascal
var
  X: integer := Identity(42);    { T inferred as integer }
  S: string  := Identity('hi');  { T inferred as string  }
```

## Generic record methods

Record methods declare type parameters in the method header; those parameters are scoped to the method.

```pascal
type
  Box = record
    Value: integer;

    function Map<R>(Self: Box; F: function(X: integer): R): R;
    begin
      return F(Self.Value)
    end;
  end;

function ToText(X: integer): string;
begin
  return 'value=' + IntToStr(X)
end;

var
  B: Box := record Value := 42; end;
  S: string := B.Map(ToText);   { R inferred as string }
```

Method-level type parameters may also use constraints:

```pascal
type
  Accumulator = record
    function Add<T: Numeric>(Self: Accumulator; Extra: T): T;
    begin
      return Extra
    end;
  end;
```

## Implementation

Generics use type erasure. The VM operates on dynamic values, so no monomorphization is needed. Type parameters are checked at compile time and erased at runtime.

## Constraints

Type parameters can be constrained to require specific capabilities from the concrete type. Constraints are written after the parameter name, separated by a colon: `<T: Constraint>`.

### Built-in constraints

| Constraint | Satisfied by | Description |
|------------|-------------|-------------|
| `Comparable` | `integer`, `real`, `boolean`, `char`, `string` | Supports comparison operators: `=`, `<>`, `<`, `>`, `<=`, `>=` |
| `Numeric` | `integer`, `real` | Supports arithmetic operators: `+`, `-`, `*`, `/`, `div`, `mod` |
| `Printable` | All types except `function` and `procedure` | Can be converted to a string representation |

### Examples

```pascal
function Max<T: Comparable>(A: T; B: T): T;
begin
  if A > B then return A else return B
end;

function Add<T: Numeric>(A: T; B: T): T;
begin
  return A + B
end;
```

Constraint violations at call sites are compile-time errors:

```pascal
var
  M: integer := Max(3, 7);    { OK — integer is Comparable }
{ var Bad := Max([1], [2]);   ← compile error: array is not Comparable }
```

## See also

- [Record methods](record-methods.md)
- [Generic routines](../functions/generic-routines.md)

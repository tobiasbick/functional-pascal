# Patterns and errors

Read this when selecting variants, extracting payloads, handling absence or
propagating expected failures.

## Pattern matching

`case Value of when Pattern: ... end case;` evaluates Value once and selects an
arm in written order. Comma-separated labels share an arm. Bind names explicitly
with `const`, as in `when Some(const Value):`; `_` ignores one payload.
Bare identifiers refer to compile-time constants rather than introducing names.
Payload patterns can nest and are fully positional or fully named, covering
every field once. Some/Ok/Error use `Value` as their named payload label.

Guards follow the whole pattern: `when Shape.Circle(const R) if R > 0:`.
Scalar bindings need a guard, as in `when const N if N > 0:`.
Integer labels allow inclusive `Low..High` ranges; labels and bounds require
compile-time constants. Use guards for runtime computations.

Statements and expressions share exhaustiveness rules. Enum/error families,
Option and Result must cover every variant; Boolean needs true and false;
subranges need their complete interval. These closed domains forbid `else`.
Integer/string cases require `else`; distinct follows its supported base type.
Guards never count toward coverage. Nested payload coverage is checked
recursively. Fully covered labels are rejected; partial overlaps retain order.
Case expressions provide one expression followed by `;` per arm.

`Value is Pattern` produces a Boolean without binding names. It supports normal
Boolean expressions and short-circuiting; any `const Name` inside its pattern is
rejected. Use a case arm when you need the payload.

## Result, Option and propagation

`Result of (T, E)` contains `Ok(T)` or `Error(E)`; `Option of T` contains `Some(T)`
or `None`. `None` has no parentheses. `Unwrap()` deliberately aborts on a failure
or missing value. Use exhaustive case for local handling or defaults; only the
selected arm runs.

`try Expression` evaluates once, unwraps success, or returns failure from the
enclosing function. That function needs the same wrapper family. Success types
may differ, but Result error types must be compatible, preserving nominal
identity and generic parameters. Program bodies and procedures cannot use `try`.
`panic(Message)` and runtime failures have no catch block; `try` does not catch them.

Native wrapper operations need no import. `Map` transforms success, `AndThen`
composes wrapped operations, `OrElse` recovers within the wrapper, and `MapError`
changes the error type. Callback argument expressions evaluate eagerly; callback
bodies run only for the selected variant. Look up signatures with `fpas doc`.

## Typed errors and library boundaries

Declare an error family with `type InputError = error ... end error;`.
It is a closed nominal value type with enum-like variants and exhaustive patterns.
Each variant may have one nonempty literal message. Ordinary fields must be
Printable; a field named `Cause` must be a concrete error family or `E: Error`.
Families preserve generic arguments, support payload equality when possible,
and have no backing integers, ordering, methods or inheritance.

`Failure.Message()` renders the family, variant, fields and typed causes;
Console printing uses the same renderer. Use `.Message()` at a text boundary,
including `%s` formatting. Preserve typed errors while composing operations.
Application-authored string errors remain valid Result payloads.

`Std.Parse` parses text to Result values with `Parse.ParseError`. `Std.Conv`
converts and formats already typed values. Files, processes, JSON, TOML, tasks
and networking also have their own error families. To propagate into a different
family, use MapError before try; a one-field cause constructor can be the callback.

```pascal id=typed-failure-and-option run
program AgentErrors;

uses Std.Parse as Parse, Std.Test as Test;

type InputError = error
  Invalid(Cause: Parse.ParseError) = 'Invalid input';
  NonPositive(Value: integer) = 'Positive value required';
end error;

function Twice(Text: string): result of (integer, InputError);
begin
  const Value := try Parse.Int(Text).MapError(InputError.Invalid);
  if Value <= 0 then
    return Error(InputError.NonPositive(Value));
  end if;

  return Ok(Value * 2);
end function;

function Next(Value: option of integer): option of integer;
begin
  return Some((try Value) + 1);
end function;

begin
  Test.AssertEquals(42, Twice('21').Unwrap());
  Test.AssertTrue(Twice('invalid') is Error(InputError.Invalid(_)));
  case Twice('0') of
    when Ok(_):
      Test.Fail('invalid input was accepted');
    when Error(const Cause):
      Test.AssertTrue(Cause is InputError.NonPositive(0));
  end case;

  Test.AssertTrue(Next(None) = None);
  Test.AssertTrue(Next(Some(3)) = Some(4));
end.
```

Further rules: [patterns](../language/pattern-matching/README.md),
[error handling](../language/error-handling/README.md),
[native errors](../language/types/errors.md), and
[text parsing](../std/text/parse.md). The
[configuration example](../../../examples/pascal/errors/config_error.fpas)
shows real file errors, local validation and generic context wrappers.

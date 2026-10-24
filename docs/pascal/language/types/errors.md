# Native error families

An `error` declaration defines a nominal, closed family of ordinary values.
Return a concrete family in `Result of (T, E)` for an expected operational
failure. A returned error does not catch `panic`, failed contracts or runtime
failures. Existing Result payloads, including string errors, remain valid.

Formal syntax: [grammar](../../../specs/grammar.ebnf) (`error_type`,
`error_member`). Construction, aliases, qualification and exhaustive patterns
follow [data enums](enums.md). Write `end error;` after the final variant.

```pascal id=example-1 run
program ErrorValues;

uses Std.Test as Test;

type FsError = error
  Missing(Path: string) = 'File not found';
  Denied;
end error;

type ContextError of E: Error = error
  Failed(Cause: E; Operation: string) = 'Operation failed';
end error;

function Wrap<E: Error>(Cause: E): ContextError of E;
begin
  return ContextError.Failed(Cause, 'load');
end function;

begin
  const Failure := Wrap(FsError.Missing('settings.toml'));
  const Expected := 'ErrorValues.ContextError.Failed: Operation failed'#10 +
    '  Operation = "load"'#10 +
    'caused by ErrorValues.FsError.Missing: File not found'#10 +
    '  Path = "settings.toml"';
  Test.AssertEquals(Expected, Failure.Message());
  Test.AssertTrue(Failure is ContextError.Failed(FsError.Missing('settings.toml'), _));
end.
```

Declare at least one variant. An optional message is one nonempty string-literal
token; omit `= 'text'` when no label is needed. Messages have no interpolation.
Variants accept the same named or positional payload arguments as enums. They
have no integer backing values, defaults, methods or inheritance. Payload
constructors can be called directly or used as [ordinary function values](../functions/function-types.md#payload-constructors-as-functions), including with [MapError](result-operations.md). A fieldless variant is a value and has no `()` form.

Aliases preserve identity. Two declarations with identical fields remain
different types. Generic families retain all applied type arguments:
`ContextError of FsError` and `ContextError of AnotherError` are distinct runtime
instances. Shared generic routine bodies receive internal constructor values;
the source signature, argument labels and evaluation order do not change.

Equality is available when every payload supports equality. Error families never
support ordering or satisfy `Comparable`, including fieldless families. Matching
uses the same exhaustive `case` and nonbinding `is` rules as data enums.

## Payloads and typed causes

Every ordinary payload field must satisfy `Printable`. A generic payload carries
the appropriate constraint, for example `T: Printable`; the existing exclusions
for callables and distinct types remain unchanged. Printable aggregate contents
use their existing display behavior and perform no resource queries.

A field named `Cause`, case-insensitively, designates the original cause. Its
type must be a concrete error family or an `E: Error` parameter. A variant may
have at most one Cause. A leaf omits the field; string, Option, record and
unconstrained parameters cannot serve as Cause. Other fields remain ordinary
data even when their type happens to be an error family.

The cause remains the original typed value, available to nested patterns. It is
never replaced with a message or an erased root object. Construction does not
render text, capture a stack or perform I/O.

## Error constraint and output

`E: Error` accepts native error families and their aliases. It implies Printable,
but does not imply Comparable, arithmetic or compatibility between two error
families. Bare `Error` is not a storable type: use a concrete family or a
constrained type parameter in Result and other signatures.

| Operation | Behavior |
| --- | --- |
| `Value.Message(): string` | Complete automatic error text, including typed causes; read-only and without I/O |

Console.WriteText and WriteLn use the same renderer as Message(). Its deterministic
text has the defining qualified family and variant name, an optional literal
message, ordinary fields in declaration order and then `caused by` frames.
Fields use two-space indentation. Strings inside payloads are JSON-quoted and
escaped at every nesting level. Message controls are escaped too. A nested
error in an ordinary data field stays escaped field data. Separators are LF;
Message() has no trailing newline and WriteLn supplies its normal newline.
Import aliases do not affect the defining name.

Rendering and destruction traverse deep cause chains iteratively. Rendering is
proportional to the text produced and obeys normal string/allocation limits.
Repeated Message() calls do not mutate or cache state on the value. Message()
is permitted in contracts under the existing read-only native-operation rules;
its receiver must also be a permitted contract expression.

Printable does not imply conversion to string. The strict `%s` format still
requires a string: write `Failure.Message()` when building formatted text.
[`try`](../error-handling/try.md) retains compatible-type propagation; an Error
constraint does not add implicit conversion or wrapping.


## Migrating a Result boundary

Selected Std operations now return concrete error families. Match the variant
instead of interpreting message wording: channel closure is TaskError.Closed,
URI syntax is UriError.InvalidSyntax and an HTTP transport failure retains its
NetError inside HttpError.Transport. See the owning [Std pages](../../std/README.md)
for the complete cases and the operations that keep ordinary EOF or absence as
success. Existing application-authored string Result payloads remain valid.

To change the error family, apply MapError before try. A one-field cause variant
can be passed directly as a constructor value. Use an ordinary closure to add
operation or resource fields. At an intentional text boundary, explicitly map
Cause.Message(); this preserves the chosen string API. A local configuration
validation failure uses MissingSetting or InvalidSetting rather than a fabricated
file or host cause.

The [complete configuration example](../../../../examples/pascal/errors/config_error.fpas)
and its [compact-reference fence](../../agent-reference.md) load a real file,
retain the FsError, validate the port separately, match the original cause and
print the error directly. Its generic ContextError wrapper is used with both
ConfigError and ParseError; the two concrete families keep distinct identities.
Constructor values and captured closures use the same explicit error mapping.

Compiled libraries and standalone images from before native error metadata must
be rebuilt. The loader rejects unsupported formats and inconsistent native
contracts rather than guessing a string payload. Optional debugger types can be
stripped without removing the essential identities, payload types and Cause.

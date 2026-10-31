# Local variables

The code fragments use `uses Std.Math as Math;`.

Variables can be declared inline inside statement lists. Every binding starts
with its own `const` or `var` keyword. The same rule
applies in routines and nested scopes. Local type declarations are not permitted.
`const` bindings are immutable; `var` allows reassignment.
Prefer `const` for immutable bindings in application and library code.
See [Constants](constants.md) for runtime initialization and static classification.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_stmt` and `var_stmt` in statement position).

## Resource locals

`using const` declares an immutable local whose value is released when its
nearest lexical block exits. Its initializer runs once when the declaration is
reached. A failed acquisition does not register a resource. Resources acquired
in the same block are released in reverse order, including on early return,
`try` propagation, loop control, and failure. An already closed native handle
is skipped; an explicit close retains its normal result and error behavior.
An error or `None` being propagated remains the function result if release also
fails; the release failure is reported separately. For runtime failures, the
first failure remains primary.

```pascal id=resource-file
program ResourceFile;

uses Std.Io.Fs as Fs;

function ReadFirstLine(Path: string): result of (option of string, Fs.FsError);
begin
  using const File := try Fs.Open(Path, Fs.FileMode.Read);
  return Fs.ReadLine(File);
end function;

begin
  null;
end.
```

The enclosing function must support the error returned by `try`. Resource
bindings cannot escape through a return value, global, or escaping closure. For a custom
record resource, declare a [release procedure](../types/records.md#resource-release).
An alias that already existed before the `using` declaration remains valid;
native handle aliases observe the same closed resource after release.

The compiler recognizes these native resource types and release operations:

| Resource | Automatic release |
| --- | --- |
| [`Std.Io.Fs.File`](../../std/host/fs-streams.md) | `Fs.Close` |
| [`Std.Net.Sockets.Connection` and `Listener`](../../std/network/net.md) | `Sockets.Close` and `Sockets.CloseListener` |
| [`Std.Os.Proc.Process`](../../std/host/proc.md) | `Proc.Close` |
| [`Std.Ai.Acp.Connection`](../../std/ai/acp/connections.md) | `Acp.Release` |
| [`Std.Ai.Mcp.Server` and `Endpoint`](../../std/ai/mcp/README.md) | `Mcp.Release` and `Mcp.CloseEndpoint` |
| [`Std.Net.Http.BodyStream`](../../std/network/http.md#streaming-responses) | `Http.CloseStream` |

```pascal id=example-1 fragment context=declarations
function FullName(First: string; Last: string): string;
begin
  const Space: string := ' ';
  return First + Space + Last;
end function;
```

## Type inference

Explicit [subrange conversions](../types/subranges.md), such as
`const Level := Percent(Value);`, infer the range type and retain its checked
bounds. Routines returning a subrange infer the same declared range type.

In statement position, a `const` or `var` binding can omit its type annotation
whenever its initializer has a fully determined type. Calls and factories,
operators, references, field/index access, record updates, closures with complete
signatures and `if`/`case` expressions follow the same rule as literals,
constructors and conversions. The inferred type is fixed for the lifetime of
the binding. Explicit annotations remain valid and supply context when needed.

```pascal id=example-2 fragment context=declarations
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

Collection elements jointly determine their matching type; dictionary keys and
values infer independently. `[[], [1]]` determines `array of array of integer`,
and `[None, Some(1)]` determines `array of Option of integer`. An empty collection
or a collection whose elements still leave a type unknown requires context.
Mixed numeric literals such as `[1, 2.0]`
are invalid even with an annotation. Import `uses Std.Math as Math;` and convert explicitly,
for example `[Math.IntToReal(1), 2.0]`; inference does not choose a numeric promotion.
Expressions inside a collection literal can supply known element types.

Typed construction uses the resolved record type or enum variant, including
imported types and type aliases. Generic arguments must be fully determined by
supplied fields or payloads, or already fixed by a type alias. Record defaults
do not infer arguments. An uninstantiated generic routine payload does not
supply a complete type. A non-generic payloadless enum variant also provides
its type; a generic payloadless variant needs an annotation or a fixed alias.
See [Records](../types/records.md), [Enums](../types/enums.md), and
[constructor inference](../types/generics.md#generic-records).

```pascal id=example-3 fragment context=declarations
type Point = record
  X: integer;
  Y: integer;
end record;

type Box of T = record
  Value: T;
end record;

type Lookup of T = enum
  Found(Value: T);
  Missing;
end enum;

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

Generic calls infer from supplied values first; an expected result type fills
only unresolved parameters. For example, `const Value: Option of integer := Empty();`
determines T for `function Empty<T>(): Option of T`. Without context that call
is rejected. See [Generic routines](../functions/generic-routines.md).
[FP3036](../../tools/diagnostics.md#semantic-analysis) requests an annotation
when a local initializer still has an incomplete type.

```pascal id=example-4 fragment context=inferred-call
const Empty: array of integer := [];
const Mixed: array of real := [Math.IntToReal(1), 2.0];
const Converted := Math.IntToReal(42);
const Computed := ReadValue();
```

Declarations outside statement lists, including program and unit declarations
and routine declarations before `begin`, remain explicitly typed. Parameters,
routine results, and record fields also require annotations.

## See also

- [Variables](variables.md)
- [Runnable inference example](../../../../examples/pascal/basics/local_inference.fpas)

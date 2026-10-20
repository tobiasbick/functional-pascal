# AP47.1: Selected language and error contract

Status: Design recorded; runtime, language, constructor values and MapError are implemented through AP47.4.
Package and decisions: [AP47](README.md#decisions-and-authority).
Native error declarations, constraints, typed construction/matching and Message()
are implemented. Constructor values, MapError and library migration remain
planned in the later work packages.

## Declaration and identity

One declaration form introduces a nominal closed error family:

```pascal
type ConfigError = error
  FileFailure(Cause: Fs.FsError) = 'Cannot load configuration';
  MissingSetting(Name: string) = 'Required setting is missing';
  InvalidSetting(Name: string; Reason: string) = 'Invalid setting';
end error;
```

Choose `end error`, matching record/enum block terminators. Error is already a
reserved Result-constructor token; parse its declaration, constraint and
expression roles by context. No new global keyword is needed. `end type`,
`derives`, annotations and alternative spellings are rejected.

Planned grammar additions, owned here until the real grammar is updated:

```text
type_body += error_type
generic type_def alternatives += error_type
constraint_name += 'Error'
error_type = 'error' error_member { error_member } 'end' 'error'
error_member = identifier [ enum_fields ] [ '=' quoted_string ] ';'
```

At least one variant is required. Reuse enum payload syntax, qualification,
visibility, named/positional calls and source-order evaluation. No backing
integers, payload defaults, methods or inheritance clauses are allowed. Messages
are optional nonempty single string-literal tokens, not expressions, templates
or runtime callbacks. Empty labels are rejected with a hint to omit the label.
Existing literal escaping applies. This does not add interpolation.
Aliases preserve the family's identity; identical declarations remain nominally
distinct. A bare Error is not a storable root type and cannot erase any error.
`Result of (T, Error)` is rejected with a hint to use a concrete family or an
`E: Error` type parameter.

Error families support case/is and nested patterns exactly as payload enums do.
Cases are exhaustive and have no else arm. Equality follows payload equality;
ordering, integer backing values and Comparable are unavailable even for a
fieldless error family. Visibility and interface hashing follow nominal types.

## Generic errors and payloads

Use the existing declaration forms with one new built-in constraint:

```pascal
type ContextError of E: Error = error
  Failed(Operation: string; Cause: E) = 'Operation failed';
end error;
```

Error is satisfied by native error families, their aliases and parameters that
already guarantee Error. It implies Printable, but not Comparable, Numeric or
conversion to another family. It is a built-in capability, not a user trait or
a class. Generic arity, constructor inference and finite construction reuse the
existing record/enum rules. Each fully applied error family has a concrete runtime
identity, including all its type arguments; ordinary generic enum layouts retain
their existing erasure. Transparent aliases identify the same error instance.
For example, ContextError of Fs.FsError and ContextError of Parse.ParseError are
different instances even though they share a declaration and variant names.
The [runtime contract](02-runtime-and-artifacts.md#error-instance-identity-and-generic-construction)
owns instance keys and construction through erased generic routines. These are
compiler details and introduce no new type-argument syntax or reflection API.

Non-Cause payloads must satisfy the existing Printable capability, including
its current exclusions. Do not change unrelated types' Printable behavior in
AP47. Generic payload declarations carry sufficient constraints; instantiated
checks retain the concrete type. Shared handles, where Printable already permits
them, keep ordinary ownership and sharing rules and print only their existing
representation. Rendering performs no resource queries.

## Cause rule

A field named Cause, case-insensitively, is the sole designated original cause.
It must have a concrete error-family type or an Error-constrained parameter.
A string, Option, arbitrary record or unconstrained parameter in that role is a
static error. Existing duplicate-field checks prohibit two Cause fields.
A variant without a cause omits the field; there is no optional/untyped cause.
Other payloads are data, not automatically inferred causes from type or name.

Store the cause as the original typed value. Match it through ordinary patterns;
no message parsing, opaque box, downcast or runtime registration is involved.
Construction and wrapping do not print, format, capture a stack or read ambient
state. Multiple independent failures can remain ordinary payload data; only the
designated Cause participates in the automatic linear cause chain.

## Messages and output

An error value has one built-in operation:

```text
Value.Message(): string
```

It returns the complete automatic error text, including designated causes.
Console.Write/WriteLn and existing Printable consumers use this same renderer.
Strict string format specifiers stay strict; use Value.Message() for `%s`.
There is no second FormatError operation, custom Describe function or reporting
record. Message() is read-only, performs no I/O and follows the existing rules
for read-only native operations in contracts. It introduces no effects system.
It can be used in a generic routine with an Error-constrained receiver.

Render each frame as follows:

1. Header: canonical defining qualified type name plus variant name. If a message
   literal is present, append `: ` and that text. Imported aliases and runtime
   register/variant IDs do not change this identity.
2. Non-Cause fields follow in declaration order on indented `Name = VALUE` lines.
   Preserve field spelling from its declaration. In error field rendering, quote
   strings at every nesting level with JSON string escaping; reuse deterministic
   scalar/aggregate display traversal for the remaining structure. Escape embedded
   line breaks/control characters so one field occupies one physical line.
   Message control characters are escaped too. Ordinary non-error output is unchanged.
3. Render Cause next, prefixed by `caused by `, using the same frame rules. No
   duplicate payload line for Cause. A nested error used as ordinary data is
   escaped as field data, not promoted into the cause chain.
4. Use LF separators, no color, timestamp, implicit location or trailing newline.
   Console.WriteLn adds its normal final newline. Preserve present empty strings;
   there is no hidden optional context to conflate with absence.

```text
MyApp.ConfigError.FileFailure: Cannot load configuration
caused by Std.Fs.FsError.FileNotFound: File not found
  Path = "settings.toml"
caused by Std.Errors.HostError.Failure: Host operation failed
  Operation = "Std.Fs.ReadText"
  Resources = ["settings.toml"]
  Domain = None
  Code = None
  Detail = "Host supplied diagnostic"
```

The last frame illustrates the [native leaf](standard-errors.md#shared-native-leaf).
Its optional values use Option display with quoted string payloads. It does not assert a real
platform code. Applications can supply contextual fields in their own wrapper,
for example Operation or ConfigPath, when that context is useful.

Formatting walks stored values iteratively, including deep causes. It does not
silently omit frames. Text generation is proportional to rendered content and
uses normal string/allocation limits; no constant-cost or bounded-logging claim
is made. Repeated descriptions do not mutate or cache state on error values.

## Explicit propagation and constructor values

Keep the agreed MapError and existing try contracts:

```text
Value.MapError(F: function(Cause: E): E2): Result of (T, E2)
```

The receiver and callable expression are evaluated once in normal call order.
Constructing a callable may evaluate captures even on Ok. The mapper body is
invoked zero times for Ok and exactly once for Error. It returns an error payload,
not another Result; T remains unchanged. E/E2 need not themselves be native
error families because ordinary Result remains general.

Payload constructors are first-class routine values, consistently for errors
and enums. Their parameter labels/types are their payload labels/types. Receiving
callable signatures determine generic specialization. Fieldless variants stay
values, not zero-argument routines. No partial application is added.

```pascal
const Text := try Fs.ReadText(Path).MapError(ConfigError.FileFailure);
```

This works because FileFailure has one Cause parameter of the required type.
A wrapper with extra contextual parameters uses an ordinary closure capturing
those values. There is no automatic selection among wrapper variants.
Try extracts Ok or immediately returns Error through a compatible enclosing
error type. Option's Some/None propagation, exit/cleanup ordering and runtime
panic/contract behavior remain unchanged. A shared Error constraint does not
make distinct error families assignment-compatible or add implicit conversions.

## User-defined example and selection rationale

```pascal
type SettingsError = error
  Missing(Name: string) = 'Required setting is missing';
end error;

function RequireSetting(Name: string; Value: Option of string):
  Result of (string, SettingsError);
begin
  return case Value of
    when Some(const Text): Ok(Text);
    when None: Error(SettingsError.Missing(Name := Name));
  end case;
end function;
```

A caller handles SettingsError.Missing directly or prints Cause / Cause.Message().
It writes no reporting function. Adding a new variant updates exhaustiveness
checks and automatically participates in output using the declaration metadata.

Static labels plus automatic fields remove a second message-template language
and repeated reporter code. Explicit Cause preserves typed provenance without
exception inheritance. A constrained generic gives reusable error consumers
without erasure. Explicit MapError avoids ambiguous automatic wrapper selection.
The [Std catalog](standard-errors.md) uses these same language rules.

## Acceptance of the planning deliverable

The user authorized autonomous detail decisions for this plan; Q28-Q36 are
recorded in the package README and all dependent packages refer to this contract.
No blocking syntax/interface choices remain within the declared scope. The
runtime and language slices are delivered separately from constructor values,
MapError, Std migration and final acceptance.

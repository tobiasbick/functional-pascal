# AP43.1: Error conventions

Package: [AP43](README.md). Status: complete; agreed contract recorded.

This decision work package records the accepted API and authoring rules.
Implementation, migration, tests, and current reference changes belong to
[AP43.2](02-error-api-and-guidance.md), which is complete.
Examples using the selected parser names describe the implemented API.

## Parser ownership, names, and results

Import `Std.Parse` with an explicit alias, for example `uses Std.Parse as Parse;`.
The selected public operations have one `Text: string` parameter:

| Operation through alias Parse | Result | Input interpretation |
|-------------------------------|--------|----------------------|
| `Parse.Int(Text)` | `Result of (integer, string)` | Existing AP33 general integer grammar: signed decimal or `$` hexadecimal input. |
| `Parse.Real(Text)` | `Result of (real, string)` | Existing AP33 finite-real grammar and bounds. |
| `Parse.Bool(Text)` | `Result of (boolean, string)` | Existing Boolean grammar: surrounding whitespace and case-insensitive true/false. |
| `Parse.HexInt(Text)` | `Result of (integer, string)` | Existing explicit-hex grammar: bare hex or `$`/`0x`/`0X` prefix. |

All return Ok(Value) for valid input and Error(Message) for invalid input.
Retain string errors. Remove TryInt, TryReal, and TryBool; the Parse owner and
Result signature establish the parsing/failure convention without a Try prefix.
HexInt and Int have different input interpretations: bare `10` represents
sixteen for HexInt and ten for Int. Preserve that meaningful distinction.

Remove Std.Conv StrToInt, StrToReal, StrToBool, and HexToInt. Explicit Unwrap
on the selected Result-returning parser represents deliberate abort, for example
`Parse.Int(Text).Unwrap()`. Std.Conv retains formatting and conversion of already
typed values, including IntToStr, RealToStr, BoolToStr, IntToHex, and IntToReal.

Preserve AP33 grammar, trimming, separators, signed ranges, mathematical hex
magnitudes, finite-real checks, signed zero, subnormals, and formatting round-trips.
The new signatures retain AP34 inference and AP36's fully positional or fully
named arguments. Named calls use Text; migrate retired Conv parameter labels.

## Canonical handling roles and helper removals

| Purpose | Canonical form |
|---------|----------------|
| Propagate an error or absence from a compatible enclosing function | `try Expression` |
| Test a variant without extracting or binding data | `Value is Pattern` |
| Extract payloads, handle locally, or choose a plain replacement value | `case Value of ... end case` |
| Abort deliberately on Error or None | `Value.Unwrap()` |
| Transform a wrapped success/present value | `Value.Map(F)` |
| Compose another fallible/optional operation while keeping the wrapper | `Value.AndThen(F)` |
| Recover within a Result/Option wrapper | `Value.OrElse(F)` |

Remove native Option IsSome/IsNone and Result IsOk/IsError. Their replacements
are `Value is Some(_)`, `Value is None`, `Value is Ok(_)`, and `Value is Error(_)`.
AP35's is tests never introduce names. Payload bindings belong to case arms;
diagnostics and examples teach case instead of a predicate followed by Unwrap.
Retain AP39's exhaustive case rules and AP36's positional-or-named patterns.

Remove Option and Result UnwrapOr. Case is the standard form for choosing an
unwrapped value or default. Retain Map, AndThen, OrElse, and Unwrap with their
current signatures, typing, callback order, failure behavior, and availability.
OrElse keeps the wrapper: its callback runs only on Error/None. Ordinary
receiver and callback-argument expression evaluation remains eager; the
callback body is conditional. Unwrap remains an explicit runtime-failure boundary.

These roles define the canonical authoring and diagnostic guidance. General
case expressions and the retained functional combinators keep their contracts;
the selected removals are the named parser wrappers and native helpers above.

## Existing try compatibility matrix

This is a documentation and verification completion of the current propagation
contract; its syntax and semantics remain unchanged.

| Operand | Enclosing function return type | Outcome |
|---------|--------------------------------|---------|
| `Result of (T, E)` | `Result of (U, F)` with compatible E/F | Unwrap Ok(T); return Error(E) immediately on failure. T and U may differ. |
| `Result of (T, E)` | Result with an incompatible error type | Reject statically. |
| `Option of T` | `Option of U` | Unwrap Some(T); return None immediately on absence. T and U may differ. |
| Result | Option | Reject statically. |
| Option | Result | Reject statically. |
| Result or Option | Ordinary value-returning function, procedure, or program body | Reject statically. |
| Ordinary value | Any context | Reject: try requires Result or Option. |

Evaluate the operand once. Successful unwrapping supplies the operand's payload
type; using that value still requires ordinary type compatibility. Failure exits
the enclosing function immediately under its existing return/cleanup/contract
rules. Preserve aliases, distinct identity, generic constraints, and AP33 numeric
compatibility. Panic and other runtime failures are not caught by try and are
not converted to Error or None. Expected absence uses Option; recoverable errors
use Result.

## Fallback evaluation and migration

New case fallback work is evaluated only in the selected arm. Removing UnwrapOr
does not authorize changing existing callers' effects: the retired helper's Default
argument was evaluated even on Some/Ok. Migration preserves receiver-first order,
single evaluation, and eager default work within the original control-flow path.
For an effectful old `MakeValue().UnwrapOr(MakeDefault())`, use local receiver and
default bindings in that order before case; select the saved default on Error/None.
Keep those bindings inside the original branch, short-circuit operand, loop
iteration, closure invocation, or initialization boundary. A panicking old default
must still abort even when the wrapped value is successful. Pure literal defaults
can appear directly in their case arm.

Aborting parser consumers become parser-plus-Unwrap consumers; recoverable
consumers keep Result handling. Use resolved operations when migrating, preserving
same-name user methods and unrelated routines. Preserve callback invocation and
payload evaluation through the retained combinators. Single evaluation applies
to each original expression occurrence: repeated effectful calls in an old
predicate/access sequence must retain their original count and conditional order.

## Replaced decisions and remaining questions

AP43 replaces the selected AP06 catalog entries, AP21's predicate/default helper
surface and guidance, and AP33's retained parser names and aborting-wrapper API
boundary. Their other contracts and completed status remain intact. AP35's
nonbinding is, AP39's exhaustive case, AP34 inference, AP36 argument styles,
AP41 qualification, and AP42 Option-valued index searches remain authoritative.

No selected AP43 language decisions remain open. Current `docs/pascal` pages
describe this implemented contract and its canonical authoring forms.

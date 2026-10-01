# Stage 6: Domain types and contracts

Prerequisites: [functional core](04-functional-core.md) and
[purity/effects](05-effects-and-tasks.md). These guarantees extend the common
type/call/error rules instead of introducing a second programming model.

## Distinct types

Declare `type UserId = distinct integer;`. An ordinary alias remains
`type UserId = integer;`. Distinct types introduce nominal identity; aliases do
not. `distinct` is the only spelling, with no repeated `type` alternative.

Construction and unwrapping use explicit checked type conversions: `UserId(42)`
and `integer(Id)`. The value must match the declared underlying type; no chaining
of implicit conversions across distinct identities. Same-distinct-type equality
and ordering are available exactly when the underlying type supports them.
Arithmetic, member lookup, and resource operations are not automatically inherited.
Use ordinary unit routines for domain operations. Wrapping a handle does not remove
its resource identity, task lifetime, transfer, or purity restrictions.

Representation hiding remains a record with non-public fields and ordinary public
factories; no second hidden-type declaration form is added. Distinctness does not
validate a URL, path, or business identifier by itself.

## Integer subranges

Declare `type Percent = 0..100;` using compile-time integer bounds, lower <= upper.
The base is signed 64-bit integer; there is no explicit base annotation or enum
subrange syntax. Degenerate one-value ranges are valid. Bounds use the same static
constant rules as other compile-time contexts.

A subrange widens implicitly to integer. Arithmetic produces integer and retains
checked-overflow behavior. Narrowing in assignments, calls, or returns always
requires an explicit `Percent(Value)` conversion. An invalid statically known
value is a compile-time error; a dynamic violation panics with value and bounds.
Separately named subranges with the same interval do not create domain identity;
use `distinct` when separate nominal identities are wanted.

`Value in Percent` tests membership without conversion or failure. `in` already
represents membership; its right operand resolves to a subrange type or a
collection value, without inventing a second range-check keyword. Successful
membership does not silently insert a narrowing conversion. Expected input
validation may return Option/Result through an ordinary function.

## Routine contracts

Use `requires` and `ensures` on routine definitions, between the heading/signature
and declarations/body. Named and anonymous functions/procedures use the same
clauses; each clause ends with `;`. In an anonymous expression, clauses follow its
signature directly, without adding a declaration terminator to that signature.
Contracts belong to the definition, not its callable type, and remain checked on
indirect invocation. Anonymous failures identify the definition's source location.

```pascal
pure function Clamp(Value: integer; Lower: integer; Upper: integer): integer;
requires Lower <= Upper;
ensures (Clamped) Clamped >= Lower and Clamped <= Upper;
begin
  return if Value < Lower then Lower
    elsif Value > Upper then Upper
    else Value
  end if;
end function;
```

Each clause is a pure boolean expression, checked by the same purity rules as a
pure function body. It may call verified pure routines. Preconditions observe
entry parameter values; postconditions observe their exit values. Each clause
receives read-only value snapshots of the referenced parameters, including var
parameters; this does not permit reading unrelated external mutable state or
observing resource state. These are current clause inputs, not `old` snapshots.
A function
postcondition may bind its returned value with `ensures (Name) Expression;`.
That immutable name is clause-local; it is unavailable to the body and other
clauses. Procedures have no returned-value binding.

Repeated clauses evaluate in written order with short-circuit conjunction.
Check requires before executing the body and ensures on every normal return,
including explicit or try-propagated Result.Error returns. A panic or cancellation
is not a normal return and does not run ensures. Evaluate the return expression
once and finish scope cleanup before ensures; a cleanup panic cancels the return.

Contracts always execute, including release builds. Failure panics with routine,
clause, source location, and bounded diagnostic renderings of relevant parameter
values. Diagnostic rendering must not call user functions or inspect external
resource state. Unsupported values use type-aware summaries. A contract violation
does not become a Result error or roll back prior mutation. No `old` snapshots,
automatic theorem proving, or mode that disables checks are introduced.

## Work and acceptance

- [ ] Implement distinct identity and explicit conversions with positive and
  negative tests for aliases, swapped domain arguments, operations, and wrappers.
- [ ] Implement subrange typing, static bounds, membership, checked conversion,
  widening, and integer arithmetic across constants and runtime values.
- [ ] Implement contract binding/type checks on named and anonymous routines,
  entry/return instrumentation, indirect calls, panic reporting, and always-on
  execution through the runner.
- [ ] Update current type, operator, function, and error-handling documentation
  and grammar only as these features land; add complete runnable examples.
- [ ] Test endpoints, adjacent invalid values, singleton/empty ranges, overflow,
  missing annotations, private representations, and nested wrapped resources.
- [ ] Test repeated/false/panicking clauses, all normal/error returns, scope
  cleanup interactions, result-name visibility, var-parameter exit state, and
  identical debug/release contract enforcement.

Acceptance: identities cannot be interchanged implicitly, range restrictions
cannot be bypassed, and contracts use the existing purity and failure model on
every relevant call/return path. These features add no implicit validation policy.

Owners: parser, sema, compiler/IR/bytecode, VM, diagnostics, formatter, and editor
type/signature consumers; current handbook types/functions/error-handling pages.

Status: target rules specified; implementation pending the functional/effect core.
Next: implement distinct identity first, subranges second, then contracts using
the already verified purity checker and scope cleanup path.

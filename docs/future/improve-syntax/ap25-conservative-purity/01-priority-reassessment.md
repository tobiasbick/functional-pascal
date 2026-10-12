# AP25.1: Priority reassessment

Package: [AP25: Conservative purity](README.md)

Status: complete. The user decided to close AP25 without implementation.
This decision work package changes no language or application code.

## Scope

Assess AP25 against AP23 usage, concrete predicate reuse, and checker costs.
The accepted outcome is closure; evidence and limitations are retained below.

## Evidence boundary

AP23 is complete. Its tracked use is in regression tests; the isolated geometry
pilot supplies additional practical evidence without production adoption.
This limited evidence supports closure, not an implementation claim.

## Assessment

AP23 is implemented and exercised by regression tests and the isolated geometry
pilot below, but adoption in shipped application/library code is not established.
Contract declarations in the inspected tracked FPAS
sources under `lib/`, `apps/`, `examples/`, and `tests/` occur only in
[`preconditions_test.fpas`](../../../../tests/runner/contracts/preconditions_test.fpas)
and [`postconditions_test.fpas`](../../../../tests/runner/contracts/postconditions_test.fpas).
Rust-embedded fixtures provide broader compiler and runtime coverage; they do
not demonstrate application demand. The practical-use prerequisite therefore
remains only partially satisfied. The pilot demonstrates applicability without
claiming production adoption.

### Existing protection and limitations

The current [contract rules](../../../pascal/language/functions/contracts.md)
already reject ordinary calls, mutable global/capture reads, tasks, reference
arguments, and unlisted native operations. The checks use resolved identities,
so a user method named `Length` cannot inherit permission from string `Length`.
The [semantic regression tests](../../../../crates/fpas-sema/src/tests/decl/contracts.rs)
cover these restrictions, including rejection of an ordinary `Read()` function
whose body simply returns a constant.

Purity would allow reusable checked predicates in contracts; it is not needed
to make the existing restricted contract expressions safe. No concrete defect
in the inspected consumers was identified that an AP25 checker would have
prevented. Rejected test fixtures demonstrate protections, not production bugs.

### Concrete candidates

These are possible future contract uses, not existing clauses or verified pure
APIs. None authorizes changing a routine's failure behavior.

| Candidate | Benefit of checked calls | Current alternative and limits |
|-----------|--------------------------|--------------------------------|
| [`TuiRect.Contains`, `Intersects`, and edge helpers](../../../../lib/Std/Tui/Geometry/Rect.fpas) | Reuse geometry predicates in internal entry/return checks instead of duplicating edge arithmetic. Scalar/record inputs make this a bounded pilot. | Inline field/arithmetic conditions are already possible; existing constructors check negative extents and overflow explicitly. Calling helpers would require checking their entire call chain. |
| [`Std.Http.Method.IsValidToken`](../../../../lib/Std/Http/Method.fpas) and [`Std.Http.Fields.IsValidValue`](../../../../lib/Std/Http/Fields.fpas) | Reuse loop-based validation that cannot be written as an ordinary contract expression. | Keep validation in the body and return `Error` for malformed external input. Replacing those errors with contract panics would change behavior and is inappropriate. Checked purity would also need an audit of native character operations. |
| [`Std.Http.Headers.ContainsUnsafeText`](../../../../lib/Std/Http/Headers.fpas) | Give a reusable name to a safety predicate. | Its two native `Contains` checks can already be written inline. This alone is a weak justification for a new checker. `IsName` additionally needs audited `ToLower` calls. |
| [`ServerLoop.HasCapacity`](../../../../lib/Std/Http/ServerLoop.fpas) | Share a scalar configuration predicate between internal checks and orchestration. | The expression is short enough to inline. `ValidateOptions` returns structured errors; preserving that behavior takes precedence over adopting contracts. |

### Cost and unresolved boundaries

The proposed user checker and standard-library audit would both be required
for useful adoption. Annotating a top-level helper alone is insufficient: transitive calls,
native operations, imported interfaces, callable types, captures, and allowed
local mutation must carry an enforceable guarantee.

An immutable binding to a cancellation token, channel, connection, or other
shared handle does not make observations of its state pure. The aliasing and
imported-API proof decisions remain unresolved. Geometry's locally mutated
scalar temporaries offer a simpler first case than shared handles or callbacks.

The debugger's [detached call sandbox](../../../../crates/fpas-vm/src/vm/debug/calls/mod.rs)
is not an AP25 proof: it isolates state and can discard writes. Contracts run
in the ordinary program and cannot rely on that isolation to permit calls.

### Geometry pilot

An isolated program wraps the actual exported `Std.Tui.TuiRect.Create` and
`FromEdges` routines; no library implementation or error behavior is changed.
It uses current syntax, for example:

```pascal
function FromEdgesChecked(Left: integer; Top: integer; Right: integer; Bottom: integer): TuiRect;
ensures (Bounds) Bounds.X = Left and Bounds.Y = Top;
ensures (Bounds) Bounds.X + Bounds.Width = Right and Bounds.Y + Bounds.Height = Bottom;
begin
  return TuiRect.FromEdges(Left, Top, Right, Bottom);
end function;
```

The corresponding `Create` wrapper checks that all returned fields equal their
input values and that both dimensions are non-negative. Execution covers an
ordinary rectangle, zero dimensions with negative coordinates, edges crossing
zero, and a right edge at the maximum integer. Independently specified expected
dimensions and edges are asserted outside the clauses.

Negative controls establish that the checks execute and failures have the
intended cause:

- A postcondition expecting width nine from a rectangle created with width eight
  produces FP5010 identifying the failed `ensures` clause and returned record.
- Negative input width still produces the constructor's original
  `Tui width must not be negative` panic; the wrapper does not replace it with
  an entry contract or run a postcondition on that failure.
- Replacing the right-edge arithmetic with `Bounds.Right()` produces FP3037
  for an ordinary method call in the clause. The same method works in the body
  and in assertions outside the clause, so this is a contract restriction,
  not a missing method or inaccessible unit.

The pilot confirms that basic constructor invariants fit AP23 today. Reusing
the checked edge helper would remove arithmetic duplication, but contracts
can already express this small invariant without loops or native-library
extensions. The experiment does not establish a prevented production defect,
acceptable overhead for broad adoption, or permission to mark geometry methods
pure. It also does not justify adding redundant wrappers to the shipped API.

### Accepted decision

Close AP25 without implementation. The geometry pilot confirms that useful
basic invariants fit AP23 and identifies only a small duplication cost. The
user accepted that this benefit does not justify the checker and maintenance
burden. Q18's earlier low-priority retention is superseded by this decision.

AP25.2 and AP25.3 are removed. The assessment and pilot rationale remain;
there is no scheduled implementation or additional adoption pilot. Reopening
would require stronger evidence from actual internal contract adoption and a
new explicit user decision.

## Delivery

- Assessed contract usage, reusable predicates, and existing protections.
- Exercised geometry constructor postconditions and negative controls.
- Recorded closure in the package README and central plan.

## Affected areas

- `docs/future/improve-syntax/ap25-conservative-purity/`, central README status.

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- The package records closure and marks AP25.1 complete; the central plan uses
  `—` for AP25 rather than claiming implementation completion.
- AP25.2 and AP25.3 are removed; no active plan promises purity implementation.

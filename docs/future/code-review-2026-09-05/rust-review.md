# Rust correctness review

**Historical review:** the descriptions and source line numbers below refer to
the original revision. See [fixes and verification](fixes.md) for the authorized
implementation follow-up and current evidence.

Revision and verification limits: [review overview](README.md). All findings are
open, P2, and supported by static inspection. No fixes or executable reproductions
were performed. Runtime cost findings are in [the performance report](performance-review.md).

## R01 — Closure discovery skips the repeat-until condition

**Location:** [closures/discover.rs](../../../crates/fpas-compiler/src/lowering/closures/discover.rs), lines 53–57.

The discovery visitor groups `Stmt::Repeat { body: statements, .. }` with a plain
block and visits only the statements. It never visits the `condition` expression.
In contrast, `while` and `if` explicitly visit their conditions.

The semantic analyzer does check the repeat condition
([loops.rs](../../../crates/fpas-sema/src/check/stmt/control_flow/loops.rs), lines
121–145). Later, [control_flow.rs](../../../crates/fpas-compiler/src/lowering/control_flow.rs)
lowers that condition, and [expr.rs](../../../crates/fpas-compiler/src/lowering/expr.rs)
requires each anonymous closure to have a registered target. A closure appearing
only in `until` therefore reaches the `unregistered closure` error path despite
being a supported expression.

**Trigger to reproduce:** pass an anonymous Boolean function as an argument in an
`until` expression. This minimal proposed regression has not been run:

```pascal
program RepeatClosure;

function Evaluate(Predicate: function(): boolean): boolean;
begin
  return Predicate();
end;

begin
  repeat
    begin
    end
  until Evaluate(function(): boolean begin return true; end);
end.
```

**Expected:** compile and exit after one iteration. **Predicted actual:** lowering
fails when resolving the anonymous closure target.

**Contract:** [closures](../../pascal/language/functions/closures.md) allows closures
as arguments; [repeat-until](../../pascal/language/control-flow/while-repeat.md)
uses a Boolean expression in the enclosing scope. No language change is needed.

**Fix direction:** handle repeat separately and visit both body and condition.
Test a plain closure, an outer mutable capture, and a bound-method value in the
condition; this visitor registers bound methods as well. Preserve the existing
enclosing-scope rule. The inspected closure and repeat tests do not cover this
combination.

## R02 — Real-to-integer conversion accepts positive 2^63

**Location:** [math.rs](../../../crates/fpas-std/src/math.rs), lines 207–221.

`checked_real_to_int` checks `value > i64::MAX as f64`. The floating representation
of `i64::MAX` rounds to exactly 2^63, so that comparison accepts
`9223372036854775808.0`. The subsequent cast saturates to `9223372036854775807`.
This silently returns an incorrect integer instead of the range diagnostic used
for larger inputs.

**Trigger:** `Floor(9223372036854775808.0)`. `Ceil`, `Round`, and `Trunc` use the same
helper and have the same boundary problem. This conclusion follows from the
numeric bound and conversion path; no FPAS invocation was run.

**Contract:** [Std.Math](../../pascal/std/numeric/math.md) specifies these rounding
operations. The implementation's own range hint explicitly ends at
`9223372036854775807`; the accepted input cannot produce a representable result.
Existing Rust tests around lines 336–361 cover infinity and very large finite
inputs, not the first overflowing positive floating value.

**Fix direction:** use an exclusive upper bound of 2^63 and an inclusive lower
bound of −2^63. Preserve the current numeric-domain error policy. Clarify these
error boundaries on the Std.Math page when implementing.

**Regression:** all four intrinsics must reject 2^63, accept its immediately
preceding representable floating value, accept −2^63, and reject the representable
value immediately below −2^63. Retain NaN/infinity and ordinary rounding coverage.

## R03 — Outgoing TLS handshake is not interrupted by VM cancellation

**Primary location:** [connections.rs](../../../crates/fpas-vm/src/vm/hosted/net/connections.rs), lines 94–106.

`connect_tls` establishes TCP and calls `client::connect` before inserting the
transport into the VM connection registry. The client blocks in `complete_io`
([tls/client.rs](../../../crates/fpas-vm/src/vm/hosted/net/tls/client.rs), lines
44–50). Registry shutdown only drains and interrupts already registered sockets
(`connections.rs`, lines 178–189). VM shutdown then joins task workers
([vm/mod.rs](../../../crates/fpas-vm/src/vm/mod.rs), lines 293–296).

**Trigger:** a task connects to a TCP peer that accepts the socket but never
answers the TLS ClientHello; cancel the VM while the handshake is pending.

**Effect:** the pending socket is invisible to cancellation. Worker joining can
wait for the handshake timeout instead of completing promptly. The API permits
300,000 ms, so a silent peer can cause a delay of approximately five minutes.
This is a source-derived timing risk, not a measured delay in this review.

**Contract:** [Std.Net](../../pascal/std/network/net.md), lines 55–58, promises
interruption of all blocking network operations before task workers are joined.
Tests for established connections and incoming handshakes do not establish that
outgoing setup meets this contract.

**Fix direction:** make the pending socket interruptible by the VM before entering
the handshake, with explicit transfer/removal on success or failure and correct
handling of cancellation racing registration. Do not merely shorten the timeout.

**Regression:** synchronize a local listener on receipt of handshake traffic,
cancel the client VM, and require worker termination well before the configured
timeout. Give the fixture an independent way to close the peer socket even if the
assertion fails. Cover shutdown racing registration and normal successful TLS
setup. This finding does not claim that DNS or TCP connection establishment was
proven cancellation-safe; those phases need separate verification.

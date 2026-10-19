# AP47.4: Constructor routine values and MapError

Status: Planned; implementation not authorized. Depends on AP47.3.
Contracts: [AP47.1](01-decision.md#explicit-propagation-and-constructor-values)
and [AP47 decisions](README.md#decisions-and-authority).
Completion is tracked only in the package README.

## Deliverable

Implement the agreed Result operation and permit payload variant constructors
as ordinary callable values. Apply the same constructor rule to errors and
ordinary enums; do not make an error-only callback exception.

```text
Value.MapError(F: function(Cause: E): E2): Result of (T, E2)
```

For Ok preserve its success value and do not invoke F. For Error call F once and
wrap its returned payload in Error. Evaluate receiver and callable expression
once in normal order; constructing captures can have ordinary evaluation effects
even if the body is not invoked. E/E2 remain unconstrained for general Result
use. MapError does not recover to Ok, flatten a Result returned as an error
payload, catch runtime failure or implicitly format/cast error types.

## Constructor contract

A payload constructor such as ConfigError.FileFailure has its declaration's
arity, readonly parameter labels/types and result family. In a receiving
callable context it is represented using ordinary noncapturing function values
and the shared concrete constructor factory from AP47.3, keyed by error instance
and variant according to AP47.2. Ordinary enum constructors retain their erased
layouts and use the same ordinary function-value mechanism. Generic wrappers
and callable adapters carry any hidden factories under AP47.2's calling contract;
MapError does not reconstruct an error instance from the callback value's display
name. Do not allocate a closure environment per call or add constructor opcodes.
Generic receiving signatures infer every required type argument, otherwise the
existing ambiguity diagnostic applies. Passing an imported alias retains the
original declaration and visibility. Direct constructor calls retain their field
labels; passing/assigning the constructor follows the same receiving-callable
label rules as an ordinary routine value, without a new label-conversion rule.

Fieldless variants remain values and have no `()` form. Payload constructors
with several parameters require a matching callable; no currying, defaulted
arguments or automatic capture is added. A wrapper requiring extra application
context uses a normal anonymous function that captures those values.

## Actual owners and planned edits

| Path | Responsibility |
| --- | --- |
| `crates/fpas-sema/src/check/expr/`, `src/types/` | Resolve variant paths as constructor callables under an expected signature |
| `crates/fpas-sema/src/std_registry/native/result.rs` | One MapError entry and documentation |
| `crates/fpas-sema/src/std_registry/builtins/result_option.rs` | Generic receiver/callback/result substitution |
| `crates/fpas-compiler/src/lowering/`, `src/bytecode/` | Shared concrete constructor bodies and ordinary function-value metadata |
| `crates/fpas-bytecode/src/intrinsic/` | ResultIntrinsic::MapError ID and portable validation |
| `crates/fpas-vm/src/vm/hosted/callbacks/` | Sync and resumable MapError routing and ResultError callback wrapping |
| `crates/fpas-vm/src/vm/debug/scheduling/effects.rs` | Classify the new callback intrinsic consistently with existing Result operations |
| `docs/pascal/language/types/result-operations.md`, callable documentation | Publish delivered semantics and constructor use |

The existing callback plan has Ok passthrough paths and SingleWrapper::ResultOk.
Extend the shared planning/wrapping mechanism for ResultError instead of adding
a second callback execution loop. Scheduled callbacks may suspend and resume;
main-task callbacks follow the existing synchronous route. Preserve current
task/scope ownership, trace observation and failure-capture rules.

## Acceptance and verification

One source owner verifies a side-effect counter: receiver/callable construction
once, mapper body zero times on Ok and exactly once on Error. A returned Result
as E2 stays nested rather than being flattened. Cover unrelated success/error
types, string-to-custom and custom-to-custom mappings and nominally distinct
errors. Use the constructor directly, an imported constructor and a captured
ordinary closure. An enum constructor demonstrates the shared callable rule.
Use two instantiations of a generic error constructor to prove MapError retains
each concrete owner/Cause identity and reuses the matching shared factory.

Separate lifecycle fixtures cover a scheduled mapper that suspends, nested
callbacks, cancellation, panic/contract failure and trace bounds. Extend existing
Result callback fixtures for these distinct routes rather than replaying all
scalar cases at every layer. Incompatible callback arity/labels/types and
uninferred generic arguments must fail at checking, not runtime.

Try rejection for incompatible E/E2 is still observable without explicit mapping;
mapping before try succeeds. Option and OrElse retain their contracts. Source,
cached-unit and program-image constructor calls agree. Run relevant compiler,
sema, VM/debugger and CLI checks plus existing VM/concurrency benchmarks. Public
examples and native operation catalogs must describe the same operation.

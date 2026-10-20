# AP47.4: Constructor routine values and MapError

Status: Complete. Depends on completed AP47.3.
Contracts: [AP47.1](01-decision.md#explicit-propagation-and-constructor-values)
and [AP47 decisions](README.md#decisions-and-authority).
Completion is tracked only in the package README.

## Deliverable

Result error mapping and payload variant constructor values are implemented.
Native errors and ordinary enums use the same readonly callable rule.

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

## Implementation owners

| Path | Responsibility |
| --- | --- |
| `crates/fpas-sema/src/check/expr/`, `src/types/` | Resolve variant paths as constructor callables under an expected signature |
| `crates/fpas-sema/src/std_registry/native/result.rs` | One MapError entry and documentation |
| `crates/fpas-sema/src/std_registry/builtins/result_option.rs` | Generic receiver/callback/result substitution |
| `crates/fpas-compiler/src/lowering/closures/constructor_values.rs`, `closures/discover.rs`, `closures/mod.rs`, `root.rs`, `calls.rs` | Shared ordinary enum constructor bodies and callable discovery |
| `crates/fpas-compiler/src/lowering/errors/mod.rs`, `errors/rewrite.rs` | Concrete error factories and forwarding hidden generic factories without per-reference adapter environments |
| `crates/fpas-bytecode/src/intrinsic/` | ResultIntrinsic::MapError ID and portable validation |
| `crates/fpas-vm/src/vm/hosted/callbacks/` | Sync and resumable MapError routing and ResultError callback wrapping |
| `crates/fpas-vm/src/vm/debug/scheduling/effects.rs` | Classify the new callback intrinsic consistently with existing Result operations |
| `docs/pascal/language/types/result-operations.md`, callable documentation | Publish delivered semantics and constructor use |

The shared callback plan includes Ok passthrough and `SingleWrapper::ResultError`.
Sync and scheduled callbacks use their existing execution loops. Scheduled mappers
suspend and resume with their owner; task/scope ownership, trace observations and
failure capture retain their existing rules. Constructor requests remain in their
lexical generic-parameter universe. Shared bodies distinguish semantic native-error
instances even when their IR signatures erase to the same types. Local and global
callable bindings named like a variant follow ordinary function-value dispatch.

## Regression coverage

The Result source fixture verifies a side-effect counter: receiver/callable construction
once, mapper body zero times on Ok and exactly once on Error. A returned Result
as E2 stays nested rather than being flattened. It covers unrelated success/error
types, string-to-custom and custom-to-custom mappings and nominally distinct
errors. Direct and imported constructors and captured ordinary closures are exercised. An enum constructor demonstrates the shared callable rule.
Distinct instantiations in one generic owner, reversed generic arguments and
separate generic calls retain each concrete owner/Cause identity and reuse the
matching shared factory.

Separate lifecycle fixtures cover a scheduled mapper that suspends, nested
callbacks, cancellation, panic/contract failure and trace bounds. The CLI and concurrency fixtures exercise these distinct routes without
replaying every scalar case at each layer. Incompatible callback arity/labels/types and
uninferred generic arguments must fail at checking, not runtime.

Try rejection for incompatible E/E2 is still observable without explicit mapping;
mapping before try succeeds. Option and OrElse retain their contracts. Source,
cached-unit and program-image constructor calls agree. Compiler, sema, VM/debugger, CLI, native catalog and public examples cover the
same operation.

Additional integration owners are `crates/fpas-std/src/std_units/symbols/std_symbols/result.rs`,
`crates/fpas-compiler/src/intrinsic_catalog.rs`, bytecode execution/debug-effect
classification and the existing native catalog export. Regression owners are
`tests/stdlib/result/map_error_test.fpas`, `crates/fpas-sema/src/tests/decl/errors.rs`,
`tests/concurrency/map_error_callbacks_test.fpas`,
`crates/fpas-language-service/tests/intellisense/error_conventions.rs` and
`crates/fpas-cli/tests/error_mapping.rs`. Public documentation also includes
`docs/pascal/language/functions/function-types.md`, `types/errors.md` and the compact
`docs/pascal/agent-reference.md`. No application or Std signature migration belongs
to this work package.


## Remaining boundaries

Std signatures and their consumers remain the responsibility of AP47.5/AP47.6;
no migration or parallel compatibility API is included here. Independent checkout
follow-ups remain in imported-generic-enum debug projections, NotesUpdate reference
counts, Linux socket fixture assumptions and existing source-format differences.
Generic record callable defaults also retain an existing limitation reproduced
with ordinary generic routine values. Verification evidence belongs in the
delivery report; these do not change AP47.4's selected contract.

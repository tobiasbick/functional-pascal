# `fpas-bytecode` review follow-up

Classification: bytecode format and validation. Fixes should tighten internal artifact validation without changing FPAS semantics.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| BYTECODE-01 | P1 | `crates/fpas-bytecode/src/executable.rs:64-65` | Validation treats the presence of any `Halt` as proof that initialization can terminate. Unreachable `Halt` and infinite entry loops pass. | Traverse control flow from entry zero, distinguish function regions, and define the required terminating paths. | `[Jump(0), Halt]`, unreachable `Halt`, fallthrough, and valid initialization graphs. |
| BYTECODE-02 | P1 | `crates/fpas-bytecode/src/value/mod.rs:163-165` | Formatting a self-referential Cell locks the same non-reentrant mutex recursively and deadlocks printing/diagnostics. | Format Cells opaquely or use cycle-aware/reentrant-safe traversal such as `try_lock` with a stable placeholder. | Construct a safe self-cycle and assert formatting terminates with deterministic output. |
| BYTECODE-03 | P2 | `crates/fpas-bytecode/src/value/equal.rs:4-9`, `src/chunk.rs:164-167` | Constant deduplication merges `+0.0` and `-0.0`, although the sign is observable and persistent values retain exact bits. | Separate runtime equality from constant identity; compare real constants by bits for pool deduplication. | Both signed zeros must receive distinct indices and preserve bits through serialization. |
| BYTECODE-04 | P2 | `crates/fpas-bytecode/src/executable.rs:87-99` | Name-bearing opcodes validate only constant index bounds, not that the constant is a string. Invalid images fail only in the VM. | Add opcode-specific constant type checks and a precise validation error. | Negative tests for Call, Global, Record/Field, and Enum operations. |
| BYTECODE-05 | P2 | `crates/fpas-bytecode/src/executable.rs:68-76,91-95` | Calls and closures are not checked against the function table or declared arity. | Resolve referenced function names during validation and check `Call` arity. | Unknown call/closure targets and arity mismatch must fail validation. |

## Implementation notes

Define the validation ownership boundary together with `fpas-unit`, `fpas-linker`, `fpas-program`, and `fpas-vm`. A linked or decoded executable should not require downstream consumers to rediscover structural corruption.

Additional gaps: complete `PersistentValue` roundtrips and rejection paths, exact real-bit preservation, captur­ing-function rejection, and a golden compatibility test for the serialized `Op` wire representation.

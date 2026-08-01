# `fpas-unit` review follow-up

Classification: compiled Unit format, compatibility, validation, and sidecar persistence. No language change expected.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| UNIT-01 | P1 | `crates/fpas-unit/src/sidecar/mod.rs:25-38`, `src/format/mod.rs:18-20` | `fs::read` allocates the complete `.fpascu` before payload limits apply. A huge source-adjacent sidecar can OOM check/run/test. | Check file metadata/envelope size before allocation or decode through a bounded reader. Bound public direct decoders as well. | Oversized whole file is rejected before allocation; exact and one-over payload/resource limits. |
| UNIT-02 | P2 | `crates/fpas-unit/src/sidecar/atomic.rs:28-40,92-100` | Any lock older than ten seconds is reclaimed without owner liveness or lease refresh, permitting concurrent writers. | Use an OS lock or token plus PID/liveness/heartbeat; a guard removes only its own token. | Hold a real lock beyond ten seconds and prove a second writer cannot enter. |
| UNIT-03 | P2 | `crates/fpas-unit/src/object/mod.rs:64-70,143-191,254-257` | Object validation permits internal `Halt` instructions although the object contract and linker reject them later. | Enforce exactly one trailing `Halt` at object validation/encoding and remove duplicate downstream policy. | `[Halt, Halt]`, internal Halt with later code, missing final Halt, and valid object. |
| UNIT-04 | P2 | `crates/fpas-unit/src/interface/symbols.rs:93-149`, `src/interface/types.rs:127-133` | Interface canonicalization lowercases record names but leaves documented-canonical enum qualified names unchanged, destabilizing bytes/hash. | Canonicalize `EnumType.name` and audit other documented-canonical identity fields. | Case-variation digest and deterministic encoding tests. |
| UNIT-05 | P3 | `crates/fpas-unit/src/sidecar/atomic.rs:47-60` | If writing lock metadata fails after `create_new`, no guard exists yet and the lock file leaks until stale cleanup. | Construct cleanup ownership immediately after open, then write metadata, or remove explicitly on failure. | Inject metadata write failure and assert immediate cleanup. |
| UNIT-06 | P2 | `crates/fpas-unit/src/sidecar/mod.rs:38-47` and decoded payload APIs | Integrity hardening: reusable envelope identity is not cross-checked against `UnitInterface.unit_name` and `RelocatableObject.owner`. A hash-consistent mismatched payload can represent another logical Unit. | Provide a typed validated-load API that decodes both payloads and checks all owner/name identities before declaring reuse. | Envelope/interface/object name mismatch and case-insensitive duplicate symbols. |

## Implementation notes

UNIT-01/02 share resource and lock policy with `fpas-build`; UNIT-03 shares validation policy with bytecode/linker. Agree those boundaries before coding. Keep sidecars source-adjacent and never commit generated `.fpascu` files.

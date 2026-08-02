# `fpas-bytecode` review follow-up

Classification: bytecode format and validation. Fixes should tighten internal artifact validation without changing FPAS semantics.
Status: BYTECODE-01 through BYTECODE-05 completed 2026-08-02; BYTECODE-01 root-return correction completed 2026-08-02.

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

## Implementation record

- BYTECODE-01 traverses root control flow from instruction zero and derives callable regions from
  function-table entries. Validation requires at least one structurally reachable `Halt` or
  root-level `Return`, rejects entry fallthrough, and prevents entry jumps or fallthrough into
  callable bodies. Root `Return` is a valid early program exit implemented by the VM; the
  workspace example regression caught and corrected its initial rejection. Conditional loops
  retain their exit edge, so this does not require programs to terminate at runtime.
- BYTECODE-02 formats capture cells as the stable opaque value `<cell>`. Formatting no longer
  acquires the cell mutex, so self-referential cells cannot recursively lock themselves.
- BYTECODE-03 gives constant-pool identity a bit-exact real comparison while preserving runtime
  equality (`+0.0 = -0.0`). Signed zeros now receive distinct pool indices and retain their exact
  IEEE-754 bits through persistent JSON encoding.
- BYTECODE-04 validates every string-name operand used by globals, calls, closures, records,
  fields, and enums before execution, with the instruction, constant index, operand role, and
  actual value category in the typed error.
- BYTECODE-05 resolves direct calls and closure targets against the callable table using the VM's
  case-insensitive fallback. Direct-call arity must match the table entry.
- The additional test gaps are closed with complete supported `PersistentValue` roundtrips,
  rejection coverage for every runtime-only category and capturing functions, and a bytecode-v1
  JSON golden containing every `Op` variant.
- `docs/pascal/program-structure/cli.md` documents the validation applied to directly executed
  `.fpascp` images. FPAS syntax, language semantics, and `Std.*` APIs are unchanged.

## Verification

- Baseline: `cargo test -p fpas-bytecode --locked` — passed: 53 tests plus doc tests.
- Targeted implementation: `cargo test -p fpas-bytecode --locked` — passed: 74 tests plus doc
  tests.
- Root-return correction: `cargo test -p fpas-bytecode --locked` — passed: 74 tests plus doc
  tests; `cargo test -p fpas-cli --bin fpas main_tests::examples::example_fs_basics --locked` —
  passed.
- Direct dependents: `cargo test -p fpas-program -p fpas-linker -p fpas-build -p fpas-bundle
  --locked` — passed.
- `cargo clippy -p fpas-bytecode --all-targets --locked -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed after the adjacent
  workspace style findings were corrected as part of the linker review follow-up.

# AP54.2: Implement managed byte storage and core operations

Package: [AP54](README.md). Status: Complete on the selected integration branch,
including implementation, documentation, review and verification. AP54.3 owns
the delivered UTF-8 and binary API integration.

## Prerequisites

- [AP54.1](01-contract.md) supplies the exact API, errors, 256 MiB limit,
  slice/capacity rules, ownership and artifact behavior.
- Implementation is authorized. No syntax change is implied by that authorization.

## Contract and scope

Implement the approved `Std.Bytes` buffer as contiguous native byte storage,
with construction, observation, checked byte access, bulk append/copy/fill,
slicing and capacity management. The representation must not be an FPAS array
of tagged integer values hidden behind an opaque name.

Use copy-on-write value semantics through assignment, function arguments, returns,
record/enum/collection fields and first-class operations. Read-only bindings
must not gain mutation through a shared alias. Handle empty storage, overlap,
self-append and growth using checked size arithmetic and initialized bytes.
Equality compares bytes, not capacity or backing identity. Apply the selected
generic constraint capabilities consistently. Invalid direct indices follow
existing array runtime-failure rules; expected conversion/limit errors use the
specified typed results. InPlace buffer functions return Result with Ok(true)
on success, including no-op calls; existing collections retain their signatures.
Support Equatable dictionary keys, protected byte iteration and Printable with a
32-byte hex preview, without ordering or implicit text conversion. Do not add
public `Close`, implicit array conversion or duplicate free-routine aliases.

Integrate backing storage with roots, graph inspection, task/domain transfer,
host values, iterative destruction and cycle collection. Ordinary bytes contain
no FPAS references; a surrounding graph that captures a buffer can still form
a cycle. Do not retain a registry entry until VM teardown as a substitute for
normal value lifetime. Reuse established Rust storage facilities; evaluate a
dependency only where it simplifies the approved contract.

Support compiled-unit/program-image type metadata and all existing debugger
functions, including retained byte values, detached calls and bounded event
capture. Ordinary type identity and declaration lookup must work; a valid buffer
must not become an unsupported value in those existing functions. A retained
value preserves all bytes and observable length/capacity. Preview limits affect
only presentation. General snapshots and state recording/restoration/replay
remain separate and are not prerequisites or acceptance targets for AP54.2.

Existing binary APIs stay usable until AP54.3 replaces their signatures together
with all callers. Do not introduce temporary buffer-suffixed I/O operations.

## Implementation decisions

The runtime uses `SharedBytes(Arc<Vec<u8>>)`, with no pool or retention registry.
The backing contains initialized native bytes rather than tagged FPAS integers.
Fallible vector allocation happens before replacing shared storage; unique backing
can grow in place. Proper slices allocate independent storage and full nonempty
slices may share. No extra Rust dependency is needed for this representation.

The buffer has its own semantic, interface, IR and portable debug type. Artifact
bytecode version 31 admits that type and the byte intrinsics. The canonical native
operation catalogue owns receiver signatures, documentation and first-class values;
the compiler commits a fallible InPlace result only on success. Capacity-only
mutations have no public value-operation aliases.

Ordinary graph traversal treats bytes as an ownership-neutral leaf; surrounding
records, closures and cells retain the existing collector and task-transfer rules.
Arc ownership releases native backing when its last owner ends. Host exports and
debugger retained values keep complete immutable storage, with copy-on-write on
mutation; only their presentation is bounded.

## Affected files and implementation owners

These are the delivered storage, type, operation and integration owners.
Canonical binary-I/O migration is owned by AP54.3.

| Action | Path | Purpose |
| --- | --- | --- |
| Create/modify | `crates/fpas-runtime/src/value/bytes.rs`, `value/mod.rs` and owning value modules | Native backing storage, cloning/mutation, graph/ownership summaries, accounting and disposal |
| Create/modify | `crates/fpas-runtime/src/stdlib/bytes.rs`, `stdlib/intrinsics.rs`, `execution/vm/` | Core operations, dispatch and task/host admission |
| Create/modify | `crates/fpas-builtins/src/std_units/`, `crates/fpas-sema/src/std_registry/loaded/bytes.rs`, `loaded/mod.rs` | Canonical unit/type/error and signature registration |
| Create/modify | `crates/fpas-code/src/intrinsic/bytes.rs`, intrinsic/type/validation metadata | Instruction metadata, verified operand/result shapes and approved artifact behavior |
| Modify | `crates/fpas-sema/src/types/`, `crates/fpas-ir/src/program.rs`, `crates/fpas-unit/src/` | Preserve the native byte type through checking, lowered IR, identities and separately compiled interfaces |
| Create/modify | `crates/fpas-sema/src/std_registry/native/bytes.rs`, `crates/fpas-compiler/src/lowering/calls/fluent.rs` | Canonical receiver operations and successful-result-only mutation commits |
| Create/modify | `crates/fpas-build/tests/byte_buffer_memory.rs` | Measure actual native allocation balance across VM/task/cycle teardown |
| Modify as required | `crates/fpas-compiler/src/intrinsic_catalog.rs`, `src/lowering/`, `src/bytecode/` | Existing call and first-class-operation lowering without grammar exceptions |
| Modify as required | `crates/fpas-runtime/src/execution/vm/debug/`, `crates/fpas-code/src/debug/` | Buffer inspection and approved debug/recording metadata |
| Create/modify | `tests/stdlib/bytes/`, owning Rust value/runtime tests | Observable operation, ownership and allocation regressions |
| Create/modify | `docs/pascal/std/bytes.md`, Std index and agent reference where needed | Delivered API and authoring rules |
| Generate | `lib/api/Std/Bytes.fpas` | Intrinsic editor declarations from registered signatures and handbook |
| Modify | `docs/specs/std-namespaces.md` | Move only the delivered `Std.Bytes` capability into the current catalogue; keep unimplemented binary helpers reserved |

Repository paths above are relative to the repository root.

## Review boundaries

Spec review uses AP54.1, including the user's decision that general snapshots and
record/replay remain separate. Standards review uses AGENTS.md and the shared
development process. Baseline is `295726a77`; the review target is the uncommitted
AP54.2 working tree on `codex/std-namespaces`.

Confirmed gaps were fixed at their owners: portable type equality now recognizes
buffers across compiled units and debugger reference parameters; indexed byte
reads cannot escape as writable integer references; byte calls validate the
BufferError ABI before execution. Bundle/program-image goldens and the intrinsic
and API inventories match the new bytecode version and delivered operation set.
Regression tests exercise these boundaries rather than only
repeating catalog definitions.

Native allocation tests require exact baseline restoration after cycles, task
handoff, VM failure and normal teardown, with normal and most frequent GC
schedules. Storage tests separately check immediate last-owner release, slice
retention and injected allocation failures. No representation benchmarks are
claimed by AP54.2; combined I/O/native-request ownership and representation-cost
evidence are owned by AP54.3/AP54.4.

## Acceptance and verification

Verify positive, rejected and boundary cases for construction, indexing, slices,
copying, mutation, overlap, capacity and checked arithmetic. Include first-class
operation references, writable-storage rejection, byte-content equality, nested
aggregates, channels/task results and host-session transfer. Prove copies behave
according to the approved contract rather than merely checking storage addresses.

Add native allocation accounting that sees actual buffer bytes/capacity, including
shared allocations and native owners; count shared backing once. Check baseline
restoration after final ownership ends and after cycles/failures. Do not add a
buffer retention pool. Account for release separately from allocator RSS, which
is supporting evidence rather than the pass/fail oracle. Inject recoverable
allocation failures through existing Rust test mechanisms; do not depend on
actual process exhaustion or add timing-sensitive acceptance tests.

Run relevant Rust and FPAS tests, builds, formatter and strict Clippy. Verify the
generated declarations, unit lookup and compiled-unit reuse. Follow the
[required checks](../development-process.md#required-checks) and run the complete
Rust/FPAS suites at delivery. Tests must pass before AP54.3 begins.

Document delivered behavior, record review findings and fix confirmed errors
before checking off AP54.2. Test evidence belongs in the delivery report, not a
chronological log in this file.

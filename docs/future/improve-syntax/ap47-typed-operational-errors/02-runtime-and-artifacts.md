# AP47.2: Runtime layouts and portable artifacts

Status: Complete. Depends on AP47.1.
Contract: [native errors](01-decision.md). Completion: [package README](README.md).

## Delivered boundary

This work package supplies the internal runtime and portable-artifact foundation
for native errors. Hand-built verified fixtures exercise it before AP47.3 emits
it from source. Native declarations, source-level Error constraints, Message(),
constructor values, MapError and Std migration remain in their later packages.

Error values reuse Value::Enum and shared enum storage. Value remains 16 bytes;
ordinary enum erasure, instruction dispatch, scalar arithmetic, register cloning
and scheduler loops retain their existing representation and contract.

## Shared metadata and runtime behavior

Each error family has a concrete ErrorInstanceKey. Each variant retains essential
semantic field types, an optional literal message and an optional Cause slot with
its exact concrete family ID. These are independent of optional debugger types.
RuntimeEnumLayout resolves the message once and shares descriptive metadata on
its Arc. Constructing a value stores payloads without formatting or capturing a
stack, performing I/O or creating a separate report chain.

The existing iterative display traversal renders error frames and causes while
preserving ordinary display behavior. It emits the defining qualified family and
variant, optional message, declaration-ordered non-Cause fields and the original
typed Cause. Strings anywhere inside displayed fields are JSON-escaped; control
characters cannot introduce extra report lines. Output uses LF without a final
newline. Nested error values in ordinary fields remain compact field data.
AP47.3 will route Message() through this same renderer.

MakeEnum, host aggregate construction and debugger construction check the
expected Cause family at construction. Debugger payload replacement checks the
same identity without depending on optional type projections. Live-image
compatibility includes the complete key, semantic field types, message text and
Cause identity. Existing inspection retains the shared nominal layouts.

Final destruction of an error enum drains uniquely owned children iteratively across
enums, records, arrays, dictionaries, closures and wrapped payloads. Atomic
Arc::into_inner transfers the last owner's children without recursive descent
or cloned chains. The ordinary shared-owner and copy-on-write contracts remain
unchanged; this adds no Value-wide Drop hook or per-instruction error work.
The traversal starts at error-enum ownership boundaries. Ordinary aggregate
release keeps its existing cleared-buffer path.

## Concrete identity and source-less construction

ErrorInstanceKey contains the canonical defining declaration and complete
ordered semantic arguments. Transparent aliases must already be normalized by
the source producer. Nominal arguments retain their kind, defining declaration
and full arguments; structural arguments retain their components, subrange
bounds and complete callable signatures, including variadic behavior. Keys use
full equality, not display names, erased layouts, optional debug types or hashes
alone. Ordinary generic arguments keep this semantic information without gaining
new ordinary runtime layouts.

The linker interns equal keys across units and keeps distinct arguments separate.
It reserves keys before following Cause dependencies, admitting finite recursive
families. ErrorSchema retains symbolic fields and generic constraints. Bounded
substitution rejects unresolved arguments, violated constraints and expanding
instance requests before execution.
Its node budget covers all fields and variants of the instance before any
argument copying; arguments are validated once rather than once per field.
A shared link budget also bounds all instantiated schemas together to one million
metadata nodes and 64 MiB of copied text, so small requests cannot multiply a
large template without a bound. These checks precede layout and factory copies.

Interfaces retain ordered hidden constructor requirements per affected routine.
Objects retain those requirements, concrete factory requests, error schemas and
their real source provenance, including schemas for private reachable helpers.
Each requested concrete variant becomes one shared noncapturing function using
existing MakeEnum and Return instructions. Factories retain declaration source
locations; mandatory executable source maps remain intact. Duplicate requests
share bodies. Full requests are checked even when internal symbol names use a
digest. Existing erased generic bodies call supplied constructor functions;
there is no constructor opcode, runtime registry or per-call type lookup.

AP47.3 owns generation and fixed-point propagation of these hidden requirements,
call-site substitution and binding factories into generic routine values. The
transport and linker support here do not expose source syntax or change source
arity, effects, inference, transfer rules or visible debugger bindings.

## Artifact admission and versions

Interfaces, objects, executable metadata and program images preserve essential
error identity, labels, payload types and concrete Cause references. Interface
hashes include schemas, constraints and construction requirements; changing a
message therefore changes the semantic hash. Existing caches use those hashes.
Object-only source provenance does not affect the public semantic interface.

Admission checks family/variant agreement, defining identity, unique concrete
keys, field arity, literal references, reserved Cause slot and exact family.
Both schema-backed and schema-free concrete payloads enforce the existing
Printable capability exclusions. Essential type graphs have bounded nodes and
depth; flat postfix wire identities avoid recursive untrusted deserialization.
Conflicting copies, malformed references and unbounded instance expansion fail
before execution. Ordinary enums cannot carry error-only variant metadata.

Changed versions are compiled-unit envelope 21, object schema 15 and program
format 23. Register bytecode remains version 23 because the instruction set is
unchanged. Source-backed caches rebuild through existing version handling;
unsupported standalone artifacts require rebuilding. No compatibility decoder
or user-artifact deletion is introduced.

## Implementation owners

| Path | Responsibility |
| --- | --- |
| `crates/fpas-bytecode/src/metadata/errors/`, `metadata/enums.rs` | Bounded full semantic keys, essential payload types and Cause metadata |
| `crates/fpas-bytecode/src/validate/errors.rs` | Executable admission independent of debugger data |
| `crates/fpas-bytecode/src/value/aggregate.rs`, `value/display.rs` | Shared runtime metadata, construction guard and unified iterative rendering |
| `crates/fpas-bytecode/src/value/drain.rs`, `managed_heap.rs`, `array.rs`, `function.rs` | Iterative final aggregate release at existing ownership boundaries |
| `crates/fpas-unit/src/errors/`, `interface/`, `object/` | Schemas, constraints, hidden construction ABI, provenance, object admission and codecs |
| `crates/fpas-linker/src/errors/`, `plan/`, `emit/errors.rs` | Concrete instance closure, shared factories, full-key coalescing and ID relocation |
| `crates/fpas-program/src/format/executable/errors.rs`, `format/` | Essential image metadata codec and changed format versions |
| `crates/fpas-vm/src/vm/layouts.rs`, `execute/aggregates.rs`, `debug/` | Runtime layouts, guarded construction/mutation, inspection and live-image compatibility |
| `crates/fpas-sema/src/interface/`, `types/generic_parameters.rs` | Dormant portable constraint transport; reject unsupported source-native interfaces explicitly |

Existing ordinary producers and fixtures initialize the new metadata empty.
The current artifact handbook is
[compiled programs](../../../pascal/program-structure/compiled-programs.md).
Source grammar, Std signatures and source-language examples are unchanged.

## Regression coverage and follow-ups

Bytecode fixtures check exact rendering, escaped nested fields, ordinary enum
output and a deep Cause chain rendering and releasing on a small host stack.
Existing Value-size checks retain the 16-byte invariant.

Unit-interface fixtures prove message-sensitive hashing, Error constraints,
hidden requirements, bounded wire identities and complete callable identity.
Linker fixtures round-trip independent objects, coalesce equal requests, retain
different concrete and nested generic causes and distinguish generic arguments
outside Cause, including ordinary generic types. An erased helper calls supplied
factories through the existing calling convention. A stripped-debug program
image round-trips through the production codec, renders exact selected output
and rejects a Cause from another concrete instance during VM construction.

Admission fixtures reject corrupted family/message/Cause/key metadata,
non-Printable concrete payloads without schemas, conflicting schemas, invalid
bounds and missing or expanding requests. Recursive finite schemas remain valid.
Separate API and linker fixtures reject aggregate expansion across fields and
across concrete instances, with valid bounded controls.
Debugger fixtures reject incompatible keys/messages during live replacement and
wrong-family Cause replacement without optional debug types.

Applicable workspace and language suites, artifact checks and unchanged VM/full
suite measurements follow the
[acceptance and performance gates](07-acceptance-and-performance.md).
New source-language and Std acceptance remains in AP47.3-AP47.7; no feature
benchmark baseline is claimed for source syntax that does not exist yet.

Independent checkout follow-ups remain in the existing imported-generic-enum
debug-projection test and the repository NotesUpdate reference-count assertion.
The unchanged pre-implementation binary reproduces the projection failure and
the current reference set. These source projection and query contracts are
outside AP47.2; verification evidence belongs in the delivery report.

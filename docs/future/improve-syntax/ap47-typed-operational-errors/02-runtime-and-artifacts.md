# AP47.2: Runtime layouts and portable artifacts

Status: Planned; implementation not authorized. Depends on AP47.1.
Contract: [native errors](01-decision.md). Completion: [package README](README.md).

## Deliverable

Extend the existing nominal variant representation with essential error metadata
and serialization support. Keep ordinary enums unchanged and the Value register
representation at 16 bytes. This package must be usable without exposing new
source syntax; hand-built verified fixtures exercise the new metadata until
AP47.3 emits it from source.

## Representation and validation

- Reuse Value::Enum(SharedEnum); do not add a Value alternative or a check in
  instruction dispatch, scalar arithmetic, register cloning or scheduler loops.
- Add an ordinary/error family discriminator to enum-layout metadata. Per error
  variant retain the optional message string reference and optional Cause slot
  together with its exact instantiated cause-family ID.
  Existing field names/types and owner identity remain authoritative.
- Mirror this information in RuntimeEnumLayout so runtime formatting needs no
  source lookup or global type registry. Descriptive data lives on shared layout
  Arcs, not on every value. Construction only stores ordinary payloads.
- Use one iterative display traversal for error frames, causes and ordinary
  payloads. It must preserve existing non-error display behavior. An error's
  Message() intrinsic and Console display call this renderer rather than building
  separate strings or using debugger projections.
- Validate owner identity, string references, family discriminator, field arity,
  Cause index and Cause-family identity at artifact admission. Store that identity
  independently of optional debugger type data. Error construction validates an
  untrusted Cause value against that expected nominal family at its construction
  owner, not by polling ordinary dispatch. Reject message/Cause metadata
  on ordinary enums and an Error constraint attached to a non-error instance.
  Enforce existing resource/graph limits without exponentially revisiting shared
  type graphs. Validation must handle recursive finite error declarations.
- Rendering has no I/O, global error buffer, stack capture or per-instruction
  work. Deep payloads/causes, printing and final destruction remain stack-safe.

## Error instance identity and generic construction

The current compiler's intern_enum/reserve_enum uses the generic template and
declaration name; the linker's canonical_enum_name likewise merges by name. That
erasure remains correct for ordinary enums, but cannot supply the exact Cause
identity required for errors. Select the following error-only representation:

- An ErrorInstanceKey consists of the canonical defining declaration identity
  and its complete ordered vector of concrete semantic type arguments, including
  any enclosing generic parameters used by that type. Resolve transparent aliases
  first. Normalize argument identities using the existing checked type rules:
  nominal types retain their defining identity and full arguments; structural
  types retain their kind and normalized components, including subrange bounds.
  Do not use display names, optional debug types, erased layout IDs or a hash
  without equality checking as the key. Ordinary generic types used as arguments
  retain their full semantic identity in this key without gaining new layouts.
- Intern one error EnumTypeId and its variant IDs per concrete key. Equal keys
  from separate units coalesce; different arguments never coalesce merely because
  the declaration name matches. Reserve the key before visiting payload/Cause
  dependencies so finite recursive families share metadata. Generic templates are
  schemas, not executable error instances. Every executable Cause reference
  resolves to the exact concrete error instance, including generic causes.
- RuntimeEnumLayout is shared per concrete instance/variant and carries that
  instance's owner and Cause ID. The printed header remains the defining qualified
  declaration and variant name specified by AP47.1; display spelling is not an
  identity key. Passing, storing, returning or wrapping a value preserves its Arc
  and instance identity, including through Result and ordinary erased aggregates.
- Keep generic routine bodies erased. Where construction depends on unresolved
  type parameters, lower it to an ordinary call through a compiler-supplied
  constructor function. Each affected routine has a deterministic list of hidden
  readonly constructor parameters; propagate requirements from callees to callers
  with checked type substitution at each call, to a fixed point. Concrete call
  sites request factories by instance key and variant; generic callers forward
  the corresponding hidden parameters.
  Passing an existing error value or calling Message() needs no such parameter.
- Each concrete factory is one shared noncapturing function using existing
  MakeEnum construction with that concrete variant and Cause check. Generic
  routine values bind required factories using existing callable adapters/captures
  when the value is formed, without allocating an environment on each invocation.
  Direct calls, nested/recursive routines, typed callbacks and task entry use the
  same internal calling contract. Hidden parameters do not change source arity,
  labels, inference, effects, transfer rules or user-visible debug bindings.
- Compiled interfaces retain symbolic construction requirements for callers;
  objects retain the referenced error schemas and factory requests/templates,
  including private helpers reached by exported generics. The linker substitutes
  concrete arguments and emits/deduplicates the small factories and layouts, not
  specialized copies of generic routine bodies. Schema references do not grant
  source access to private declarations. Resolve all requests before execution;
  reject unresolved parameters or unbounded instance expansion with a compilation
  or linking diagnostic under the existing graph/resource limits. No runtime
  registry, type inference from values, new constructor opcode or per-call lookup
  is added.

This costs shared metadata and a small constructor body per used error instance
and variant, plus hidden argument passing/ordinary calls in affected generics.
Only bound routine values may need their ordinary capture storage. Account for
these costs in AP47.7; unrelated generic routines and ordinary enum layouts keep
their current representation and calling contract.

## Artifact contract

Persist family kind, normalized instance keys, variant messages and concrete Cause
slots through interface exports, unit objects, executable metadata, linking and
program images. Interfaces/objects also preserve the internal construction
requirements and schemas needed to link new concrete callers without source.
The linker coalesces errors by instance key, remaps owner/variant/Cause IDs and
checks equal keys have compatible schemas; name-only enum merging applies only
to ordinary enums. Error identity and output must work after stripping optional
debugger information. Include Error constraints, public declarations and internal
construction requirements in interface hashes and build-cache keys. A message-only
change invalidates dependent images because it changes public output; matching
and variant identity remain nominal.

Increment the unit envelope, object and program format versions once when their
schemas change. Use versions current at implementation time, not fixed numbers
copied into this plan. Rebuild owned caches/precompiled Std artifacts. Old
standalone images fail with the established version diagnostic; do not silently
reinterpret them or add a compatibility decoder. Source-backed build caches use
their existing rebuild behavior. Do not delete unrelated work or user images.

## Actual owners and planned edits

| Path | Responsibility |
| --- | --- |
| `crates/fpas-bytecode/src/metadata/enums.rs` | Essential family, concrete instance keys and per-variant error metadata |
| `crates/fpas-bytecode/src/value/aggregate.rs`, `value/display.rs` | Shared runtime metadata and unified iterative rendering |
| `crates/fpas-bytecode/src/validate/` | Admission of nominal metadata and typed causes |
| `crates/fpas-unit/src/interface/`, `src/object/`, `src/format/` | Interface identity, construction requirements/schema templates, codecs and versions |
| `crates/fpas-program/src/format/` | Program-image codec, version and admission |
| `crates/fpas-linker/src/plan/layouts.rs`, `src/emit/` | Coalesce error instance keys, resolve factories and remap variant/type/string/Cause IDs |
| `crates/fpas-vm/src/vm/inspection/`, `src/snapshot_value.rs` | Preserve nominal inspectability without making debugger data essential |

Create focused error-rendering/metadata helpers beside these owners if needed;
keep files below roughly 500 lines. No unrelated VM refactor is required.

## Acceptance and verification

Extend existing bytecode, unit, linker and program-image tests at their production
interfaces. One fixture uses two nominally distinct error families with a typed
cause, a literal message and aliases. A hand-built generic schema fixture links
two units requesting ContextError with different cause families plus an alias of
one instance: different keys/IDs survive, equal keys coalesce and each factory
accepts only its declared Cause. Include nested generic causes and a generic
argument outside Cause, so equality cannot depend only on the cause's layout.
Verify import/link and stripped-image round trips preserve these identities,
matching and exact selected output. Mutate individual owner/string/Cause/key and
factory references in serialized fixtures and assert admission rejects them
before execution. Reject equal keys with conflicting schemas and unresolved or
expanding factory requests. Existing graph-bound tests cover shared and recursive
metadata; extend their tables rather than duplicate them.

Ordinary enums retain byte-for-byte selected display behavior; Value size stays
16 bytes. Deep causes render and drop without host-stack overflow. An error frame
has one stored typed Cause, not a copied report chain. Relevant crate suites and
format/build checks must pass. Existing VM benchmarks establish that dormant
metadata does not slow ordinary execution, following [AP47.7](07-acceptance-and-performance.md).

Source syntax, public Std signatures and real handbook examples are unchanged in
this package. Describe only the shipped artifact behavior if documentation of
the artifact format changes; do not advertise source support early.

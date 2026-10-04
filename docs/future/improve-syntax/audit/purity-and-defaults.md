# Purity and default initializer integration

Status: implementation, consumer migration and verification complete. Final stage acceptance is recorded in the
[member-removal delivery](member-functions.md). This work implements the approved
[default-purity boundary](default-purity-boundary.md) together with the stage-5
pure-function rules required by stage 4.

## Implemented foundation

- `pure function` declarations, anonymous functions and callable types retain
  their capability through parsing, formatting, semantic types, imported
  interfaces and editor signatures. Ordinary callables cannot satisfy pure
  requirements; forgetting a guarantee cannot recover it later.
- The shared checker validates pure calls, reads, captures, local mutation,
  parameter modes, recursive component capabilities and concrete generic
  instantiations. Nominal generic arguments and writable channel/reference
  positions retain invariant type identity.
- Recursive and forward nominal signatures are checked after complete headers.
  The finite component-state analysis tracks both signature validity and data
  capability when recursive arguments change shape.
- Audited intrinsic purity metadata drives checker registration and generated
  declarations. Functional collection callbacks require explicit pure function
  signatures; action traversal and caller-mutation procedures remain ordinary.
- Defaults use the same evaluation checks without the pure-function result-type
  restriction. Internal compiler functions execute defaults in their declaration
  context. Stored expressions retain AST identity for lowering metadata. Scalar
  static defaults remain available to importing static-expression checks.
- Compiled-unit default descriptors preserve internal initializer identities
  through aliases and supporting interfaces. Implementations are link-visible
  without introducing source callable symbols or granting a pure callable type.
- Generic defaults retain the concrete capabilities required by their operations.
  Selected nested defaults propagate requirements to a fixed point, including
  forward declarations. Explicitly supplied fields skip those default obligations.
  Initializer descriptors carry the requirements through compiled-unit reuse.

## Corrections exposed by integration

The existing specialized Option/Result callback paths did not validate every
parameter, mode and container result. A shared callback context now checks the
complete signature and contextually instantiates generic callback values.
Projection predicates also validate the input container and UnwrapOr validates
its fallback type.

The implicit property-getter path initially bypassed pure-call checking. A default
could read a property whose getter mutated global storage. The user-authorized
correction checks the shared getter resolution for both defaults and pure
functions. Regressions cover designators, parentheses, indices, nested getters,
callable-valued properties, decision expressions and imported aliases. Ordinary
closure bodies remain ordinary even when creating the closure is a pure default.
The same evaluation check covers implicit property setters, event queries,
handler assignment and event invocation.

The imported-getter regression also exposed a baseline link-visibility error:
public properties could refer to private accessors that compiler imports and
exports filtered out. Those accessors are now link-visible through the public
member while retaining their private source visibility. The same reachability
rule applies to event accessors.

Transitive intrinsic types in source-unit interfaces were unavailable unless the
consumer directly imported their intrinsic unit. Capability analysis now resolves
canonical intrinsic type metadata without making those names source-visible.
Tests cover both resource-free enum components and forbidden resource handles.

Two CLI streaming fixtures joined server threads before checking compilation
success. They now report a failed program before joining a server that never
received a connection. The initial workspace migration run required terminating
its stalled CLI test process; that run is not a passing verification checkpoint.

## Verification and remaining work

Focused checks cover parser rejection of pure procedures, signature variance,
recursive and forward capabilities, mutable captures, forbidden intrinsic calls,
default shadowing, resource-bearing empty defaults, generic selected-default
requirements, private helpers, callable/aggregate defaults, alias chains and
cold/warm compiled-unit reuse. Compiler regressions verify that supplied fields
evaluate in written order before defaults, defaults follow declaration order, and
each construction calls only its selected initializers once.

Verification passes: `cargo fmt`, `cargo build`, 3,869 Rust tests across 192
groups, and the full FPAS suite (470 passed, one intentional skip, no failures).
FPAS formatting, all 22 project manifests, 109 application/example programs in
their source or project contexts, 57 complete handbook programs, documentation
links and the real VS Code extension suite also pass at this integration
checkpoint. Imported public properties and events retain working private
accessors on both cold and reused compiled units; direct private access remains
rejected.

The subsequent [member-removal delivery](member-functions.md) replaces methods,
properties, events, static record routines, fluent calls, Assigned and nil with
ordinary functions and optional handlers. Its parser and callable regressions
supersede the intermediate getter/accessor tests described above. Bare-task
removal retains its separately approved stage-5 migration boundary.

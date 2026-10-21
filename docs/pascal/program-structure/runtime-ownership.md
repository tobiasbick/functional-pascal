# Runtime value ownership for Rust hosts

The Rust runtime represents mutable captured variables with `OwnedCell` and
identifies their owner with `MemoryDomain`. Domain identity is independent of
numeric task IDs, worker threads and executable identity. A new domain has a new
identity; cloning its handle preserves identity. Saved task state and supervised
retries retain the logical task's domain. Synchronous callbacks use their caller's
domain, and a `CallbackSession` retains its domain across invocations.

## Cells and references

Create cells through `MemoryDomain::new_cell`. Reads, snapshots, replacement and
updates require the owning domain. Construction and replacement inspect the
complete contained graph before changing storage. A foreign domain or an escaping
`var` reference rejects the operation without replacing the old value. Public Rust
code cannot obtain a mutable lock guard for a cell's payload.

`OwnedCell::update` computes a replacement from an immutable snapshot outside the
cell lock. Commit checks that another replacement has not changed the snapshot in
the meantime. A failed transformation, ownership check or concurrent update leaves
the current stored value unchanged. Removed values are released outside cell locks.
Nonblocking inspection uses `try_read` or `try_read_snapshot` and reports unavailable
storage instead of waiting for another accessor.

Cell-backed `var` references retain the cell's identity. Global-backed references
also carry the originating domain, so equal global slots in different sessions do
not confer access. References remain temporary call arguments; they cannot be
stored in cells or captured as ordinary retained values.
Directly invoked nested helpers may temporarily capture their enclosing `var`
arguments. Same-domain invocation permits these borrowed captures; storing the
callable in a cell or aggregate still rejects the reachable reference.

The domain registers cells weakly. Registration neither retains an acyclic value
nor keeps an abandoned domain alive. Releasing a cell removes its registration;
registry snapshots also prune expired entries. These administrative snapshots are
not program roots. Cycles use the explicit collection protocol described below.

## Function provenance

Create callable values from `VerifiedExecutable::function_target`. This validates
the numeric function index against the executable and supplies its canonical name
and opaque runtime identity. Safe Rust constructors cannot attach an arbitrary
index to a different executable's identity. A separate display label may preserve
source spelling, without changing the validated target or its identity.

Cloning `Arc<VerifiedExecutable>` preserves code identity. Independently verifying,
loading or cloning a `VerifiedExecutable` creates a different identity, even if its
contents and function IDs are equal. Structural executable equality remains a
comparison of content. Provenance is runtime-only and does not change persistent
artifact bytes or their ABI version.

`SharedFunction` retains the validated target, captures and optional bound receiver.
A task-bound function additionally retains its owning `DomainId`. Code identity,
cell ownership and task-bound invocation restrictions are separate properties.
Immutable wrapper types and `const` bindings alone do not prove transferability.

## Session arguments and returned values

`Vm::call` and `CallbackSession::invoke` take `Vec<HostArgument>`.
Use `HostArgument::Data` (or `Value::into`) for independently constructed data and
`HostArgument::Export` (or `HostValue::into`) for a previously returned value.
Parameter types remain the Rust caller's responsibility; admission checks ownership
and provenance, without adding a complete runtime type checker for the host ABI.

`Execution::value` is a sealed `HostValue`. It retains the returned graph and its
memory domain, without keeping execution frames or the whole hosted runtime alive.
Clone it to retain the same export. `try_to_data` extracts only a graph without
local state or session resources. `child` selects immutable aggregate children as
sealed values, and `enum_info` returns layout metadata. These operations do not
expose raw cells or a mutable runtime-value reference. Capture and receiver selection
returns another sealed handle. `read_cell` returns a registered payload handle;
`replace_cell` and `replace` apply ownership and code checks under domain access.
They remain valid after shutdown without invoking FPAS or reviving resources.
`DebugTermination::value` uses the same registered export boundary.

An export containing local state or resources may return only to its originating
session. This includes nested opaque/task handles and the HTTP BodyStream and
SseDecoder records, even though their fields contain integers. Returning an own
closed handle preserves its identity; the corresponding operation still reports
its existing closed result. Ordinary callback return keeps the session active;
shutdown prevents further invocation and closes unused selection cases.

Cross-session imports require a transferable, resource-free graph. Every contained
callable must have the target session's identical shared executable identity.
Accepted functions use target-session globals. A separate executable with equal
numeric function IDs is rejected before execution. These checks apply recursively
to bound receivers, captures, keys and payloads.

Ordinary value teardown and nested Option/Result cloning use iterative traversal
for deep graphs. `release_values` also drains an explicit batch of owned values;
sealed exports and rejected host arguments use it automatically.

## Strong-edge inventory

The common iterative graph traversal follows all contained values, records owning
domains and callable code identities, and distinguishes temporary references from
task-bound callables. Repeated shared allocations and cells are visited once; cycles
do not recurse through the Rust call stack. Cell snapshots share immutable payload
storage, avoiding recursive wrapper cloning during classification.

Immutable aggregates cache a conservative summary of whether they contain any
ownership or callable edges. Checked element updates preserve that summary when
possible; raw mutable access and interrupted updates invalidate it. This avoids
repeated full scans of ordinary data. The summary does not establish resource
provenance: a record with integer fields may still represent a resource handle.

| Value storage | Strong edges followed | How edges can change |
| --- | --- | --- |
| Integer, real, Boolean, string, unit and None | No contained runtime values | Scalar replacement; strings contain bytes only |
| Task and opaque resource handles | No direct `Value` edge; runtime registries own their contents | Registry lifecycle, outside the contained-value graph |
| Array | Every element | Copy-on-write detaches shared storage before mutation |
| Dictionary | Every key and value | Copy-on-write detaches shared storage before mutation |
| Record and enum/error | Every field or payload | Copy-on-write detaches shared storage before mutation; layouts contain metadata only |
| Option.Some and Result | Their uniquely owned boxed payload | Exclusive ownership permits replacement; the box cannot alias itself |
| Function | Captures and bound receiver | Immutable after `SharedFunction` construction |
| Cell | Its stored payload | Checked replacement through its owning domain |
| Cell-backed reference | Its root cell | Immutable reference path; writes go through the checked cell |
| Global-backed reference | Domain identity and a numeric slot, not an owning value edge | The owning execution context resolves the slot |

Safe aggregate constructors only attach already existing child values to new
storage. They cannot attach a strong reference to the allocation being constructed.
When a shared aggregate is updated, copy-on-write detaches it first: a retained
reference to its previous allocation cannot become a back-edge to the new one.
Functions and references expose no mutable shared body, and unique payload boxes
cannot create a shared back-edge. The only remaining operation that changes an edge
of an already shared value allocation is an owned-cell replacement. Therefore every
strong cycle constructible through these safe value APIs passes through a cell.

This argument includes Rust host constructors and debugger writes, not only
compiler-produced FPAS. External runtime registries require their own complete root
inventory before collection can be enabled. Adding a new shared mutable value
variant or a raw mutation API requires revisiting both the inventory and this
cycle argument.

See [closures](../language/functions/closures.md),
[variable references](../language/functions/var-parameters.md) and
[compiled programs](compiled-programs.md).

## Registered roots and external access

`TraceRoots` supplies the common direct-root enumeration interface. Root providers
report busy or poisoned storage instead of treating an incomplete view as empty.
`MemoryDomain::root_values` and `register_roots` retain external/native graphs;
registries index these owners weakly. A registration retains its domain, without
retaining a VM, worker or hosted registry. Independent clones register independently.

Use `MemoryDomain::access` for external reads and mutation. Nested access on the
same thread is allowed. Removing a root inside access defers its removal until the
outermost guard exits; the pending entry remains a root. Removal and the maintenance
request commit together. Every independent removal requests maintenance, even when
another root remains. Replacing a host root or cell also requests maintenance after
disconnecting old graph edges. The execution gate excludes other-thread access and maintenance. Completed dispatch
boundaries publish all execution roots, including the last returned value. Saved
tasks and explicitly published native waits can service requests without dispatching
FPAS. Opaque native calls keep execution access until return/unwind. Registering a
few temporary values alone does not authorize a wait to release execution access.

Maintenance uses a per-domain generation/permit handshake; idle and retired domains
can service requests on the requesting thread. VM-created domains install an explicit-only local collector;
automatic collection remains disabled. See the
[execution protocol and native blocking inventory](../../../crates/fpas-vm/src/vm/domain/EXECUTION.md).

Debugger inspection registers bounded live copies and writable cell origins.
Evaluation uses a separate domain with registered partial construction and error
cleanup. Local output copied for inspection carries its detached domain until the
handle expires; ordinary live closure assignment preserves source cell identity.
Failure reports and trace histories retain detached observations rather than live
cell graphs. See the [runtime holder inventory](../../../crates/fpas-vm/src/vm/domain/ROOTS.md).

Invocation checks executable provenance as well as domain ownership. Compatible
live-image address mapping does not by itself rebind stored callable targets.
A debugger update with retained callable graphs is rejected before changing any
worker or image, until an atomic graph rebind is available. This includes nested
callables, globals, inspection roots and selection cases.

## Explicit cycle reclamation

VM-created domains support explicit maintenance through the internal Rust domain
interface. No public FPAS collection control or automatic scheduling is enabled.
Each pass owns the domain exclusively and uses the complete registered root view.
It snapshots weak cell candidates, marks reachable cells with the shared iterative
walker, and only then disconnects unreachable payload edges. Candidates themselves
are administrative owners, not roots. Reference counting still frees acyclic data
immediately; live cells preserve identity, aliases and copy-on-write behavior.

Preparation reserves required graph/commit buffers and acquires every required cell
lock before changing any payload. Busy, poisoned, foreign or scratch-limited state
aborts without clearing cells. Removed values drain iteratively after all cell locks
are released. A panic after destructive commit starts permanently poisons the domain;
it cannot resume normal execution or expose a partially cleared graph.

Raw `OwnedCell` handles and `CellSnapshot` payload copies are low-level values, not
automatically registered roots. Runtime/host construction, copying and traversal
must keep domain access until the corresponding execution/native/external roots
are published. Retain raw snapshots across a safe point only with registered roots;
public sealed host exports already follow this discipline. Cell access/mutation and
allocation participate in domain exclusion; try operations report busy ownership.

Explicit roots can outlive execution and the VM. In a retired domain, releasing A
and requesting a pass reclaims A's unreachable cycle while independent root B remains
valid. Mutating B can disconnect another cycle without releasing B. Independent
requests use the same generation handshake as execution safe points.

`collection_stats` observes allocated/released storage, cleared nonempty payloads,
completed/aborted collector passes and live cell registrations. An administrative
handle may retain an already cleared cell; it is not evidence of a program root.
See the [collector contract and regression owners](../../../crates/fpas-vm/src/vm/domain/COLLECTION.md).

# AP48.1: Owned cells and graph classification

Package: [AP48](README.md). Status: complete. Effort: large.
Prerequisite: Q37 through Q41 recorded and implementation authorized.

## Work

1. Inventory every strong edge in `Value`, `ReferenceRoot`, function captures and
   bound receivers, plus every cell construction and mutation site. Establish that
   all permitted strong cycles pass through cells. Include safe Rust host constructors
   and debugger mutation, not only compiled FPAS expressions.
2. Add a domain identity that cannot alias another VM, session or retired task.
   Encapsulate cells so construction, reads and writes validate ownership. Carry the
   domain through saved task state, callback workers and supervised restarts.
3. Create one iterative, cycle-safe traversal of contained values. Follow dictionary
   keys and values, all payloads, references, captures and bound receivers. Classify
   transferability separately from invocation restrictions. A task-bound callable
   can remain forbidden even when it has no directly captured cell.
4. Reject insertion of foreign local values before mutating storage. Apply the same
   rule to `var` roots and debugger edits. Classify local graphs by their reachable
   cells and callable capabilities, not by `const` or an outer aggregate's type.
5. Add weak cell registration for later collection, with dead-entry cleanup.
   Administrative handles must not introduce strong domain/value cycles.
6. Give every runtime function value executable provenance, including named functions,
   closures and bound methods. Follow Q41 in the package README: use the identity of
   the shared verified executable, not its function IDs or serialized bytes. Keep
   code identity separate from cell-domain identity and task-bound capability.
   All constructors, constant loading and debugger copies must preserve or explicitly
   establish validated provenance. Do not expose a safe host constructor that can
   attach a foreign function ID without validating the executable association.

## Owners and migration

Modify `crates/fpas-bytecode/src/value/{mod,reference,function,drain}.rs` and focused
new cell/graph modules under that directory. Keep ownership primitives independent
of the VM. Modify `crates/fpas-vm/src/vm/calls/closures.rs`, `worker.rs`,
`tasks/state.rs`, reference execution and debugger cell users. Introduce focused
domain policy modules under `crates/fpas-vm/src/vm/` rather than expanding `Worker`.
Audit and migrate all direct cell constructors, including embedded fixtures.

## Acceptance and verification

- Direct, deeply nested and cyclic graphs terminate classification with the correct
  owner. Mixed-domain writes fail without replacing the original value.
- Equal task numbers in different sessions do not confer access. Worker migration
  and same-task callbacks preserve access and alias identity.
- Equal function IDs in distinct executable instances do not confer callable import
  permission: their validated targets carry different identities, which AP48.2
  checks at import boundaries. Copies sharing one verified executable retain the same code identity.
  Provenance is runtime-only and reconstructed when an artifact is loaded.
- `var` reference reads/writes remain correct and retain their non-escape rule.
- Weak registration neither retains an acyclic cell nor grows without bound as
  ordinary cells are created and released.
- Record the cycle-through-cells argument by value variant and constructor. Any
  counterexample blocks AP48.6 until the collector design is revised.

Extend value/reference and VM task-owned-function tests at their public behavior
interfaces. Update Rust ownership documentation with delivery. Run the applicable
Rust and language checks from the development process; collection stays disabled.

## Implementation scope

- Add domain, owned-cell and graph traversal modules under
  `crates/fpas-bytecode/src/value/`; migrate `mod.rs`, `reference.rs`, `function.rs`,
  `drain.rs` and `equal.rs` to encapsulated ownership and validated callable targets.
- Extend `crates/fpas-bytecode/src/executable.rs` with runtime-only executable
  provenance and validated function targets. Preserve artifact representation.
- Add focused domain policy under `crates/fpas-vm/src/vm/domain/`; migrate
  `worker.rs`, `tasks/state.rs`, task save/restore and supervision retry paths,
  `callback.rs` and `callback_call.rs` to logical-task domains.
- Migrate `calls/closures.rs`, `access.rs`, `execute/references.rs`, inspection
  cell readers and debugger mutation/detachment/callable construction to checked
  cells and preserved executable provenance.
- Validate optimized persistent updates in `execute/global_index_path.rs` before
  consuming the source register or mutating a global, and extend its regressions.
- Maintain a conservative plain-scalar summary in `value/array.rs` so repeated
  checked appends and indexed writes do not rescan growing scalar arrays. Raw
  mutable access invalidates the summary; graph traversal trusts only proven
  primitive contents. Migrate indexed VM updates to the checked summary-preserving
  array operation and cover invalidation and copy-on-write isolation.
- Extend this bounded-cost validation to immutable compound data in
  `value/aggregate.rs` and a focused `value/ownership_summary.rs`. Keep the ownership
  summary distinct from resource provenance: HTTP handles can be records of scalar
  fields. Preserve or invalidate summaries through guarded aggregate edits, including
  nested reference/global paths, and test invalidation on unwinding.
- Migrate affected bytecode, linker and VM fixtures, add ownership and graph
  regressions, and document the strong-edge inventory and Rust ownership contract.
  Update `docs/pascal/language/functions/closures.md` and add
  `docs/pascal/program-structure/runtime-ownership.md`; confirm the compact reference.
  No collector or transfer-publication enforcement is introduced in this package.

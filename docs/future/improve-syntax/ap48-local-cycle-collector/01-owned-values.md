# AP48.1: Owned cells and graph classification

Package: [AP48](README.md). Status: not started. Effort: large.
Prerequisite: Q37 recorded and implementation explicitly authorized.

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
- `var` reference reads/writes remain correct and retain their non-escape rule.
- Weak registration neither retains an acyclic cell nor grows without bound as
  ordinary cells are created and released.
- Record the cycle-through-cells argument by value variant and constructor. Any
  counterexample blocks AP48.6 until the collector design is revised.

Extend value/reference and VM task-owned-function tests at their public behavior
interfaces. Update Rust ownership documentation with delivery. Run the applicable
Rust and language checks from the development process; collection stays disabled.

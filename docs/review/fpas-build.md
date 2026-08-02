# `fpas-build` review follow-up

Classification: compiler/build infrastructure and artifact persistence. No FPAS language change expected.
Status: all findings completed.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| BUILD-01 | P1 | `crates/fpas-build/src/engine.rs:101,137,157` | The source hash is read from current bytes while compilation uses the AST cached in `UnitGraph`. A file changed after graph construction can publish old code under the new source hash and then reuse it permanently. | Couple source bytes and parsed AST in one immutable snapshot, or hash the exact graph snapshot. Detect changes before publication and reload or fail. | Build a graph, change a unit, build the same graph, and prove no incorrectly labelled sidecar is published. |
| BUILD-02 | P1 | `crates/fpas-build/src/distribution/tree.rs:60-71` | `replace_tree` deletes the valid destination before staging is renamed. Rename failure loses the previous distribution. | Move the destination to a unique backup, publish staging, rollback on failure, and remove the backup only after commit. Preserve restoration errors. | Failure injection for publish and rollback; old tree must remain usable. |
| BUILD-03 | P1 | `crates/fpas-build/src/program_artifact/mod.rs:32-51`, `program_artifact/identity.rs:7` | Public API accepts `Program` and source bytes independently, allowing an old AST to be cached under new source identity. | Accept one authoritative input and parse internally, or introduce a validated snapshot type coupling AST and bytes. | Deliberately mismatched program/source input must fail before cache lookup or publication. |
| BUILD-04 | P2 | `crates/fpas-build/src/program_artifact/atomic.rs:36,85` | Any lock older than ten seconds is deleted, even when its writer is alive. The first guard can later delete a second writer's lock. | Use an OS lock or owner token with liveness/heartbeat. A guard may remove only the lock it owns. | Real concurrent writers, including a controlled writer held beyond ten seconds. |
| BUILD-05 | P2 | `crates/fpas-build/build/compiler_identity.rs:5` | `fpas-program` is omitted from compiler identity even though it defines program-image encoding and validation. Old images can survive a format-relevant change. | Include `fpas-program` in the fingerprint inputs or define an explicit program-image compatibility identity. | Test the complete set of build-relevant workspace dependencies used for identity. |

## Implementation notes

BUILD-01 and BUILD-03 are the same invariant at different API layers and should be solved with one authoritative snapshot design. Coordinate BUILD-02 and BUILD-04 with `fpas-bundle`, `fpas-std`, and `fpas-unit` to avoid divergent publication semantics.

Existing cold/warm, interface invalidation, corrupt-artifact, option-change, and staging tests provide useful scaffolding. Add the race and rollback cases before changing production code.

## BUILD-02 completion record

Completed on 2026-08-01.

- Implementation: `distribution/publication.rs` now owns staging, backup, publish, rollback, and cleanup. The previous destination is moved to a unique backup instead of being deleted. A failed publish restores it; a failed restore is reported and preserves the backup.
- Structure: tree validation, artifact cleanup, and recursive copying remain in `distribution/tree.rs`.
- Regressions: deterministic tests cover successful restoration and failed restoration with a preserved backup. The end-to-end distribution test also rejects leftover transaction siblings.
- Docs: normative `docs/pascal/` pages are unchanged because FPAS behavior and the documented exact-replacement contract did not change.
- Verification: `cargo fmt`; `cargo test -p fpas-build`; `cargo clippy -p fpas-build --all-targets --locked -- -D warnings`; `cargo build`; `cargo test --workspace`.

## BUILD-01 and BUILD-03 completion record

Completed on 2026-08-02.

- Implementation: filesystem-backed `UnitNode` values retain the hash of the exact bytes used to build the graph. `source_snapshot.rs` rejects a changed graph input, compiles an AST parsed from those same bytes, and rechecks the file before sidecar publication. Parsed overlay graphs remain read-only build inputs because they do not own filesystem snapshots.
- Program API: `build_program_artifact` no longer accepts an independently parsed `Program` or caller-provided unit selection. It parses the authoritative source bytes and resolves reachable units internally before cache lookup, compilation, or publication.
- Regressions: `source_changed_after_graph_creation_is_rejected_without_relabelling_sidecar` preserves the prior valid sidecar. `non_program_source_is_rejected_before_cached_artifact_lookup` proves that the authoritative bytes are parsed before an existing image can be reused.
- Docs: public Rust snapshot contracts were documented. FPAS syntax and semantics are unchanged.

## BUILD-04 completion record

Completed on 2026-08-02.

- Implementation: program publication now uses a stable OS lock file and `atomic-write-file` same-directory replacement. Lock ownership is tied to the open file handle and released by the operating system; no writer removes another writer's lock based on age.
- Regressions: four simultaneous writers publish one complete decodable image. A controlled writer holds the OS lock for 10.1 seconds; the second writer remains blocked and publishes successfully only after the first lock is released.
- Docs: `docs/pascal/program-structure/projects.md` documents the persistent derived `.fpascp.lock` coordination file, and `.gitignore` excludes it.

## BUILD-05 completion record

Completed on 2026-08-02.

- Implementation: `fpas-program` sources and manifest now participate in `FPAS_COMPILER_BUILD_ID`.
- Regression: `compiler_identity_lists_every_build_relevant_workspace_crate` checks the complete explicit identity input set.

## Final verification

- `cargo fmt --all -- --check`
- `cargo test -p fpas-build --locked` — 5 unit, 1 identity, 4 distribution, 6 incremental, and 7 program-artifact tests passed.
- `cargo test -p fpas-project --locked` — all unit, integration, and doc tests passed.
- `cargo clippy -p fpas-build -p fpas-project --all-targets --locked -- -D warnings`
- `cargo build --workspace --locked`
- `cargo test --workspace --locked`

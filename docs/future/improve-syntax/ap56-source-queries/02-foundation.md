# AP56.2: Build the shared core and migrate both adapters

Package: [AP56](README.md). Status: complete; depends on AP56.1.

The concrete encoding and adapter fields are in [the wire contract](wire-contract.md).
The foundation adds `source_inputs/inventory.rs` to `fpas-project`
for detached verification, `analysis/identity.rs` and `build/analysis_identity.rs`
to the language service for shared fingerprints, and their owning tests. The
adapter migrations use the paths in the implementation table below.

## Prerequisites

- [AP56.1](01-contract.md) is complete; Q85, Q86 and Q91 are recorded.
- Existing positional navigation and rename safety regressions remain covered
  through the shared protocol.

## Contract and scope

Deliver the selected schema-2 envelope, context rules, snapshot inventory,
reference encoding, process preparation and coordinator publication path for
existing positional `refs`, `definition`, `type` and `rename`.

Use the same input scope for find/inspect, call queries and refactoring.
Export an owned inventory of
bytes/hashes, manifests, file membership and path aliases from `fpas-project`;
retain its existing bounds and final verification. An operation's incidental
lazy reads must not define reusable identity. Expose semantic results and
completeness through typed language-service data, with wire rendering in the CLI.

Move rename's preparation onto that scope, preserving before/after overlay
analysis, conflict checks, root/Std protection, byte preservation and staging
before replacement. Apply Q86's preparation transport and publication boundary.
Move reusable before/after analysis and edit validation into the common core,
and migrate both CLI rename and LSP rename to it. Source publication remains an
adapter responsibility. Support saved inputs and synchronized multi-file editor
overlays through the same typed snapshot/context interface.
Generic report clipping must never alter edit bytes or identity fields.
AP56.6 later moves rename to the canonical refactor command and adds saved
plans/explicit apply. Keep that route change out of this foundation step.

Selector types belong here, but name/reference CLI forms become available with
AP56.3's complete checked selection. This step must remain independently usable
with positional queries. Do not advertise unavailable find/inspect commands.
Replace old schema/option forms in consumers without a parallel legacy mode.
Implement the shared core build fingerprint and LSP extension registration from
the [adapter contract](adapter-contract.md). Later work packages register only
their delivered operations. Preserve current editor request caching, ordering,
UTF-16 conversion and revision guards; do not force full-inventory queries for
ordinary hover. Core identity must not depend on either adapter executable.

## Affected files and implementation owners

Paths are relative to the repository unless marked package-local.

| Action | Path | Purpose |
| --- | --- | --- |
| Modify/create | `crates/fpas-project/src/source_inputs.rs`, `crates/fpas-project/src/source_inputs/inventory.rs` | Export/recheck owned scope inventory without exposing thread-local state |
| Move/split | `crates/fpas-language-service/src/navigation/source_query.rs` into `crates/fpas-language-service/src/navigation/source_query/` (`mod.rs`, `model.rs`, `scope.rs`, `reference.rs`) | Keep existing query semantics with focused shared types, scope and reference code |
| Modify | `crates/fpas-language-service/src/navigation/mod.rs`, `crates/fpas-language-service/src/lib.rs` | Export the shared semantic query interface |
| Modify as needed | `crates/fpas-language-service/src/analysis/service.rs` | Reuse context analysis without inheriting the editor catalogue's search scope |
| Create/modify | `crates/fpas-language-service/` (`build.rs`, `build/analysis_identity.rs`, `Cargo.toml`), `src/analysis/` (`identity.rs`, `mod.rs`), `src/document/` (`store.rs`, `snapshot.rs`) | Shared build identity and explicit disk/overlay snapshot inputs |
| Create/modify | `crates/fpas-language-service/src/refactoring/` (`plan.rs`, `validation.rs`, `mod.rs`), `src/navigation/rename.rs` | Shared checked edit model and before/after validation used by both rename adapters |
| Modify/create | LSP registration, existing consumer and regression paths in `adapter-contract.md` | Connect existing navigation/rename to shared core primitives; register typed extensions and version guards |
| Modify | `crates/fpas-cli/src/cli_input/query.rs`, `crates/fpas-cli/src/cli_query.rs`, `crates/fpas-cli/src/cli_run.rs`, `crates/fpas-cli/src/main.rs` | Parse the canonical context and unify dispatch, early errors and worker bootstrap |
| Modify/create | `crates/fpas-cli/src/cli_query/` (`request.rs`, `worker.rs`, `report.rs`; new `protocol.rs`) | Typed schema, bounded preparation and one public error path |
| Modify | `crates/fpas-cli/src/cli_query/rename.rs`, `crates/fpas-cli/src/cli_query/publication.rs` | Carry verified inventory and explicit publication/recovery outcomes |
| Modify | `crates/fpas-project/src/source_inputs/tests.rs` | Inventory identity, aliases and membership coverage at their owner |
| Modify | `crates/fpas-cli/tests/source_queries.rs`, `crates/fpas-cli/tests/source_queries/mod.rs`, `crates/fpas-cli/src/main_tests/input/help.rs` | Migrate wire fixtures and cover read-query transport/limits and focused help |
| Modify | `crates/fpas-cli/src/main_tests/query/rename.rs`, publication's existing unit tests | Preserve edit, conflict, permissions and recovery behavior through the new envelope |

Keep files focused and below roughly 500 lines; reuse the existing oneshot/process-tree
supervisor and source publisher instead of introducing another subsystem.

## Migration and documentation

Update focused query help, `docs/pascal/tools/source-queries.md`,
`docs/pascal/program-structure/cli.md` and `docs/pascal/agents/tools.md` with this
delivery. Replace rename's explicit directory/`--project` examples with manifests
and `--context`; preserve implicit positional discovery. Migrate CLI assertions
of `schemaVersion`, `matches`, `type`, stderr errors and publication status.

Check tracked scripts, FPAS tools and editor consumers for CLI JSON dependencies.
LSP navigation consumes typed core results, not the CLI JSON envelope. Migrate
its existing definition/reference/type/rename routes and semantic edit validation
alongside the CLI. Update editor integration and architecture documentation.
This scope adds no FPAS grammar, VM, artifact-format or generated standard-library
changes.

## Acceptance and verification

- Existing definition/reference/type results retain semantic identity and source
  attribution; all operations and invalid requests use the selected envelope.
- Changed source bytes, an unedited analysis input, manifests, membership or
  lexical path bindings prevent a usable reference or unsafe rename publication.
- Read deadlines reap the owned worker. Rename preparation timeout writes no
  source; publication/recovery follows the separately reported boundary.
- Invalid contexts/selectors, conflicts, permissions, staging, replacement,
  recovery and stdout failures remain explicit. Oversized edit reports/plans fail
  before publication. No test asserts an atomic multi-file transaction.
- Syntax/semantic errors and source-less builtins retain distinct incomplete or
  missing outcomes. Read queries execute no FPAS and write no build artifacts.
- Both actual adapter routes observe the same semantic results/blockers on equal
  saved inputs. LSP handles dirty dependencies and stale document versions, and
  never applies CLI disk publication to an editor overlay. Shared core identity
  agrees across executables built from the same analysis inputs.

Extend the existing primary-owner tests rather than adding copies at every
layer. Use failure cases that reach the actual guard, not fabricated receipts.

Compile focused tests separately, then run the corresponding targets:

```text
cargo test --locked -p fpas-project --lib --no-run
cargo test --locked -p fpas-language-service --test navigation_edits --test analysis_state --no-run
cargo test --locked -p fpas-cli --bin fpas --test tools --no-run
cargo test --locked -p fpas-project --lib source_inputs::
cargo test --locked -p fpas-language-service --test navigation_edits
cargo test --locked -p fpas-language-service --test analysis_state query_cache::
cargo test --locked -p fpas-cli --bin fpas query::
cargo test --locked -p fpas-cli --bin fpas input::help::
cargo test --locked -p fpas-cli --test tools source_queries::
```

Include the CLI publication unit tests when selecting the binary target. Follow
the package's build/resource rules and [shared constraints](../shared-constraints.md).
Run applicable formatting, document links and `git diff --check`. Complete
AP56.2 only after migrations and its owning checks pass.
Include LSP target compilation/execution and parity/overlay cases from the
[adapter contract](adapter-contract.md#acceptance-for-every-delivering-work-package).

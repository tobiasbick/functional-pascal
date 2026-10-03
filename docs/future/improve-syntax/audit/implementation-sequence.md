# Bounded implementation sequence

This sequence implements the [settled contract](../language-contract.md) using
the [source map](source-map.md) and [test inventory](test-inventory.md). It is not
another stage checklist. Stage 1 is a source-audit/design deliverable. The shared
model and build/project source-error transport are implemented; the owning
[diagnostics stage](../stages/02-diagnostics.md) records verification and remaining
integration work. Later language slices still require implementation, migration
and executed verification.

## Completed first delivery: shared diagnostic representation

The shared record preserves existing Fxxxx identities and known-location text
output. Parser expectation details, source-read failures and runtime diagnostics
use the same model. The implementation is organized as follows:

- `crates/fpas-diagnostics/src/diagnostic.rs` contains optional spans and
  expected/found details; `codes.rs` contains the audited project/build additions.
- `crates/fpas-diagnostics/src/span.rs` validates byte spans and marks synthetic
  point locations; `source_range.rs` resolves Unicode scalar coordinates.
- `crates/fpas-diagnostics/src/render/json.rs` serializes deterministic records;
  `render.rs` retains text rendering from the same model.
- `crates/fpas-diagnostics/tests/structured_output.rs` and
  `tests/value_invariants.rs` cover ranges, null positions, escaping and codes.
- Parser/runtime producers and compiler/editor consumers handle the shared
  model. Project and build transport is described below.

The first-delivery gate passed: shared schema tests, parser/runtime source
identity, no-source errors and known-location text diagnostics. The public JSON
command option remains open until every command/runner path supports its stream
contract. Preserve source IDs through linked artifacts; do not serialize
formatted terminal prose as diagnostic data.

## Build-error transport before command streams

Implemented: `crates/fpas-build/src/engine/error.rs` retains compiler/parser
records and native linker failures. Unit paths and artifact main paths remain
available without parsing display text; failed artifact parsing retains multiple
diagnostics. The [owning stage](../stages/02-diagnostics.md) records coverage and
verification.

Source records now also propagate through `ProjectError` in project loading,
dependency traversal, standard-library loading and graph/snapshot APIs. The
build boundary preserves these records and their source path.

Implemented: every `ProjectError` and `BuildError` now carries coded records
(F5003–F5034, F9003); `LinkError::code()` assigns linker categories and
`stage_standard_library` returns `BuildError`. Loading warnings are
`FileDiagnostic` records (F5035, F5036), the same type `BuildError` exposes.
This boundary is complete; CLI JSON routing is next.

## Second delivery: CLI and runner streams

Implemented for `check`, `build`, `run` and `test` (see the
[owning stage](../stages/02-diagnostics.md)): `cli_output/diagnostics.rs`,
`cli_output/failure.rs`, coded test-runner output and JSON-mode suppression of
progress lines. The runner reads `FPAS_DIAGNOSTICS=json`, `Std.Proc.Run` child
stderr becomes program-output records, and real-process tests cover both. The
second delivery is complete.

- Extract diagnostic routing from `crates/fpas-cli/src/cli_run.rs` into
  `crates/fpas-cli/src/cli_output/diagnostics.rs`; keep ordinary output handling
  in `cli_output.rs`. This is the first affected oversized-file reorganization.
- Extend `crates/fpas-cli/src/cli_input/`, `cli_check.rs`, `cli_build.rs`,
  `cli_run.rs`, `cli_test/runner.rs`, `cli_test/run/program.rs`,
  `cli_test/process/output.rs` and `bin/fpas-runner.rs` together. Add mode
  transport to the runner without consuming FPAS application arguments.
- Capture program stderr separately from tool diagnostics and wrap it as
  program-output JSON events. Suppress/route progress and test summaries in JSON
  mode. Keep stdout and failure exit codes intact, including runner startup errors.
- Extend `src/main_tests/diagnostics.rs` and actual process tests; split JSON
  cases into `src/main_tests/diagnostics/json.rs` rather than growing one file.
- Extend `docs/pascal/tools/diagnostics.md` with the implemented envelope,
  coordinate unit, null locations, code inventory and all four command examples.
  Update command help and editor adapters to the same source mapping.

Gate: check/build/run/test and real runner all satisfy text/JSON parity,
multiple/imported/no-source errors, Unicode, program stderr and exit handling.
Only then can stage 2 be marked complete.

## Syntax and resolution deliveries

1. **Named blocks and declarations (implemented):** see the
   [block/name delivery](block-syntax-delivery.md). Owners: parser `program.rs`,
   `decl/routines.rs`, `decl/data/const_var.rs`, `decl/data/type_defs.rs`,
   `stmt/branching.rs`, `stmt/loops.rs`, and formatter `emit/stmt/`, `emit/decl/`,
   `emit/program.rs`, `comments/traversal/` under their owning crates. Add null
   bodies and recovery tests. Migrate all source/templates/goldens with matching
   grammar/handbook updates; new later-stage constructs are not reserved early.
2. **Import qualifiers and forward type collection (implemented):** see the
   [block/name delivery](block-syntax-delivery.md). Owners: parser `program.rs`, sema
   `check/entry.rs`, `check/name_resolution/`, `interface/install.rs` and project
   `unit_graph/resolve.rs`. Add focused sema `check/decl/types/collection.rs`
   for whole-unit type headers. Migrate resolved names, qualified variants,
   generated APIs, completion and auto-import together. Keep initializer order.
3. **Operators and bit APIs (implemented):** see the
   [delivery evidence](operator-delivery.md). Owners: parser `expr/precedence.rs`, sema
   `check/expr/operators.rs`, compiler `lowering/expr.rs`, VM `value_ops/integer.rs`.
   Put new bit registration/runtime code in unit-owned modules alongside existing
   `std_registry/loaded/` and `fpas-std/src/` units and include intrinsic catalog,
   bytecode selection and generated declarations. Split boolean lowering into
   `crates/fpas-compiler/src/lowering/expr/boolean.rs` before extending the
   490-line dispatcher. Preserve source evaluation with explicit grouping.
   Deliver side-effect-free bit runtime functions here; checked purity metadata
   and their use from pure callables land with the common stage-5 checker.

Each delivery includes its consumer migration and real execution tests. Source
conversion tools may exist temporarily; no delivered legacy parser mode remains.
The [source conversion verification](source-conversion.md) closes the stage-3
migration item for these three implemented deliveries. Later constructs are
migrated and tested with their owning stage, rather than extending this item.

## Coordinated functional-core and mutation boundary

Build these capabilities in this order; the binding/default migration must not
ship while old implicit caller-mutation intrinsics remain accessible.

1. **Callable targets and metadata:** parser `expr/postfix.rs`, AST expression
   definitions, sema `check/expr/postfix.rs`, compiler `lowering/calls.rs` and
   closure lowering. Move callable descriptors out of sema `types/mod.rs` into
   `types/callables.rs`, and argument checking from `check/calls.rs` into
   `check/calls/arguments.rs` when touched. Include IR call validation, bytecode
   verification, VM invocation, serialized unit interfaces and debugger adapters.
2. **Generic data and decisions:** parser `decl/type_expr.rs`,
   `decl/data/type_defs.rs`; sema `check/name_resolution/types/generics.rs`,
   `check/decl/types/`, `check/stmt/control_flow/if_case/`; compiler
   `lowering/aggregates/records.rs`, `lowering/case/`, `lowering/types/`.
   Add focused expression-decision modules instead of growing expression dispatch.
   Prove finite recursion, constructor inference, private construction, nested
   patterns and expected concrete generic routine values end to end.
3. **References and purity:** add focused sema `check/calls/references.rs` and
   `check/purity/` modules plus matching compiler/VM reference operations. Reuse
   capture declaration identity, but split traversal from capture collection if
   extending `check/closures/capture.rs`. Add storage-root reservation/exclusivity
   and reborrow checks, reference modes, resource classification and pure
   callable metadata in interfaces. Audit intrinsic implementations before
   assigning purity. No purity inference from names or computed const.
4. **Bindings, defaults, numbers and consumers:** migrate const/var and local
   inference in parser declarations and sema `check/decl/consts.rs`, `vars.rs`.
   Update `lowering/calls/arrays.rs` and every public caller-mutating path to
   explicit var; migrate locally mutable parameters to local copies. Correct
   constructor order/default checks, nested copy/equality and all arithmetic
   paths identified in the source audit. Migrate std source/API/callers and
   corresponding handbook/grammar only with working behavior.
5. **Member removal:** once callable fields, Option patterns and invocation work,
   move methods to unit routines and events to ordinary Option fields. Remove
   obsolete sema member resolution, compiler bound-receiver code, AST forms,
   formatter/editor paths and old docs. Preserve private factories and application
   behavior; migrate project exports when routines move.

The implementation may use several internal commits, but the integration gate
requires the whole coupled behavior. Run tests for snapshot/reference aliases,
callback mutation, defaults/purity and nested resources before calling stage 4
complete. The task-scope part of stage 5 follows this boundary.

## Task scope delivery

Add lexical ownership and escape analysis to sema; carry verified helper contracts
through `crates/fpas-unit/src/interface/`. Add scope entry/exit lowering around
`crates/fpas-compiler/src/lowering/concurrency.rs` and every boundary-crossing exit.
Extend IR/bytecode control-flow and verifier rules together, with debug mappings.

Reuse the scheduler and cooperative wakeups under `crates/fpas-vm/src/vm/tasks/`.
Introduce `tasks/scopes/` for lexical owner state and cleanup, rather than mixing
it into `spawn.rs` or retaining public group lifetimes. Change `shared/task_results.rs`
to separate retained values from observation. Migrate `groups/`, `supervision/`,
hosted network/server code, `lib/Std/Tui/Runtime/Application/`, CLI tests and all
public std spawn entry points before deleting detached/group APIs.

Gate: two go forms, repeated waits, missing observation, imported helpers,
container/resource escapes, every exit type, cancellation-aware blocked operations,
panic arbitration and no children after completed exit. A blocked host call can
delay exit; a timeout must not release ownership. Record any unavoidable consumer
policy adaptation in stage 5 instead of keeping a hidden detached escape hatch.

## Domain delivery and completion discipline

Implement distinct types, then integer subranges, then requires/ensures using the
same purity and cleanup mechanisms. Add focused modules under sema
`check/decl/types/` and compiler `lowering/`; extend parser routines/closures and
the shared diagnostic model. Add conversion/contract cases to compiler/CLI tests,
including release execution and imported/indirect calls.

Before each implementation slice, list its exact final create/change/move/remove
paths after refreshing the checkout; the later module proposals here are not a
license for unrelated reorganization. Run the shared verification commands and
record results, remaining work and next step in the owning stage. Never mark an
implementation complete solely because the stage-1 mapping exists.

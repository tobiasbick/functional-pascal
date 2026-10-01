# Bounded implementation sequence

This sequence implements the [settled contract](../language-contract.md) using
the [source map](source-map.md) and [test inventory](test-inventory.md). It is not
another stage checklist. Stage 1 is a source-audit/design deliverable. The first
shared-model slice is implemented; the owning [diagnostics stage](../stages/02-diagnostics.md)
records its evidence and remaining integration work. Later slices still require
implementation, migration and executed verification.

## First delivery: shared diagnostic representation

Start stage 2 with one parser error, one no-source project/build error and one
runtime error carried by the same record. Preserve all existing Fxxxx identities
and text output for errors with known locations. Do not expose a partial JSON
command option before every command/runner path supports its stream contract.

Intended initial file layout (new paths are proposals, not existing modules):

- Modify `crates/fpas-diagnostics/src/diagnostic.rs` for optional source range
  and structured expected/found details; keep severity and code-derived phase.
- Modify `crates/fpas-diagnostics/src/codes.rs` and `code.rs` only for audited
  additions, including a project/build range; keep current codes stable.
- Keep `crates/fpas-diagnostics/src/span.rs` as validated byte spans; add
  `crates/fpas-diagnostics/src/source_range.rs` for resolved optional source
  identity/start/exclusive-end positions. Compute coordinates from source text;
  do not turn a missing location into a synthetic line one.
- Add `crates/fpas-diagnostics/src/render/json.rs`; adapt the existing
  `render.rs` and `lib.rs` exports. JSON and text consume one model.
- Add `crates/fpas-diagnostics/tests/structured_output.rs`; extend
  `tests/value_invariants.rs` for Unicode/end/null behavior and code stability.
- Update producer/adaptor construction sites revealed by compilation across
  lexer, parser, sema, compiler, project/build/linker, VM and language service.
  Keep that mechanical propagation in the same compiling slice.
- Trace existing CLI rendering through `crates/fpas-cli/src/cli_run.rs`
  (emit_diagnostic and render_cli_diagnostic_with_sources), actual bundled
  `src/bin/fpas-runner.rs` and `src/main_tests/diagnostics.rs`. Preserve source IDs
  through linked artifacts; do not serialize formatted terminal prose as data.

Before editing, refresh the diagnostic-code registry and precise constructor
call sites. The new resolved-range representation is an internal design proposal;
it must not duplicate byte-span validation or introduce a second error system.

First-delivery gate: shared round-trip/schema tests, parser/runtime source identity,
no-source errors and unchanged known-location text diagnostics pass. Current
documentation describes only the landed shared behavior; the public JSON option
remains stage-2 work until the next delivery is complete.

## Second delivery: CLI and runner streams

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
- Create `docs/pascal/tools/diagnostics.md` with the implemented envelope,
  coordinate unit, null locations, code inventory and all four command examples.
  Update command help and editor adapters to the same source mapping.

Gate: check/build/run/test and real runner all satisfy text/JSON parity,
multiple/imported/no-source errors, Unicode, program stderr and exit handling.
Only then can stage 2 be marked complete.

## Syntax and resolution deliveries

1. **Named blocks and declarations:** change parser `program.rs`,
   `decl/routines.rs`, `decl/data/const_var.rs`, `decl/data/type_defs.rs`,
   `stmt/branching.rs`, `stmt/loops.rs`, and formatter `emit/stmt/`, `emit/decl/`,
   `emit/program.rs`, `comments/traversal/` under their owning crates. Add null
   bodies and recovery tests. Migrate all source/templates/goldens with matching
   grammar/handbook updates; new later-stage constructs are not reserved early.
2. **Import qualifiers and forward type collection:** parser `program.rs`, sema
   `check/entry.rs`, `check/name_resolution/`, `interface/install.rs` and project
   `unit_graph/resolve.rs`. Add focused sema `check/decl/types/collection.rs`
   for whole-unit type headers. Migrate resolved names, qualified variants,
   generated APIs, completion and auto-import together. Keep initializer order.
3. **Operators and bit APIs:** parser `expr/precedence.rs`, sema
   `check/expr/operators.rs`, compiler `lowering/expr.rs`, VM `value_ops/integer.rs`.
   Put new bit registration/runtime code in unit-owned modules alongside existing
   `std_registry/loaded/` and `fpas-std/src/` units and include intrinsic catalog,
   bytecode selection and generated declarations. Split boolean lowering into
   `crates/fpas-compiler/src/lowering/expr/boolean.rs` before extending the
   490-line dispatcher. Preserve source evaluation with explicit grouping.

Each delivery includes its consumer migration and real execution tests. Source
conversion tools may exist temporarily; no delivered legacy parser mode remains.

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

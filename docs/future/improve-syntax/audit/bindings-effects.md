# Bindings, references and purity integration

This audit continues [stage 4](../stages/04-functional-core.md) with the mutation
and purity contract from [stage 5](../stages/05-effects-and-tasks.md). It records
current ownership and implementation progress, not a smaller acceptance boundary.

## Current owners and required changes

| Concern | Current ownership | Required integration |
|---|---|---|
| Formal parameters | Parser `ast/types.rs`, `parser/decl/routines.rs`; sema `types/callables.rs`, `check/decl/routines.rs`; compiler `lowering/routines.rs` | Carry value/var modes through callable signatures and interfaces. Migrate old locally reassigned parameters to local copies. |
| Arguments | Parser `ast/expr.rs`, `parser/expr/postfix.rs`; sema `check/calls/arguments.rs`; compiler `lowering/calls.rs` | Require actual var markers, check writable roots/paths, preserve target/argument order, and cover indirect/imported calls. |
| Mutable roots | Bytecode `value/mod.rs`; compiler `lowering/context/bindings.rs`, `lowering/stmt.rs`, `lowering/globals.rs`; VM `calls/closures.rs` | Reuse cell allocation identity. Include globals and uncaptured mutable locals in reference access checks. |
| Collection mutation | Sema `std_registry/builtins/array/mutation.rs`; compiler `lowering/calls/arrays.rs`, `closures/array_mutation.rs`; VM array operations and var invocations | Push/Pop use explicit var arguments and ordinary invocation cleanup. Complete the wider intrinsic purity audit. |
| Selected paths | Compiler `lowering/aggregates/`, `lowering/context/designators.rs`; VM `execute/global_index_path.rs`, `value_ops/index.rs` | Freeze/validate indices once, reject all same-root var arguments, and retain partial effects on failure. |
| Purity | Sema callable types, captures and std registrations; unit interfaces | Add explicit capability, transitive resource/capture checks, concrete generic validation, pure defaults and callback requirements. |
| Bindings/static values | Parser `parser/decl/data/const_var.rs`; sema `check/decl/consts.rs`, `vars.rs`; compiler `lowering/stmt.rs`, `globals.rs` | Separate computed const from static classification. Infer locals from their initializer; keep public signatures and top-level bindings explicit. |
| Arithmetic | IR `constants.rs`; VM `value_ops/integer.rs`, `scalar.rs`; compiler optimizations/static evaluation | Check every integer arithmetic path and static invalid operation; preserve IEEE real behavior. |
| Debugger | VM `debug/inspection/`, `debug/mutation/`, `debug/calls/detach.rs` | Preserve reference provenance and live storage restrictions. Detached calls must not acquire live authority. |

Aggregate storage already detaches shared nested values on write. Closures share
capture-cell identity. These mechanisms need interaction tests with source-level
snapshots and references; neither alone proves the caller-mutation contract.

## Implemented storage-root foundation

`crates/fpas-bytecode/src/value/references/` contains:

- `registry.rs`: allocation-based root identity, reservations, exclusive activation,
  atomic access checks, snapshots, writes and release. Reference identities never
  wrap or get reused.
- `borrow.rs`: shared reference authority independent from register copies.
  Reserved children permit only their immediate parent's snapshots; activation
  suspends parent access. Release invalidates retained copies. Drop releases
  abandoned authority and cleans deep parent chains iteratively.
- `selection.rs`: shared root authority with a retained, evaluated field/index
  path. Forwarding preserves the selected path; snapshots copy only its value.
- `path.rs`: one checked slot resolver for reads and writes, including record
  layout/field checks, signed array bounds and structural dictionary key equality.
  Selected writes detach shared aggregates under the root's locks and commit only
  after the entire path succeeds. Superseded values are dropped outside both locks.
- `mod.rs`: explicit access/transition errors; `tests.rs`: positive, negative and
  boundary coverage against actual shared cell storage. `path/tests.rs` and its
  `authority.rs`/`validation.rs` children cover selected storage and snapshots.

The registry in `vm/hosted/mod.rs` is shared by workers and callbacks. VM
CellRead/CellWrite in `vm/calls/closures.rs` use it. Conflicts produce source-mapped
F4026 with a corrective hint; `vm/tests/references.rs` verifies those operation
paths and shared callback state.

Writes take effect immediately. Error/unwind cleanup restores authority without
rolling back stored values. Tests cover nested copy-on-write snapshots, same-root
aliases, disjoint roots, retained copies, nested reborrows and their restoration,
shared worker access, identity exhaustion and 4,096-level cleanup.

The selected-path tests also cover mixed record/dictionary/array updates, frozen
structural keys, unchanged siblings, signed integer bounds, missing keys,
invalid record layouts, read-only strings, failed writes, retained authority
after release, parent/child restoration and atomic updates through cloned
authority. Resource handles and closure capture cells retain their identities
inside value snapshots. No index expression is reevaluated by this storage API.

At the foundation delivery, the reference instructions and call-frame cleanup
were implemented while source modes and pure declarations remained open. The
source-mode delivery below now connects those instructions to source calls.
No stage implementation checkbox is closed by the storage foundation alone.

Selected-path verification: 15 new regressions pass, including positive, negative
and boundary cases. `cargo fmt --all`, `cargo build` and `cargo test --workspace`
passed with 3,696 tests across 188 groups and
zero failures. Changed plan links, Rust formatting and `git diff --check` pass.
Current language behavior and its handbook remain unchanged by this storage API;
source-level caller references require the remaining integration below.

## Implemented IR, bytecode and invocation integration

IR `instruction/references.rs` represents root reservation, field/index selection,
snapshots, immediate writes and release. `validate/operands/references.rs` preserves
the selected type through these operations. `reference_storage.rs` rejects authority
in data storage, callable results and task signatures; ordinary callable values may
describe var modes. Scalar operators were separated into `instruction/operators.rs`
before extending the previously large instruction file.

The bytecode selector emits six checked reference opcodes. A field projection copies
the reference register when its source remains live, then selects the field in place.
Block sizing and instruction/debug address mapping use those emitted words. Function
metadata retains ordered, distinct var argument positions; executable and object
validation share the positional-mode check. Import shapes include those positions,
and linking rejects incompatible modes. Field selection relocates its record layout
through the existing record relocation; the validated field ordinal stays unchanged.
Debug type graphs retain the reference's selected type across object and program
encoding. The current bytecode version is 16 and object schema version is 8.

VM `calls/references/` separates operations, argument checks and frame-owned leases.
Direct, indirect, inline and synchronous callback entries validate modes, VM
provenance, paths and same-root exclusion before activation. Matching reservations
transfer to the callee. Return and execution errors release authority independently
from retained register copies; partial writes remain visible. Explicit release also
removes completed frame leases. Calls with references use ordinary frame transitions
instead of tail-frame reuse. Suspension moves leases with the saved worker state;
dropping that state releases them. References cannot be stored in globals, cells or
compound data, captured, returned, passed as ordinary values or transported to tasks.

Debugger forced exits release discarded frames, and detached evaluation rejects
authority transport. Inspection reads an authorized snapshot or reports unavailable
authority. Frame restart rejects active var invocations because their input authority
cannot be replayed; ordinary frame restart discards pending reservations. Portable
signature comparison and live-image fingerprints include var modes.

Regression coverage includes typed IR validation, operand encoding, executable
round trips, unit-object codec/linking, relocated layouts, direct/indirect calls,
overlapping argument windows, tail calls, forwarding, same-root rejection, disjoint
roots, frozen indices, snapshots, index boundaries, blocked later argument writes,
partial effects on panic, retained copies, callback reuse after success/failure,
saved-state transfer/drop and explicit-release cleanup. These tests exercise internal
IR and bytecode; they do not claim implemented source-level var syntax.

Integration verification: `cargo fmt --all`, explicit formatting checks for the
included IR validation files, `cargo build` and `cargo test --workspace` pass.
The full suite reports 3,726 passing tests across 190 groups, with zero failures.
This integration adds 30 regressions to the selected-path baseline. Canonical
program and bundle expectations are updated for bytecode version 16.

## Remaining integration

1. Complete source-mode interaction verification with the remaining binding and
   purity changes. Modes, selected roots, source maps, standard array mutations and
   old locally reassigned parameter migration are implemented below.
2. Audit intrinsic bodies and implement explicit purity metadata/checks, including
   ordinary/pure conversions, recursive resources, generics, local mutation,
   captures, higher-order APIs and defaults.
3. Migrate bindings, inference, static classification, checked arithmetic and
   affected standard APIs/consumers together.
4. Replace members with ordinary routines and Option handler fields; remove stale
   syntax/resolver/AST/runtime/formatter/editor/documentation paths.
5. Execute the full stage-4 interaction matrix and shared verification. Keep the
   approved procedure-task/bare-task migration boundary with stage 5.

Foundation checks: `cargo fmt --all`, `cargo build`,
`cargo test -p fpas-bytecode references`, and `cargo test -p fpas-vm references`.
The foundation's full `cargo test --workspace` run passed 3,670 tests across 188
groups, with zero failures. This includes 21 new foundation regressions. Changed documentation
links and `git diff --check` pass. Source-level integration and final application
verification remain open; these results do not close the stage-4 acceptance gate.

## Authorized correction: read-only string-index assignment

During the writable-path audit, this current-syntax program reproduced a baseline
error with both the real CLI `check` and `run` commands:

```pascal
program StringWrite;
uses Std.Console as Console;
begin
  mutable var Text: string := 'abc';
  Text[0] := 'z';
  Console.WriteLn(Text);
end program;
```

Before the correction, both commands reported F9001, `index assignment on
non-collection`, rather than a source-level target diagnostic. Sema resolved the
indexed character as `string`; the assignment target check considered only the
root's mutability. `check/stmt/assignment.rs` therefore accepted the target. Compiler
`lowering/aggregates/paths.rs` supports array/dictionary writes and rejects this
string target as an internal invariant failure.

At reproduction, all three affected source files matched the preceding commit.
The new VM reference registry does not participate in this failure, which occurs
during compilation.
The implemented string indexing documented in
[operators](../../../pascal/language/basics/operators.md#string-indexing) reads
character values; it does not provide mutable character storage.

The user authorized correcting this finding. Semantic assignment checking now
rejects the first string index in a target path with F2005, `String indices are
read-only`, and a hint to assign a whole replacement string. It uses the checked
projection types, without analyzing index expressions again. Invalid target paths
keep their existing diagnostics; immutable string roots receive one read-only
index diagnostic.

The regression modules `crates/fpas-sema/src/tests/stmt/assignment/string_indices.rs`
and `crates/fpas-cli/src/main_tests/diagnostics/string_indices.rs` cover direct and nested
writes, generic records and aliases, globals, parameters, captures, imported
storage, source locations and JSON diagnostics. Positive cases preserve Unicode
character reads, whole-string replacement in bindings/fields/collections and
nested value snapshots. Both CLI `check` and `run` reject invalid writes during
semantic analysis. The operator documentation states the read-only rule.

Correction verification: all 11 new regressions pass. `cargo fmt --all`,
`cargo build` and `cargo test --workspace` pass, with 3,681 tests across 188 groups
and zero failures. The original reproduction produces F2005 with both real CLI
commands. Changed documentation links, Rust documentation references and
`git diff --check` pass.

This resolves the string-index finding; the remaining stage-4 source-level
binding, mutation and purity work stays open.

## Authorized correction: immutable imported storage

The next source-mode audit reproduced a separate current-language error through
the real CLI. The unit exports an immutable binding:

```pascal
unit App.State;
public var Count: integer := 1;
end unit;
```

The importing program assigns through its declared qualifier:

```pascal
program Main;
uses App.State as Store;
uses Std.Console as Console;
begin
  Store.Count := 2;
  Console.WriteLn(Store.Count);
end program;
```

The reproduction is retained under the ignored scratch directory
`.temp-data/bindings-effects-audit/immutable-import/`, with an `app.fpasprj`
listing the program and unit. Both CLI commands were run on that project.

| Case | `fpas check` | `fpas run` |
|---|---|---|
| Immutable imported scalar assignment | Exit 0 | Exit 2, F4008 |
| Immutable imported array element assignment | Exit 0 | Exit 2, F4008 |
| Immutable imported record field assignment | Exit 0 | Exit 2, F4008 |
| Mutable imported scalar assignment | Exit 0 | Exit 0, prints `2` |
| Immutable local scalar assignment | Exit 1, F2005 | Exit 1, F2005 |

The current [binding rules](../../../pascal/language/basics/variables.md) require
immutable reassignment to fail at compile time. Runtime F4008 protects the global
slot, but does not satisfy semantic checking or the CLI `check` contract.

In `check/stmt/assignment.rs`, the mutability gate looks up only the first written
segment (`Store`). That segment is an import qualifier, not the resolved storage
root. The gate is skipped. `check/expr/designator.rs` already resolves qualified
value prefixes for typing, but `designator_is_mutable_target` also checks only the
first segment. Both faulty checks are present in the preceding commit; the new
reference registry and internal var modes do not cause this finding.

Implementation stopped under the user's stop-on-new-findings instruction.
The incomplete source-mode edits were removed; the previously verified reference
implementation remains in place. `cargo fmt --all` and `cargo check --workspace`
pass after that restoration. No source var feature or stage item is marked complete.

The user authorized correcting the finding and continuing the plan. The shared
resolver in `check/expr/designator/roots.rs` now returns the actual storage symbol
and consumed prefix length. Typing, string-index permission checks and ordinary
assignment permissions use that same resolution. Alias qualification preserves
the storage root's mutability and symbol kind. Invalid paths keep their existing
diagnostics without an additional immutable-assignment error.

Four sema regressions cover mixed-case aliases, nested generic record/array/dict
paths, signed index boundaries, nested routines, private fields, unknown indices,
mutable roots and independent copies. Semantic interfaces are encoded and decoded
before testing imports. Two CLI regressions verify `check` and `run` against real
unit projects, located JSON F2005 diagnostics, and nested copy isolation alongside
mutable imported updates. All six targeted regressions pass. The binding handbook
explicitly documents qualified roots and independent mutable copies.

Correction verification: `cargo fmt --all`, `cargo build` and
`cargo test --workspace` pass with 3,732 tests across 190 groups and zero failures.
The retained scalar reproduction now reports F2005 and exit 1 with both CLI
commands. This resolves the imported-storage finding; the stage-4 acceptance
items remain open.

## Source consumer migration: local parameter copies

`lib/Std/Http/Sse.fpas` no longer declares locally mutable formal parameters.
`EmitEvent`, `Process` and `FailedState` accept read-only initial values and create
explicit mutable local copies. The returned events/state and retained decoder
handle behavior are preserved; these parameters do not become caller references.
The source-defined standard units now contain no `mutable Name: Type` formals.
The public HTTP/SSE API and its handbook require no change for this internal
migration.

`fpas fmt --check lib/Std/Http/Sse.fpas` and all 16 tests under
`tests/stdlib/net/` pass, including parallel decoders, terminal error-state cleanup,
final-event size boundaries and UTF-8 handling. Existing coverage is used because
the source migration preserves behavior. Compiler modes, bindings and purity
still require integration.

The same migration is applied to `mutable_nested_functions.fpas`,
`local_index_write_benchmark.fpas`, `callable_targets_test.fpas` and the debugger's
`cell_capturing_routine_assignment.fpas` fixture. Compiler and CLI closure sources
also use explicit local copies, including the exported factory routine. Debugger
array/dictionary/variant tests mutate initialized local copies and retain their
existing result, capture-cell, error and cancellation assertions. Large debugger
test files are split into focused storage-root and limits modules before migration.

All source-defined standard units, examples, applications and FPAS fixtures now
contain no old locally mutable formal parameters. Legacy parser, formatter,
signature and member-validation fixtures remain until their corresponding source
paths are replaced. This is a consumer migration, not implemented source-level
caller references. Existing closure tests, the clamp example, 17 VM debugger tests,
four protocol debugger tests and 17 storage-mutation tests pass. The HTTP fluent
integration test also passes. The public standard API remains unchanged.

Consumer verification: `cargo fmt --all -- --check`, explicit formatting checks
for included IR files, `cargo build` and `cargo test --workspace` pass. The full
workspace reports 3,732 passing tests across 190 groups, with zero failures.
`fpas fmt --check` passes for all five migrated FPAS files. The complete
`tests/suite.fpasprj` run reports 466 passed, one intentional skip and zero failures.
Changed documentation links and `git diff --check` pass. No stage acceptance item
is closed by this migration; source modes, bindings/inference and purity remain
open.

## Source modes and explicit array mutation

Source formal parameters and callable type parameters use `var`; actual arguments
use `var Target`. The old locally mutable formal spelling produces a localized
migration diagnostic. One shared semantic checker validates writable storage,
selected paths, exact reference types, modes, duplicate roots and generic arguments.
Record routine calls also use that checker. An implicit receiver cannot supply
`var Self`; caller mutation uses an ordinary routine with an explicit argument.
Anonymous and named routines cannot capture reference authority, and task creation
cannot transport it. Explicit value snapshots remain eligible for capture.

Compiler `lowering/references.rs` reserves the resolved cell root and evaluates
selected field/index paths once before later arguments. Reference parameters keep
their logical value types for ordinary reads and writes while retaining raw
storage for forwarding. Addressed locals and mutable globals use stable cells.
Branching indices retain authority through internal reference spills. Captured
callable targets are selected before argument evaluation. Interfaces, artifact
codecs, imported signatures and linker checks retain var modes.

`Std.Arrays.Push/Pop` use the same semantic storage checker. Concrete synthetic
entries in `lowering/closures/array_mutation.rs` have ordinary var signatures;
existing invocation activation and cleanup therefore apply to native updates too.
The entries read a value snapshot, use existing array operations, and immediately
write the updated array through active authority. Their operations retain the
source call span without introducing extra debugger sequence points. The previous
simple-variable-only and implicit receiver lowering paths are removed. Standard
units, applications, examples, Rust fixtures and FPAS consumers pass explicit var
arguments. The array handbook and generated intrinsic API declarations agree.

Debugger storage inspection and mutation use logical cell/reference snapshots.
Authorized var parameters can be edited, while aliases blocked by an active
reference display as unavailable. Global data watches follow cell and reference
writes, including unchanged-value filtering. Synthesized global cell allocation
does not add a source stop before initialization.

Regression coverage includes parser/formal diagnostics, exact modes and writable
paths, generic forwarding, indirect/returned/indexed/field calls, compiled-unit
and program reuse, nested value-copy isolation, frozen branch indices, alias
conflicts during argument evaluation, record-visible var parameters, debugger
edits and data watches. Native array regressions additionally cover forwarded,
field, nested-array and dictionary storage, reservation-before-side-effect order,
readonly/missing/wrong-type arguments, task escape and located empty-pop failure.
The current handbook describes this implemented source behavior. Local binding
inference, static classification, checked arithmetic, purity and member removal
remain required; no stage-4 acceptance checkbox is closed by this delivery.

Source-mode verification: `cargo fmt --all`, explicit formatting of the included
IR validation cases, `cargo build` and `cargo test --workspace` pass with 3,768
Rust tests and zero failures. `fpas fmt --check examples tests apps lib` passes.
The complete FPAS bundle reports 468 passed, one intentional skip and zero
failures. All 22 example/application manifests pass; their project contexts cover
14 entry files that need declared units/dependencies, while the remaining 95
programs pass as loose sources. All 57 complete handbook programs pass, including
three examples checked with their documented companion units. Changed document
links and `git diff --check` pass.

The full runs exposed two migration consistency gaps: generated parameter
comments retained the `var` marker as part of a name, and native receiver metadata
needed an explicit negative entry for the caller-mutating array APIs. Both are
corrected with regressions. Array storage, compiler/runtime results and earlier
source behavior regressions pass after these corrections. The following work
continues with computed const bindings, local initializer inference and static
classification, coordinated with checked numbers and purity.

## Authorized baseline correction: alias-only selected storage

The binding audit found a remaining baseline bypass of the stage-3 import rule.
A direct identifier chain already called `resolve_source_name`, but a chain with
an index went directly through value-prefix resolution. For example,
`App.State.Writable.Values[0]` could read or write imported storage even when the
only source qualifier was declared as `uses App.State as Store;`. The same bypass
made a forbidden full-name path eligible for a new var argument. The preceding
commit's `resolve_designator_base` uses the identical ungated prefix lookup; this
finding predates the storage-root permission correction and source var modes.

Under the user's authorization to fix baseline errors, selected-path typing now
validates the resolved storage prefix with the existing source-name checker before
projection. Permissions and types still share the same resolved symbol. There is
no second import access path. The current alias-only specification is unchanged.
The caller-reference duplicate-root test now uses mixed-case spellings of its
one declared alias instead of relying on the forbidden full unit name.

Sema regressions cover reads, writes and indirect var arguments across arrays,
record fields and nested dictionary/array storage, including mixed-case names and
interface encoding/decoding. The real CLI verifies located JSON F2003 with both
`check` and `run` for reads, writes and native array var arguments. Existing valid
alias paths and readonly-root diagnostics remain covered.

The negative tests also exposed an extra diagnostic introduced during source var
integration: a rejected reference argument was rechecked as an ordinary value
when its error type appeared to contain inference holes. Contextual rechecking
now applies only to value arguments; reference storage already has its declared
type and mode. Invalid reference roots retain their original single diagnostic.
All 558 sema tests and the targeted CLI alias/root regressions pass. `cargo fmt`,
`cargo build` and the full Rust workspace pass (3,770 tests across 191 groups).
FPAS source formatting and the Git diff check pass. Binding/inference integration
continues after this verified correction.

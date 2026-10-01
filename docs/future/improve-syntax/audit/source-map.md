# Stage 1 source audit

This is implementation mapping for the [language contract](../language-contract.md),
not a second language specification or completion index. Source inspection used
revision `2303e837` with a clean working tree. Findings below are source-backed;
no build or executable test was run for this documentation-only audit. Existing
test names are evidence of test intent, not a claim of passing execution.

Read this with the [test inventory](test-inventory.md) and
[implementation sequence](implementation-sequence.md). Paths in code spans are
repository-relative. These findings describe the stage-1 baseline; current
diagnostic implementation and verification are tracked in
[stage 2](../stages/02-diagnostics.md). Recheck the affected implementation before
each slice.

## Values, mutation, and resources

| Contract | Existing implementation and reusable behavior | Required change / verification boundary |
|---|---|---|
| Isolated value copies, identity-preserving resource copies | `crates/fpas-bytecode/src/value/mod.rs` clones scalars inline and shares aggregate storage. `value/array.rs` and `value/aggregate.rs` detach through copy-on-write; cells share `Arc<Mutex<Value>>`. | Reuse storage, but verify nested writes through every field/index/cell/global path. Copy-on-write alone does not establish source-level snapshot or exclusive-reference rules. |
| Resources remain resources through containment | `crates/fpas-sema/src/types/mod.rs` distinguishes channels and tasks, but many host handles are registered as empty records in `std_registry/loaded/channel_task.rs`, `net.rs`, and `server.rs`. Runtime `Value::OpaqueHandle` carries identity; hosted network/server/console code interprets it. | Introduce common semantic resource classification rather than treating every record as pure data. Cover channels, task/cancellation/selection handles, network connections/listeners, server lifetimes and console handles, including nested containers and later distinct wrappers. |
| Computed const, mutable var, local inference | `crates/fpas-sema/src/check/decl/consts.rs` requires static initialization. `decl/vars.rs` resolves an explicit annotation and has a special bare-task inference path. | Separate static-expression classification from binding mutability; replace the bare-task exception with ordinary local initializer inference. Keep explicit top-level, field and routine types. Do not infer from later assignments. |
| Initialization order and forward types | `crates/fpas-sema/src/check/entry.rs` checks declarations in order. `decl/types/records.rs` installs a placeholder for the record currently being checked. `crates/fpas-project/src/unit_graph/resolve.rs`, `crates/fpas-build/src/source_snapshot.rs`, and `crates/fpas-linker/src/emit/constants.rs` participate in cross-unit builds. | A current-record placeholder is not whole-unit forward type resolution. Separate type declaration collection from value initialization; reject alias/layout cycles and initialization dependency cycles without reordering effects. Preserve project graph ownership. |
| Read-only snapshots versus caller var references | `crates/fpas-sema/src/check/decl/routines.rs` resolves existing parameter metadata; `crates/fpas-compiler/src/lowering/calls.rs` lowers values. Existing `mutable` parameters describe local mutation. | Add explicit reference modes through AST, sema, unit interfaces, IR, bytecode and VM. Existing local mutation must become a local copy, never silently become caller mutation. |
| Intrinsic caller mutation | `crates/fpas-sema/src/std_registry/builtins/array/mutation.rs` requires simple mutable variables for Push/Pop. `crates/fpas-compiler/src/lowering/calls/arrays.rs` has local ArrayPush/ArrayPop operations and cell/global read-modify-write paths. `calls/fluent.rs` is another call route. | Replace these special call rules with explicit var arguments; cover direct, fluent migration, indirect and imported calls. Resolve root/indices once, reserve during argument evaluation, check root identity and reborrows, retain partial effects on failure. Resource operations do not replace the handle binding. |
| Closure identity and transfer | `crates/fpas-sema/src/check/closures/capture.rs` records declaration identity, mutable capture and transitive task-bound state; `closures/capability.rs` propagates mutable/transitive captures. `crates/fpas-bytecode/src/value/function.rs` shares environments and tracks task ownership. | Reuse cells and task ownership. Computed const capture, pure captures, forbidden var-parameter capture and containment-wide transfer checks need coordinated changes. Retain fresh per-iteration bindings and verify escaping loop closures. |

Value-path owners beyond these entry points are
`crates/fpas-compiler/src/lowering/aggregates/paths.rs`,
`aggregates/global_index_path.rs`, `context/bindings.rs`,
`crates/fpas-vm/src/vm/execute/aggregates.rs`, and
`crates/fpas-vm/src/vm/value_ops/index.rs`. Debugger writes under
`crates/fpas-vm/src/vm/debug/mutation/` must follow the same storage restrictions.

## Numeric and equality differences

- `crates/fpas-vm/src/vm/value_ops/integer.rs` wraps addition, subtraction and
  multiplication. Negation, integer division and remainder are checked. Shift
  counts are already restricted to 0..63, but signed `wrapping_shr` is not the
  target zero-filling right shift.
- `crates/fpas-compiler/src/optimize/constant_folding.rs` also wraps integer
  addition/subtraction/multiplication. Failed checked folds return no folded
  value; a static-context rejection must not depend solely on that optimizer.
- `crates/fpas-vm/src/vm/value_ops/scalar.rs` rejects real division by zero;
  `execute/typed.rs` routes zero denominators out of its real fast path.
  Constant folding also excludes real zero division. All paths must adopt the
  target IEEE division rules together and keep integer zero division distinct.
- `crates/fpas-sema/src/check/expr/equality.rs` rejects arrays/dictionaries in
  structural data and has an early Option/Result acceptance path. Audit nested
  unsupported equality rather than assuming wrapper validation is recursive.
- `crates/fpas-bytecode/src/value/equal.rs` compares dictionary pairs in insertion
  order and supports internal task/handle/callable equality. These internal
  comparisons are not evidence that source equality should admit those types.
  Source semantics need mapping equality and recursive resource/callable rejection.
- The same file compares real bit patterns, while VM scalar real equality in
  `value_ops/comparison.rs` uses numeric equality. Specify regression expectations
  from the existing target IEEE/component rules for NaN and signed zero in nested
  values; do not accidentally export an internal equality shortcut.

Keep integer limits, shifts, conversions, scalar/aggregate real comparisons and
optimized/unoptimized execution in one behavior-level test matrix. No new numeric
semantics are decided by this audit.

## Calls, types, decisions, and names

| Contract | Concrete owners / inspected foundation | Gap and migration dependency |
|---|---|---|
| Arbitrary callable targets; target then positional arguments once, left to right | `crates/fpas-parser/src/parser/expr/postfix.rs`, `crates/fpas-sema/src/check/expr/postfix.rs`, `check/calls.rs`, `crates/fpas-compiler/src/lowering/calls.rs` | Parser calls start from a designator; postfix operations cover field/index/method, not general invocation. Add a common callable operation before removing member machinery. Preserve procedure/value distinction and compare callable signatures without parameter-name identity. |
| Record construction, default purity/order and private fields | `crates/fpas-sema/src/check/decl/types/records.rs`, `check/record_visibility.rs`, `check/decl/vars.rs`; `crates/fpas-compiler/src/lowering/aggregates/records.rs` | Defaults are type-checked without a pure capability. Construction currently resolves supplied fields through a map and iterates declaration/default order. Stage 4 must evaluate supplied fields in written order, then missing defaults in declaration order. Reuse visibility checks; do not expose fields to simplify migration. |
| Record updates | `crates/fpas-sema/src/check/expr/record_fields.rs`, `crates/fpas-compiler/src/lowering/aggregates/records.rs` | Reuse contextual field typing and copy-update lowering; preserve base-once and written replacement order while changing the closer. Test duplicate/unknown/inaccessible fields. |
| Generic data, recursive finite types, constraints | `crates/fpas-parser/src/parser/decl/data/type_defs.rs` explicitly rejects generic type definitions; `decl/type_expr.rs` and `decl/routines.rs` own applications/routine generics. `crates/fpas-sema/src/check/name_resolution/types/generics.rs` and `types/mod.rs` own current generic typing. | Build generic records/enums and finite-layout checking. Replace angle brackets and special built-in application forms with parenthesized of lists. Add Equatable separately from Comparable. Keep supported dictionary-key restrictions explicit. |
| Generic routine values and type inference | `crates/fpas-sema/src/tests/generics/bodies.rs` currently rejects coercion of a generic function value to a concrete signature; `check/calls.rs` owns call checking. | Instantiate from a concrete expected callable type; routine calls infer from actuals. No call-site type argument notation or first-class polymorphism. Constructor type context must not become a search for unqualified variants. |
| if/case values, nested patterns, guards and coverage | `crates/fpas-parser/src/parser/stmt/branching.rs`; `crates/fpas-sema/src/check/stmt/control_flow/if_case/` (bindings, labels, exhaustiveness); `crates/fpas-compiler/src/lowering/case/` | Reuse statement checking/lowering, extend value positions and nested patterns. Const bindings differ from constant labels; enforce explicit variant coverage, guard gaps, grouped-binding agreement, unreachable arms and open-domain expression fallback. |
| Explicit imports, one namespace, forward types | `crates/fpas-parser/src/parser/program.rs`, `crates/fpas-sema/src/check/entry.rs`, `check/name_resolution/`, `interface/install.rs`; `crates/fpas-project/src/unit_graph/resolve.rs` | Remove implicit short aliases and secondary full-name access after resolved consumer migration. Reserve aliases against shadowing and duplicates; preserve case-insensitive type/routine collision detection and imported visibility. |
| Named closers, mandatory terminators, one declaration per keyword | `crates/fpas-parser/src/parser/program.rs`, `decl/data/const_var.rs`, `decl/data/type_defs.rs`, `decl/routines.rs`, `stmt/` | Rework statement lists and recovery together with formatter/comment traversal. Empty bodies need null; plain blocks retain independent lexical scope. Expression closers do not add statement semicolons. |
| Boolean precedence and short-circuiting | `crates/fpas-parser/src/parser/expr/precedence.rs`, `crates/fpas-compiler/src/lowering/expr.rs` | Preserve comparison-chain rejection. Current And/Or/Xor all use lower_direct_binary, which lowers both operands; add branch-based short-circuit evaluation for boolean and/or. Comparisons move above not; mixed logical chains require grouping; integer bit operations move to Std.Bits with checked counts. |
| Ordinary routines replace members | `crates/fpas-sema/src/check/expr/calls/methods.rs`, `calls/fluent.rs`, `bound_method.rs`, `event_access.rs`, `check/decl/types/record_properties.rs`, `record_events.rs`; compiler `lowering/closures/bound_methods.rs` | Migrate methods, receiver insertion, properties and events after callable fields/generic Option patterns work. Rename collisions through resolved symbols and retain factory visibility. Remove event nil/Assigned and old formatter/editor paths in the same delivery. |
| Consumed values, discard and Result forwarding | `crates/fpas-sema/src/check/stmt/calls.rs`, `crates/fpas-compiler/src/lowering/expr.rs`, `crates/fpas-compiler/src/tests/aggregates/try_expressions.rs` | Require consumption of ordinary function results; add discard, reject procedures and transitively task-containing values. Reuse try propagation without converting panics or inferring routine result types. |

## Purity, tasks, and domain rules

| Contract | Owners / foundation | Required extension |
|---|---|---|
| Pure declarations and callable capability | `crates/fpas-sema/src/types/mod.rs`, `check/decl/routines.rs`, `check/closures/`, `std_registry/`; `crates/fpas-unit/src/interface/types.rs` | No purity capability in current callable types. Add transitive resource/capture checks, local-only mutation, pure-to-ordinary conversion and concrete generic validation. Export verified metadata across unit interfaces, not an unchecked annotation. |
| Every spawn has a lexical owner | `crates/fpas-compiler/src/lowering/concurrency.rs`, `lowering/closures/intrinsic_tasks.rs`; `crates/fpas-vm/src/vm/tasks/spawn.rs`, `groups/spawn.rs`, `supervision/execution.rs` | Ordinary spawn explicitly accepts a detached flag and uses SpawnDetachedTask. Group and supervised spawning are additional entry points. Introduce scope ownership across all routes; a nested routine must create its own scope. |
| Observation and repeatable Wait | `crates/fpas-vm/src/vm/shared/task_results.rs`, `tasks/scheduler.rs`, `tasks/scheduler/result_polling.rs` | Current states include consumed results. Retain successful results for repeated waits under copy/identity rules; track observation separately from joining. Reject dropped handles statically where obvious and diagnose remaining unobserved results after normal join. |
| Cancellation, joining and panic precedence | `crates/fpas-vm/src/vm/tasks/groups/close.rs`, `groups/registry.rs`, `cancellation.rs`, `channel/`, `supervision/`; hosted network/server code | Group close already keeps ownership until join and supports cooperative cancellation. Normal lexical exit must join without automatically using cancel-on-close. Abnormal exits cancel/join; preserve body panic, attach later panics, distinguish returned Result.Error and cancellation. No timeout may certify a completed scope with surviving children. |
| Escape and helper contracts | `crates/fpas-sema/src/check/expr/task_bound.rs`, `check/closures/capability.rs`, `crates/fpas-unit/src/interface/types.rs`, `crates/fpas-vm/src/vm/tests/task_owned_functions.rs` | Existing task-bound closures are a foundation, not lexical handle escape analysis. Track scope provenance through containers, closures, globals, returns, channels and resource stores; verify synchronous helper behavior across imports and reject unknown retaining indirect calls. |
| Distinct and subrange types | `crates/fpas-parser/src/parser/decl/data/type_defs.rs`, `decl/type_expr.rs`; sema `check/decl/types/`, `check/expr/operators.rs`; compiler `lowering/types/`, `lowering/expr.rs`; VM `value_ops/` | Add nominal identity, explicit checked conversion, static bounds, widening and membership. Preserve resource restrictions through wrappers; no implicit narrowing or inherited resource operations. |
| Routine requires/ensures | Parser `decl/routines.rs` and `expr/closure.rs`; sema `check/decl/routines.rs`; compiler `lowering/control_flow.rs`; VM `diagnostics.rs` | Add clauses on definitions, reuse purity checks, bind clause-local results and snapshot parameter inputs. Instrument indirect calls and all normal returns including Result errors, after successful scope cleanup. Keep mandatory release checks and bounded non-user-code rendering. |

## Grammar and handbook map

The current grammar is `docs/specs/grammar.ebnf`. Production names below are the
current anchors to replace or extend, not claims that the target already parses.
Handbook paths in the last column are relative to `docs/pascal/`.

| Area | Current productions | Current documentation to migrate with implementation |
|---|---|---|
| Bindings/initialization | const_block, const_def, var_block, var_def, mutable_var_block, var_stmt | language/basics/constants.md, variables.md, local-variables.md; program-structure/initializing.md |
| Imports/declarations | uses_clause, program_declaration, unit_declaration, type_block | program-structure/; language/types/type-aliases.md |
| Closers/terminators | program, unit, main_block, block, statement_list, if_stmt, case_stmt, case_arm, for_stmt, for_in_stmt, while_stmt, repeat_stmt | language/control-flow/; language/functions/declarations.md; tools/fmt-style.md |
| Calls/captures/references | function_decl, procedure_decl, param_group, formal_type_param, closure_expr, postfix_suffix, call_args, arg_list | language/functions/parameters.md, mutable-parameters.md, function-types.md, closures.md, nested.md, first-class.md, postfix-chaining.md |
| Generic data and construction | type_def, type_params, constraint_name, type_expr, record_type, field_def, enum_type, record_literal, record_update | language/types/generics.md, records.md, enums.md, arrays.md, dictionaries.md, result-option-types.md, record-update.md; language/functions/generic-routines.md |
| Removed members | record_method, record_property, record_event, designator_root | language/types/record-methods.md, record-properties.md, record-events.md; language/functions/fluent-calls.md |
| Operators/decisions | comparison_expr, additive_expr, mult_expr, unary_expr, primary_atom, case_label, pattern_arg, destructure_label | language/basics/operators.md, primitive-types.md; language/pattern-matching/; language/error-handling/ |
| Tasks/purity | go_call, go_stmt, type_expr, function_type, statement | language/concurrency/; std/concurrency/task.md; std/collections/array/mutating.md; new purity/scope pages only when implemented |
| Domain/contracts | type_body, type_expr, comparison_op, function_heading, procedure_heading, closure_expr | language/types/type-aliases.md and new distinct/subrange pages; language/functions/ and error-handling/panic.md |
| Diagnostics | No new language production; command option only | tools/ diagnostic reference to create during stage 2 |

## Consumer and artifact migration

- Migrate source under `lib/Std/`, `apps/`, `examples/`, `tests/` and generated
  `lib/api/Std/`. Inventory Rust-embedded source in all crates, not just parser
  tests. Formatter goldens and editor fixtures are also language consumers.
- Concrete task consumers include `lib/Std/Tui/Runtime/Application/Background.fpas`
  and `Loop.fpas`, `examples/network/tcp_parallel_echo_server.fpas`,
  `examples/pascal/concurrency/task_group_workers.fpas`,
  `examples/pascal/concurrency/supervised_worker.fpas`, and the mandelbrot/julia
  renderer examples. Preserve their intended service lifetime under explicit owners.
- `lib/api/Std/Server.fpas` exposes group-dependent lifecycle operations;
  `tests/stdlib/server/lifetime_test.fpas` and `failure_test.fpas` must migrate
  with `Std.Tasks`, not after removing group APIs.
- `crates/fpas-cli/src/cli_init/templates.rs`,
  `editors/vscode/snippets/fpas.json`, `editors/vscode/syntaxes/fpas.tmLanguage.json`,
  `crates/fpas-language-service/src/intellisense/completion.rs` and `auto_import.rs`
  must emit only implemented canonical syntax. Include repository FPAS skills.
- Carry callable modes, purity, resource kind and non-escape guarantees through
  `crates/fpas-unit/src/interface/`, `crates/fpas-sema/src/interface/`, build
  validation and linker metadata. Rebuild incompatible source-adjacent artifacts;
  do not add a cache/package model or commit generated `.fpascu` files.
- Debugger function/sequence/task assignment tests and LSP diagnostics/signature
  tests must consume the new metadata. A VM-only restriction is insufficient.

## Structural constraints

Measured source sizes at this audit: `crates/fpas-cli/src/cli_run.rs` 415 lines,
`crates/fpas-sema/src/types/mod.rs` 429, `check/closures/capture.rs` 436,
`check/calls.rs` 418, and `crates/fpas-compiler/src/lowering/expr.rs` 490.
Do not append entire new capabilities to those files. Split their affected
concern when its slice starts, as specified in the implementation sequence.
Existing parser/check/lowering subdirectories are useful ownership boundaries.

The diagnostic record (167 lines) and text renderer (93 lines) are already small;
extend the shared model and add focused output modules rather than replacing the
diagnostic system. No source reorganization is necessary for this audit itself.

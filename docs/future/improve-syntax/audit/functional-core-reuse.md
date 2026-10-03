# Stage 4 functional-core reuse audit

This records the reuse boundary for the first work item in
[stage 4](../stages/04-functional-core.md). Delivered callable behavior, module
splits, consumers and executed verification are recorded separately in the
[callable delivery](functional-core-callables.md).
The inspected checkout is `69c9b14a4ea2` on `codex/syntax-changes`, with no local
changes before this audit. Full paths are repository-relative; abbreviated paths
use the owning crate/directory identified in the same paragraph or table row.

The observations describe inspected source and tests. Tests were not executed for
this documentation-only delivery. No parser, language behavior, current handbook,
or source consumer is changed. The [language contract](../language-contract.md)
and [bounded sequence](implementation-sequence.md) determine the target; existing
behavior below is reusable only where it satisfies that contract.

## Reuse and gaps by owner

| Area | Existing owner and reusable foundation | Remaining work |
|---|---|---|
| Parser and AST | `crates/fpas-parser/src/ast/expr.rs`, `ast/types.rs`, `parser/expr/postfix.rs`, `parser/expr/closure.rs`: explicitly typed anonymous routines, designator calls, ordered field/index/method suffixes, record updates | Calls store a `Designator`; postfix operations have no general invocation suffix. Add calls on expression values and procedure statement targets. AST has no decision expressions or recursive pattern tree. |
| Callable checking | `crates/fpas-sema/src/check/expr/calls/mod.rs`, `check/expr/postfix.rs`, `check/calls.rs`, `check/stmt/calls.rs`: arity, positional arguments, generic argument inference and procedure/value distinction | Separate callable-value invocation from methods/fluent lookup; make all value results require consumption or `discard`. Share argument checking across direct, indirect and postfix calls. |
| Callable types | `crates/fpas-sema/src/types/mod.rs`: function/procedure signatures compare parameter order, types and current mutability, ignoring parameter names | Expected concrete generic routine values are not instantiated by the current expected-type path. Current mutable parameters are local mutation, not caller references. Purity and reference modes belong to coordinated stage 5. |
| Captures | `crates/fpas-sema/src/check/closures/`, `crates/fpas-compiler/src/lowering/closures/`: declaration identity, transitive anonymous/named captures, value/cell distinction and task-bound metadata | Visit arbitrary targets and their arguments, including index expressions. Extend containment/transfer checks without treating task-bound tracking as a purity or lexical task-ownership checker. |
| IR | `crates/fpas-ir/src/instruction.rs`, `function.rs`, `validate/operands/calls.rs`, `validate/operands/closures.rs`: `CallValue`, `MakeClosure`, cells, signature/capture validation and block parameters | Reuse operations for new targets. Carry new signature capabilities through IR only when reference/purity behavior lands; decision expressions need typed merge values. |
| Compiler and bytecode | `crates/fpas-compiler/src/lowering/calls.rs`, `lowering/context/saved_values.rs`, `bytecode/selection.rs`; `crates/fpas-bytecode/src/validate/instruction/abc.rs`, `validate/calls.rs`: ordered operands, indirect invocation, argument windows and closure validation | General targets must lower once before arguments; preserve the saved target through argument-generated control flow. No new invocation opcode is required by the existing representation. |
| VM | `crates/fpas-vm/src/vm/calls.rs`, `calls/closures.rs`; `crates/fpas-bytecode/src/value/function.rs`: numeric function IDs, shared environments, mutable cells, owner checks and argument/capture frames | Reuse ordinary invocation. Bound-receiver behavior remains a separate removal owner; exercise new source targets through compiler and CLI, not only hand-built bytecode. |
| Generic routines and data | `crates/fpas-parser/src/parser/decl/type_expr.rs`; `crates/fpas-sema/src/check/calls.rs`, `check/name_resolution/types/generics.rs`, `check/decl/types/collection.rs`; `crates/fpas-compiler/src/lowering/types/layouts.rs`: routine constraints/inference, whole-unit type collection and reserved layouts | Generic type declarations/applications, instantiated nominal identities, constructor inference, finite-value recursion checks and canonical `of (...)` syntax are missing. Do not replace existing whole-unit collection. |
| Records and updates | `crates/fpas-sema/src/check/decl/vars.rs`, `check/expr/record_fields.rs`, `check/record_visibility.rs`; `crates/fpas-compiler/src/lowering/aggregates/records.rs`: expected record context, defaults, required/duplicate field checks, private construction and ordered updates | Replace contextual record literals with resolved named construction. Construction currently evaluates in declaration/default order; target construction needs supplied-field order followed by omitted defaults. Default purity is coupled to stage 5. |
| Equality and storage | `crates/fpas-sema/src/check/expr/equality.rs`; `crates/fpas-bytecode/src/value/equal.rs`, `value/array.rs`, `value/aggregate.rs`: component checks, iterative deep equality and copy-on-write aggregate storage | Source array/dictionary equality, mapping-based dictionary comparison, recursive unsupported-component rejection and `Equatable` are missing. Storage sharing alone does not prove snapshot/reference behavior. |
| Decisions | `crates/fpas-sema/src/check/stmt/control_flow/if_case/`; `crates/fpas-compiler/src/lowering/case/`: arm scopes, grouped-binding checks, guards, top-level coverage and one-time scrutinee lowering | Add explicit/nested patterns, payload wildcards/constants, reachability and nested coverage. Closed enums must reject catch-all arms. Expression branches need expected-type checking and one merged value. |
| Formatter and tooling | `crates/fpas-fmt/src/emit/expr/`, `comments/traversal/expressions.rs`; `crates/fpas-project/src/unit_graph/source_map/expressions.rs`: ordered AST traversal, comment spans and source IDs | Extend invocation/constructor/pattern/decision emission and traversal together; retain round-trip/idempotence checks. Editor signature help currently resolves a callable token, not an arbitrary callable expression. |

## Callable targets and capture representation

`parse_primary` applies postfix suffixes to completed primary expressions, but
`apply_postfix_suffixes` accepts only `.Field`, `[Index]`, and `.Method(args)`.
`Expr::Call` retains a designator rather than an expression target. Consequently
`MakeAdder(3)(5)`, `(Callback)(5)` and calling an indexed callable are not supported
uniformly through the syntax/checking/lowering pipeline. Returned-record field
access and returned-collection indexing are existing foundations, not proof of
general invocation.

Callable record fields already have useful support. In
`check/expr/postfix.rs`, a field/property callable enters
`check_member_value_call`; `lowering/members.rs` emits `Operation::CallValue`
without inserting a receiver for this path. That behavior can be extracted and
reused. Actual methods, computed properties, event access and fluent calls still
have separate semantic maps and receiver rules. Preserve their resolved ownership
until the later member migration; do not classify every `.Name(...)` as a method
or add a receiver to an ordinary field value.

`check/stmt/calls.rs` currently accepts named function calls as statements, while
`check/expr/postfix.rs` requires an expression statement to end with a method
call. Both must reach the same result-use rule. `discard` has no current AST/token
implementation. It must evaluate an ordinary value once, reject procedure calls, and
reject a task handle transitively through arrays, dictionaries, records, variants,
Option and Result. Migrating unused results requires inspecting intrinsic calls
and Rust-embedded consumers as well as `.fpas` files.

Capture reuse crosses all layers:

- `check/closures/capture.rs` records the source declaration span, type, mutable
  state and transitive task-bound state. It excludes root-scope bindings and
  respects nested routine/closure metadata and local shadowing.
- `check/expr/closure.rs` checks anonymous routines through the same body/capture
  machinery used by named nested routines. `closures/capability.rs` marks mutable
  or transitively task-bound captures.
- `lowering/closures/mod.rs` and `discover.rs` assign numeric targets and identify
  owner locals that need cells. `fpas-ir::CaptureKind` distinguishes `Value`,
  `Cell` and `EnclosingCell`; validators check capture types and provenance.
- `value/function.rs` shares immutable function metadata/environments through
  `Arc`; captured mutable state uses shared cells. Copying a stateful callable
  therefore retains its environment identity. VM `calls/closures.rs` constructs
  cells/closures; `calls.rs` checks invocation arity, capture count and task owner.
- `check/expr/task_bound.rs` already propagates capability through several
  aggregates and accesses. Its call-result and recursive named-type handling,
  together with runtime containment checks, need dedicated interaction coverage.
  Do not assume arbitrary returned callables inherit correct transfer metadata.

General invocation should use the existing `CallValue`/`MakeClosure` operations
and `SavedValue` mechanism. Evaluate the target once, save it, evaluate each
argument left to right, then restore the target and earlier arguments after any
`try` or decision-generated control flow. Fresh loop captures, copying nested
value captures and anonymous/named capture parity require source-level tests.
Computed const captures, pure captures and forbidden caller-reference captures
must land with the binding/reference/purity boundary, rather than being inferred
from the existing mutable flag.

## Generic routines, data and recursive layouts

The current parser accepts angle-bracket routine parameters and built-in generic
type forms without the target's parenthesized lists. `TypeDef` has no type
parameters; `TypeExpr` has no user-defined type application. Generic type
declarations are rejected, and named `of` applications are rejected in
`parser/decl/type_expr.rs`. Canonical `of (...)` must migrate routine declarations,
all built-in types, formatter, interfaces, source consumers and editor examples
together with the data facilities.

`check/calls.rs` infers type arguments from actual arguments and nested callable
signatures, checks consistent reuse and validates the small constraint set.
Substitution handles collections, Option/Result/tasks and callable signatures,
but has no instantiated generic record/enum model. `TypeConstraint` currently
contains `Comparable`, `Numeric` and `Printable`; `Comparable` accepts scalar
types, and `Printable` only directly excludes function/procedure types. Audit
their component rules when adding `Equatable`; their names alone are not evidence
that they satisfy the target's resource/equality restrictions.

Expected typing in `check/decl/vars.rs` specializes record literals and arrays of
records. It does not instantiate a generic routine designator from a concrete
expected signature. Strict assignment compatibility rejects unresolved generic
parameters against concrete types. The test
`generic_function_value_does_not_coerce_to_concrete_signature` assigns
`Identity<T>(T): T` to `function(integer): string`: this remains an invalid
assignment under the target contract. Retain that negative case and add compatible
`function(integer): integer` instantiation plus missing-context/conflict cases;
do not turn the existing incompatible example into a positive regression.

The compiler currently lowers generic parameters to `IrType::Dynamic`
(`lowering/types.rs`) and lowers routine signatures using declared type parameters
(`lowering/routines.rs`). This is a reusable execution mechanism, not a generic
data instantiation model or evidence of monomorphization. New concrete data
instances must have distinct, consistent semantic/layout identities through
compiled-unit interfaces, linking and debugger types.

Unlike the earlier stage-1 snapshot, `check/decl/types/collection.rs` now collects
all unit/program type headers before resolving bodies and rejects transparent
alias cycles. `lowering/types/layouts.rs` reserves record/enum layouts before
populating fields. Reuse both mechanisms. Their ability to represent a recursive
layout is not a check that the type has a finite value: direct/mutual mandatory
record cycles and recursive enums without a base case need explicit rejection;
collection recursion and enums with a finite base case need positive cases.

`crates/fpas-unit/src/interface/types.rs` already serializes callable generic
parameters, named recursive references, record visibility and enum payloads.
`crates/fpas-sema/src/interface/conversion/to_interface.rs` and
`from_interface.rs` convert those descriptors. They lack generic data templates
and purity/caller-reference distinctions. Preserve authoritative source/manifests
and automatic sidecar rebuilding; do not introduce a separate generic artifact
cache or hand-edit `.fpascu` files.

## Construction, value copying and equality

Retain duplicate-field validation in `check/expr/record_fields.rs`, expected
field context and required/defaulted-field validation in `check/decl/vars.rs`,
and owner/private-field checks in `check/record_visibility.rs`. Private
construction already rejects records with inaccessible fields even if those
fields have defaults. New named constructors must reuse that gate and distinguish
resolved type targets from ordinary callable and variant-constructor targets.

`lower_record_literal` and `lower_record_literal_as` collect supplied expressions
in a name map, then iterate declared fields/defaults. This currently evaluates
supplied expressions in layout order and interleaves omitted defaults. The target
must stage supplied expressions in written order first, then omitted defaults in
declaration order, and finally assemble values in layout order. Type/visibility
checks are reusable; the current evaluation order is not. Imported defaults also
need attention: interface fields store canonical scalar constant defaults, not
general pure default expressions.

`lower_record_update` already evaluates/saves the base before processing
replacements in written order, preserving earlier values across control flow.
It contextually lowers replacement values to their field type and emits
`UpdateRecord`. Preserve this path and extend its coverage to nested copies,
resources and stateful callables. VM record updates and `SharedRecord`,
`SharedEnum`, `SharedDict` and shared-array storage provide copy-on-write building
blocks; test every nested field/index/cell/global write rather than assuming
aggregate cloning establishes snapshot semantics.

Equality has three distinct migration requirements:

1. `check/expr/equality.rs` recursively checks record/payload-enum components but
   directly accepts top-level Option/Result before recursive component checking.
   Arrays/dictionaries are rejected in source equality. Use one recursive source
   eligibility rule, including nested resources/tasks/callables and generic
   constraints; do not silently admit runtime handle equality.
2. `value/equal.rs` already walks deep aggregate pairs iteratively. Dictionary
   equality zips stored pairs, so insertion order affects the result. Reuse the
   iterative traversal but compare supported dictionaries as mappings. Dict type
   resolution currently resolves key/value types without a key eligibility gate;
   define and diagnose supported key representations consistently with equality.
3. Runtime internal equality also compares functions, cells and handles, and
   allows equal real bit patterns. Scalar VM real equality in
   `vm/value_ops/comparison.rs` uses numeric equality. Source structural equality
   must preserve the contract's numeric behavior, including nested NaN and signed
   zero, without exporting internal identity/bit-pattern shortcuts.

Checked numeric behavior and value snapshots remain coupled to bindings and
stage-5 caller references. For example, `vm/value_ops/integer.rs` still uses
wrapping add/subtract/multiply. General callable syntax does not close this later
numeric/mutation acceptance item.

## Patterns and decision expressions

`CaseLabel` currently separates expression/range labels from special Option/Result
destructuring. Payload-enum patterns are encoded as calls/designators with plain
identifier payload bindings. `if_case/labels.rs` explicitly rejects nested
patterns, payload `_` and payload literals. A guarded scalar identifier can also
introduce a binding. These are current forms to migrate, not target pattern rules.

Reuse arm scopes and `if_case/bindings.rs`, which now checks that labels in a
shared arm introduce matching names/types. Preserve guard-after-match execution
and one-time scrutinee lowering in `lowering/case/variant.rs` and `scalar.rs`.
`if_case/exhaustiveness.rs` excludes guarded arms from top-level coverage, but
does not analyze nested payload coverage. Current `else` handling can bypass
closed-enum coverage; the target must reject enum catch-alls explicitly and
diagnose duplicate/unreachable arms.

There is no `Expr::If` or `Expr::Case`. Add focused expression checking/lowering
and a shared pattern representation rather than parsing expressions as statements
or inferring a value from their last statement. Branch typing must honor expected
types or find one common compatible type independent of branch order; lower only
the selected branch to a typed merge value. Open scalar statement cases retain
their no-match behavior; expression cases require a fallback unless finite
coverage is proven. Selected callable targets depend on these later expression
facilities and must be tested when those facilities land.

## Module shape before extension

The existing thematic directories should remain the ownership boundaries. These
line counts were refreshed from the inspected checkout; they are implementation
planning evidence, not a reason to refactor unrelated files in this audit.

| Existing path | Lines | Required approach when touched |
|---|---:|---|
| `crates/fpas-sema/src/types/mod.rs` | 429 | Move callable descriptors to `types/callables.rs` before adding reference/purity metadata; keep re-exports stable. |
| `crates/fpas-sema/src/check/calls.rs` | 418 | Split shared argument checking and generic inference under `check/calls/`; reuse both across target kinds. |
| `crates/fpas-sema/src/check/closures/capture.rs` | 448 | Separate binding collection from syntax traversal before extending target/pattern visitors. |
| `crates/fpas-sema/src/check/expr/mod.rs` | 442 | Add focused invocation/decision modules; retain a small dispatcher. |
| `crates/fpas-sema/src/check/expr/calls/methods.rs` | 558 | Split method-target resolution; route member-value call sites through shared value invocation extracted from `calls/fluent.rs`. |
| `crates/fpas-compiler/src/lowering/members.rs` | 466 | Move ordinary postfix/member-value invocation under `lowering/calls/`; leave property/event mechanisms with their removal owner. |
| `crates/fpas-compiler/src/lowering/expr.rs` | 457 | Delegate new target/decision behavior to focused modules. |
| `crates/fpas-compiler/src/lowering/types.rs` | 416 | Extend concrete instantiation/layout ownership under `lowering/types/`, not the dispatcher. |
| `crates/fpas-ir/src/instruction.rs` | 493 | Reuse existing call/closure operations; split by operation concern if later reference work grows this file. |

Parser postfix (91 lines), sema postfix (339), compiler calls (279), closure
discovery (342) and formatter expression dispatch (368) already have focused
neighbours. Extend those seams instead of introducing generic helpers or a second
call/capture runtime. No Rust modules are moved or split by this documentation edit.

## Existing coverage inspected

These are current executable tests to preserve or deliberately migrate, not a
claim that they passed during this audit or cover the target completely.

| Area | Concrete existing coverage | Target additions |
|---|---|---|
| Parser targets | `crates/fpas-parser/src/tests/expr/postfix.rs`, `expr/closures.rs`: returned fields/indexes, chained methods, parenthesized bases, recovery and typed anonymous routines | Returned/indexed/parenthesized/anonymous callable invocation, final procedure calls, malformed call suffixes and named-argument rejection. |
| Sema targets | `crates/fpas-sema/src/tests/expr/postfix.rs`: intermediate aliases, indexes, final procedure position and non-call suffix rejection | Uniform target typing, no receiver insertion, callable field visibility, arity/type errors and unused-result/discard rules. |
| Capture checking | `crates/fpas-sema/src/tests/expr/closures.rs`: nested parameter/block shadowing, scalar guard bindings and transitive task-bound closures | Captures used only in arbitrary targets/indices/arguments, loop freshness, copied environments, returned/container capabilities and named/anonymous parity. Migrate scalar guard-binding cases to explicit patterns later. |
| Source closure execution | `crates/fpas-compiler/src/tests/closures.rs`: `immutable_anonymous_capture_executes`, `mutable_anonymous_capture_shares_cell_with_repeated_calls`, `named_nested_routine_escapes_with_numeric_capture_target`; `closures/repeat.rs`; `tests/stdlib/closures/capture_values_test.fpas` | Exact target/argument/body trace, exactly-once evaluation, early propagation and copied stateful closure identity. Existing ignored `Next()` result needs explicit discard migration. |
| Named callable execution | `crates/fpas-compiler/src/tests/functions.rs`: `first_class_named_function_uses_call_value`, `first_class_named_procedure_uses_call_value` | Composition through returned and indexed values using the same runtime operations. |
| IR and bytecode | `crates/fpas-ir/tests/validation/cell_cases.rs`; `crates/fpas-bytecode/tests/bytecode/verifier.rs`: arity/destination/window checks, capture count and provenance | Malformed indirect callable signatures/captures and generated target calls; retain verifier validation through the real compiler. |
| VM identity and owners | `crates/fpas-vm/src/vm/tests/calls.rs`: numeric callable invocation and mutable cell capture; `crates/fpas-vm/src/vm/tests/task_owned_functions.rs`: mutable capture owner and foreign-task rejection | Same-task copies sharing state; transitive containment cannot weaken transfer restrictions. Bound-receiver tests remain until member removal. |
| Generic routines | `crates/fpas-sema/src/tests/generics/bodies.rs`, `generics/methods.rs`; `crates/fpas-cli/src/main_tests/projects/generic_aliases.rs` | Compatible expected callable instantiation, ambiguity/conflicts, generic record/enum instances, nesting/recursion and constraints. Preserve strict generic-body negatives. |
| Record context/updates | `crates/fpas-sema/src/tests/expr/record_context.rs`, `record_updates.rs`, `crates/fpas-sema/src/tests/decl/types.rs`; `crates/fpas-compiler/src/tests/aggregates/record_updates.rs`; `crates/fpas-cli/src/main_tests/projects/record_updates.rs`; `tests/runner/record_update_context_test.fpas` | Named construction, private/defaulted construction, supplied/default effect order, generic inference and nested copy isolation. Preserve unknown/duplicate/type-error checks and empty context-typed replacements. |
| Enum construction/recursion | `crates/fpas-cli/src/main_tests/projects/enum_variants.rs`; `tests/runner/recursive_enum_expression_test.fpas` | Expected generic variants, payloadless ambiguity, finite recursive data, nested patterns and explicit closed coverage. |
| Equality/storage | `crates/fpas-sema/src/tests/expr/equality.rs`; `crates/fpas-compiler/src/tests/aggregates/structural_equality.rs`; `crates/fpas-bytecode/tests/value.rs`; unit tests in `value/array.rs` and `value/aggregate.rs` | Source arrays/dictionaries, reverse insertion order, recursive unsupported components, NaN/zero and nested source writes. Internal value-equality tests are not the source equality specification. |
| Formatting | `crates/fpas-fmt/tests/round_trip.rs`, `comment_regressions.rs`, `syntax_and_names.rs`, `golden/postfix_chaining.expected.fpas` | Invocation suffixes, multiline callable arguments, comments, target-parenthesization, new types/patterns/decisions and idempotence across the full valid-source trees. |

## Next delivery: ordinary callable expression targets

The smallest next implementation slice is arbitrary invocation of existing
concrete function/procedure values with the existing capture representation.
Use explicitly typed current bindings until the coordinated binding migration.
This slice includes the result-use/discard rule and its consumers so a final
postfix call cannot silently drop a result. It does not introduce pure callables,
caller references, new binding semantics or generic data yet. Selected targets
join the same invocation path when decision expressions are delivered.

Before implementation, refresh these paths and list every additional migration
file discovered by resolved-source analysis. This is the concrete starting layout;
it is not authorization for unrelated cleanup.

| Action | Paths | Purpose |
|---|---|---|
| Modify | `crates/fpas-parser/src/ast/expr.rs`, `ast/stmt.rs`, `parser/expr/postfix.rs`, `parser/expr/primary.rs`, `parser/stmt/basic.rs`, `parser/stmt/mod.rs` | Represent expression-target invocation and discard; preserve assignment roots, precedence, recovery and procedure statement position. |
| Modify | `crates/fpas-lexer/src/token/kind.rs`, `token/keywords.rs`; `crates/fpas-parser/src/parser/display.rs` | Introduce the canonical discard keyword and diagnostic spelling. |
| Create / move | `crates/fpas-sema/src/types/callables.rs`; modify `types/mod.rs` | Extract callable/parameter descriptors with documented re-exports. |
| Create / split | `crates/fpas-sema/src/check/calls/arguments.rs`, `check/calls/inference.rs`; modify `check/calls.rs` | Separate shared positional argument checks from inference/substitution. |
| Create / extract | `crates/fpas-sema/src/check/expr/calls/values.rs`; modify `check/expr/calls/mod.rs`, `calls/fluent.rs`, `calls/methods.rs`, `check/expr/postfix.rs`, `check/stmt/calls.rs`, `check/stmt/mod.rs` | Extract `check_member_value_call` from fluent resolution into one concrete callable invocation/result-use path; recursive task-containing discard rejection. |
| Create / split | `crates/fpas-sema/src/check/expr/calls/methods/resolution.rs`; modify `check/expr/calls/methods.rs` | Move static/instance target lookup out of the oversized method dispatcher before updating its value-call sites. |
| Create / split | `crates/fpas-sema/src/check/closures/capture/traversal.rs`; modify `check/closures/capture.rs`, `check/expr/task_bound.rs` | Visit target/argument/index expressions while retaining declaration identity and capability propagation. |
| Create / extract | `crates/fpas-compiler/src/lowering/calls/values.rs`; modify `lowering/calls.rs`, `members.rs`, `expr.rs`, `stmt.rs`, `closures/discover.rs` | Reuse `CallValue` and saved operands; move ordinary member/postfix invocation out of mixed member lowering; discard produces no extra evaluation. |
| Modify | `crates/fpas-sema/src/check/decl/consts.rs`; `crates/fpas-project/src/unit_graph/source_map/expressions.rs`, `statements.rs` | Keep static classification and source-ID traversal exhaustive for new AST forms. |
| Modify | `crates/fpas-fmt/src/emit/expr/mod.rs`, `expr/postfix.rs`, `emit/stmt/mod.rs`, `emit/stmt/line.rs`, `comments/traversal/expressions.rs`, `comments/traversal.rs` | Emit canonical invocation/discard and preserve comments/parenthesization. |
| Modify | `crates/fpas-language-service/src/intellisense/signature_help.rs`, `navigation/selection.rs` | Resolve concrete expression targets in call signatures and keep new statement ranges. |
| Create / extend tests | `crates/fpas-compiler/src/tests/calls/targets.rs`, `crates/fpas-compiler/src/tests/calls.rs`; `crates/fpas-cli/src/main_tests/projects/callable_targets.rs`; `tests/stdlib/closures/callable_targets_test.fpas` | Focused execution and imported-unit/runner coverage; wire new test modules into their existing parents. Extend parser/sema/formatter tests listed above. |
| Modify current docs | `docs/specs/grammar.ebnf`, `docs/pascal/language/functions/first-class.md`, `function-types.md`, `closures.md`, `postfix-chaining.md`; `docs/pascal/tools/fmt-style.md` | Describe implemented invocation/result consumption only; migrate handbook examples and editor/source consumers found by the slice audit. |

Retain IR/bytecode operations and VM invocation code where the new source tests
prove them sufficient. Check interfaces/converters, debug callable reconstruction,
spawn/callback adapters, editor grammar and templates for any signature/AST
assumptions; update only affected consumers. `lowering/concurrency.rs` and
`closures/intrinsic_tasks.rs` currently recognize specific final call forms, so
new call forms must either use their existing spawn path or receive an explicit
diagnostic until the stage-5 task delivery. They must not bypass task-bound checks.

The next slice passes only when returned, parenthesized, indexed and field
callables execute; targets/arguments execute once in order even through `try`;
procedure calls fail in value position; function statements require consumption or
discard; task-containing discard fails; and named/anonymous captures retain
identity, shadowing, copy and loop behavior. Cover failures through structured
diagnostics and a real imported-unit CLI run. Preserve negative sources as
Rust-embedded cases outside the formatter's valid-source corpus.

Then deliver generic records/enums, canonical type lists, constructor/expected
callable inference and finite recursion, followed by nested patterns and shared
if/case expression typing. Reuse the owners above. Keep defaults, computed
bindings, value/numeric behavior and standard caller-mutation migration behind
the coordinated stage-5 reference/purity gate. Remove member mechanisms only
after callable fields, Option patterns and ordinary invocation work.

## Audit verification and completion

Source owners, directory shapes, module sizes and the listed existing tests were
inspected. Relative links, existing repository paths and the documentation diff
are checked for this delivery. No build or runtime test result is claimed; those
checks do not verify an unimplemented target. The next implementation delivery
must run [shared verification](../verification.md), including workspace build/tests,
FPAS formatting/suite, relevant CLI/editor checks and the new interaction cases.

The audit itself closes only stage 4's audit checkbox. The
[callable delivery](functional-core-callables.md) closes the callable checkbox
with implementation and verification evidence. Generic data, decisions, bindings,
references/purity, member removal and their migrations remain implementation work.

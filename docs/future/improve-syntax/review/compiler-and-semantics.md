# Compiler and semantic findings

See [scope, priorities and verification](README.md). These are confirmed audit findings; the finding index tracks completed repairs, and resolved entries describe their implemented behavior and coverage. Package attribution is not a historical introduction bisect. Commands run from the repository root. Complete minimal sources are included where practical; save them under the named ignored scratch path to repeat the checks. The copied executable used for audit is identical to the freshly built `target/debug/fpas.exe`; either can be used with `--std-lib lib`.

<a id="c01"></a>

## C01 — P1: enum comparison patterns use resolved constant identity

Packages: AP20.2 and AP20.3.

Status: resolved. See [repair verification](README.md#repair-verification) for
checks and independent workspace validation blockers.

Implemented behavior: semantic analysis records which full designators resolve
to enum-member symbols. Both ordinary expression lowering and fieldless variant
tests use that identity. Constants, parameters, and record fields with a variant's
name retain their resolved values. Simple enums compare their backing integers;
actual fieldless variants of data enums retain variant tests.

Owners:

- [Semantic designator resolution](../../../../crates/fpas-sema/src/check/expr/designator.rs)
  and [analysis metadata](../../../../crates/fpas-sema/src/check/context.rs).
- [Expression lowering](../../../../crates/fpas-compiler/src/lowering/expr/designators.rs),
  [pattern tests](../../../../crates/fpas-compiler/src/lowering/case/patterns.rs),
  and [lowering context](../../../../crates/fpas-compiler/src/lowering/context/mod.rs).

Contract: [Pattern syntax](../../../../docs/pascal/language/pattern-matching/syntax.md) says a plain identifier compares with the constant or member it names. [Exhaustiveness](../../../../docs/pascal/language/pattern-matching/exhaustiveness.md) gives named simple-enum constants their evaluated coverage. [Is-test patterns](../../../../docs/pascal/language/pattern-matching/is-test.md) reuses the case pattern rules.

Runtime control: save as `.temp-data/pattern_shadowed_enum_constant.fpas`:

```pascal
program PatternShadow;
uses Std.Console;
type Shade = enum
  Red;
  Blue;
end enum;
begin
  const Red: Shade := Shade.Blue;
  const Value: Shade := Shade.Blue;
  if Value is Red then
    WriteLn('matched');
  else
    WriteLn('missed');
  end if;
  const Candidate: Option of Shade := Some(Shade.Blue);
  case Candidate of
    when Some(Red):
      WriteLn('blue');
    when Some(Shade.Red):
      WriteLn('red');
    when None:
      WriteLn('none');
  end case;
end.
```

Commands:

```text
fpas check --std-lib lib .temp-data/pattern_shadowed_enum_constant.fpas
fpas run --std-lib lib .temp-data/pattern_shadowed_enum_constant.fpas
```

The control checks successfully and prints `matched`, then `blue`, with exit 0.
The local `Red` constant denotes `Shade.Blue` in both coverage and emitted
comparisons; qualified `Shade.Red` still denotes the actual red member.

Regression coverage:
[enum comparisons](../../../../crates/fpas-compiler/src/tests/control_flow/enum_comparisons.rs)
contains five execution tests covering plain and parenthesized shadowing constants
in `is` and exhaustive nested `case`, case-insensitive names, nonsequential
backing values, constant copies, parameters, record fields, actual fieldless data
variants, and qualified/import-aliased constants after interface serialization
and linking. Each comparison has matching and nonmatching controls.

<a id="c02"></a>

## C02 — P2: sibling nested calls preserve transitive captures

Package: AP17.1, with closure/capture infrastructure.

Status: resolved. See [repair verification](README.md#c02) for checks and
independent workspace validation blockers.

Implemented behavior: references to named nested routines import their analyzed
captures transitively. Capture identity, type, reference storage, shared mutable
cells, task-bound status, and discard proofs retain the original declaration.
Repeated references are deduplicated by name and source declaration; same-named
captures from different enclosing scopes remain distinct, with the nearest binding visible to
ordinary name lookup. Parameters and locals in a wrapper retain their own bindings.
Anonymous wrappers keep their lexical routine path when resolving named calls.

Named wrappers that directly or indirectly use an enclosing `var` parameter
can be called by name during the enclosing call. Returning, assigning, passing,
spawning, or capturing such wrappers in an anonymous closure reports FP3030.

Owners:

- [Lexical capture lookup](../../../../crates/fpas-sema/src/scope/captures.rs),
  [routine registration](../../../../crates/fpas-sema/src/check/decl/routines.rs),
  and [capture analysis](../../../../crates/fpas-sema/src/check/closures/capture/mod.rs).
- [Transitive capture traversal](../../../../crates/fpas-sema/src/check/closures/capture/traversal.rs)
  and [reference escape checks](../../../../crates/fpas-sema/src/check/references/escapes.rs).
- [Capture storage forwarding](../../../../crates/fpas-compiler/src/lowering/context/captures.rs),
  [named-routine capture typing](../../../../crates/fpas-compiler/src/lowering/routines.rs),
  and [anonymous-closure discovery](../../../../crates/fpas-compiler/src/lowering/closures/discover.rs).

Contract: [Reference-parameter lifetimes](../../../../docs/pascal/language/functions/var-parameters.md#lifetime)
and [named nested closures](../../../../docs/pascal/language/functions/closures.md#named-nested-routines)
apply capture and escape rules throughout the call chain.

Runtime control: save as `.temp-data/reference_indirect_direct.fpas`:

```pascal
program ReferenceIndirect;
uses Std.Console;
procedure Outer(var Value: integer);
  procedure First();
  begin
    Value := Value + 1;
  end procedure;
  procedure Second();
  begin
    First();
  end procedure;
begin
  Second();
end procedure;
begin
  var Total: integer := 0;
  Outer(var Total);
  WriteLn(Total);
end.
```

Commands:

```text
fpas check --std-lib lib .temp-data/reference_indirect_direct.fpas
fpas run --std-lib lib .temp-data/reference_indirect_direct.fpas
```

The control checks successfully and prints `1`, with exit 0. Changing the enclosing
routine to `function Make(var Value: integer): procedure()` and returning `Second`
is rejected with FP3030.

Regression coverage:

- [Compiler execution tests](../../../../crates/fpas-compiler/src/tests/functions/var_parameters/transitive_captures.rs)
  cover multi-hop sibling calls, recursion, parameter/local/block shadowing,
  same-named captures from different ancestors, immutable captures through returned
  routines and anonymous wrappers, and shared mutable cells after the parent returns.
- [Semantic tests](../../../../crates/fpas-sema/src/tests/expr/var_parameters/transitive_captures.rs)
  cover deduplication and declaration identity, FP3030 for returned/assigned/passed/
  spawned/anonymous wrappers, local callable shadowing, and retained task/discard
  restrictions.

<a id="c03"></a>

## C03 — P2: pattern bindings preserve callable capability metadata

Packages: AP20 with AP16/AP04.

Status: resolved. See [repair verification](README.md#c03) for checks and
independent workspace validation blockers.

Implemented behavior: `is` bindings and all `case` binding paths use one
binding definition that retains the matched expression's task-bound state and
known discard proof. The same definition serves `if`, `elsif`, `while`, guards,
scalar case bindings, and nested payload patterns. Scalar types, including
`Numeric`/`Comparable` generic parameters, keep their own guarantees; binding a
scalar does not make it task-bound because another payload contains a callable.
Unknown or mutable callable contents do not gain a discard proof from extraction.
Task-freedom and task-bound state remain independent: a mutable-capturing callable
with no task handles can be discarded but cannot be spawned or sent to another task.

Capture identity uses the binding's name and source declaration. Lowered `is`
bindings use the same pattern declaration as semantic analysis. Multiple bindings
from one pattern remain distinct in closure captures and debugger provenance.
Recursive enum inspection terminates with the same cycle protection as records.

Owners:

- [Pattern binding capabilities](../../../../crates/fpas-sema/src/check/stmt/control_flow/pattern_capabilities.rs),
  [condition checking](../../../../crates/fpas-sema/src/check/stmt/control_flow/conditions.rs),
  and [case checking](../../../../crates/fpas-sema/src/check/stmt/control_flow/if_case/mod.rs).
- [Callable-containing type checks](../../../../crates/fpas-sema/src/check/expr/task_bound.rs)
  and the existing [discard proof checks](../../../../crates/fpas-sema/src/check/discard/mod.rs).
- [Pattern declaration lowering](../../../../crates/fpas-compiler/src/lowering/control_flow/conditions.rs),
  [capture collection](../../../../crates/fpas-sema/src/check/closures/capture/mod.rs),
  and [debugger capture provenance](../../../../crates/fpas-compiler/src/lowering/debug/capture_sources.rs).

Contract: [Pattern bindings](../../../../docs/pascal/language/pattern-matching/syntax.md#pattern-bindings),
[closure task restrictions](../../../../docs/pascal/language/functions/closures.md#concurrency),
and [discard capture proofs](../../../../docs/pascal/language/functions/discard.md#channels-and-callable-captures)
apply to extracted callable values.

Rejected control: save as `.temp-data/pattern_task_escape.fpas`:

```pascal
program PatternTask;
uses Std.Console, Std.Tasks;
procedure Main();
begin
  var Value: integer := 0;
  const Change: procedure() := procedure() begin
    Value := Value + 1;
  end procedure;
  const Wrapped: Option of procedure() := Some(Change);
  if Wrapped is Some(const F) then
    const Worker: task := go F();
    Wait(Worker);
  end if;
  WriteLn(Value);
end procedure;
begin
  Main();
end.
```

Commands:

```text
fpas check --std-lib lib .temp-data/pattern_task_escape.fpas
fpas run --std-lib lib .temp-data/pattern_task_escape.fpas
```

The control is rejected during checking with FP3016. Equivalent `case` and
`while` extraction controls are also rejected during checking. The captured
mutable local remains task-bound after extraction.

Accepted control: save as `.temp-data/pattern_task_free.fpas`:

```pascal
program PatternTaskFree;
uses Std.Console, Std.Tasks;
function Work(): integer;
begin
  return 42;
end function;
procedure Main();
begin
  const Wrapped: Option of function(): integer := Some(Work);
  if Wrapped is Some(const F) then
    discard F;
    const Job: task := go F();
    WriteLn(Wait(Job));
  end if;
end procedure;
begin
  Main();
end.
```

The same check/run commands with this filename succeed; runtime prints `42`.
The extracted capture-free callable keeps both its discard proof and permission
to run in another task.

Regression coverage:

- [Semantic tests](../../../../crates/fpas-sema/src/tests/stmt/pattern_capabilities.rs)
  cover task-bound rejection through `if`, `elsif`, `while`, nested Result/Option
  payloads, guards, channel sends, and enclosing closures; accepted immutable and
  task-free captures; conservative rejection of unknown, mutable, or task-handle
  captures; scalar and generic guarantees; records; and binding-name shadowing.
- [Compiler execution tests](../../../../crates/fpas-compiler/src/tests/control_flow/pattern_capabilities.rs)
  cover accepted task spawning and discard, closures over extracted callables,
  discard without invocation, scalar fields of task-bound records, and distinct
  captures for multiple bindings from one enum pattern.
- Existing recursive-type and exhaustiveness tests cover the cycle-safe type
  inspection used by the shared binding definition.

<a id="c04"></a>

## C04 — P2: record-derived compile-time constants contribute their values to pattern coverage

Packages: AP20.2 with AP16/AP10.

Status: resolved. See [repair verification](README.md#c04) for checks and
independent workspace validation blockers.

Implemented behavior: static record constants retain known scalar and nested
record fields. The scalar evaluator follows the longest visible binding prefix
and then projects its fields. Copies, updates, omitted defaults and transitive
constants preserve these values; defaults use their declaration scope.
Boolean and simple-enum values participate in finite pattern coverage and
duplicate comparisons. Mutable and computed records remain invalid constant
labels.

Compiled-unit interfaces preserve these known fields, including qualified
names, aliases and facade exports. Owned enum identities are qualified and
canonicalized. Aggregate constants retain immutable runtime-global storage.
The `.fpascu` envelope version is 11; incompatible sidecars are rebuilt through
the normal unit-build validation.

Owners:

- [Record evaluation and field projection](../../../../crates/fpas-sema/src/check/decl/consts/records.rs)
  and [scalar evaluation](../../../../crates/fpas-sema/src/check/decl/consts/scalar.rs).
- [Default values](../../../../crates/fpas-sema/src/check/decl/types/records/defaults.rs)
  and [constant binding metadata](../../../../crates/fpas-sema/src/scope.rs).
- [Interface export](../../../../crates/fpas-sema/src/interface/export/record_constants.rs),
  [import](../../../../crates/fpas-sema/src/interface/conversion/from_interface.rs),
  and [portable record constants](../../../../crates/fpas-unit/src/interface/record_constants.rs).

Contract: [Constants](../../../../docs/pascal/language/basics/constants.md)
retains static record values and their known scalar fields.
[Exhaustiveness](../../../../docs/pascal/language/pattern-matching/exhaustiveness.md)
uses evaluated Boolean and simple-enum values for coverage and reports
comparisons already covered by earlier unguarded labels.

Runtime control: save as `.temp-data/pattern_record_constant.fpas`:

```pascal
program PatternConstant;
type Flags = record
  Enabled: boolean;
end record;
const Settings: Flags := Flags(Enabled := true);
const Enabled: boolean := Settings.Enabled;
begin
  const Candidate: Option of boolean := Some(true);
  case Candidate of
    when Some(Enabled):
      null;
    when Some(false):
      null;
    when None:
      null;
  end case;
end.
```

Commands:

```text
fpas check --std-lib lib .temp-data/pattern_record_constant.fpas
fpas run --std-lib lib .temp-data/pattern_record_constant.fpas
```

Both commands exit 0. `Enabled` denotes `true` and contributes `Some(true)`
to coverage. A repeated `Some(Settings.Enabled)` is unreachable and reports
FP3033; omitting `Some(false)` reports that missing pattern.

Regression coverage:

- [Semantic coverage](../../../../crates/fpas-sema/src/tests/stmt/record_constants.rs):
  direct, nested, parenthesized and case-insensitive field paths; scalar aliases;
  Boolean and simple-enum duplicates; defaults, copies and updates; enclosing
  hoisted constants; missing values and computed/mutable rejection.
- [Unit interfaces](../../../../crates/fpas-sema/src/interface/tests/constants.rs):
  transitive projections, aliases, enum identity, duplicates and computed controls.
- [Compiler dispatch](../../../../crates/fpas-compiler/src/tests/control_flow/record_constants.rs)
  and [linked unit execution](../../../../crates/fpas-compiler/src/tests/computed_const/units.rs):
  Boolean and enum branches, nested Result/Option patterns, `is`, and aggregate
  globals containing arrays.
- [Portable encoding](../../../../crates/fpas-unit/tests/interface/record_constants.rs):
  canonical identities, shared nested records, deterministic round trips and
  interface hashes that include known field values.

<a id="c05"></a>

## C05 — P2 — Finite construction checking expands a shared type graph exponentially

- Owner: [crates/fpas-sema/src/check/decl/types/collection/finite.rs:14-32,42-97](../../../../crates/fpas-sema/src/check/decl/types/collection/finite.rs), especially the field traversal at lines 76-80 and removal from the active set at line 92.
- Contract: [docs/pascal/language/types/declaration-order.md:66-91](../../../../docs/pascal/language/types/declaration-order.md) and AP11.1 require finite record graphs to be accepted.
- Actual: the checker tracks the current recursion path but does not retain a completed finite result. Both fields referencing the same next record recursively recheck its entire subtree; the outer loop repeats this from every declared type. For N records each containing two fields of the next type, this performs O(2^N) graph traversal despite an O(N) source/type graph. No values need to be constructed.
- Reproduction: generate `type R0 = record Left: R1; Right: R1; end record;`, continuing through R21, then `type R22 = record Value: integer; end record;`, all in `program FiniteDag; ... begin end.`. Source is 1,214 bytes and 23 type declarations.
- Command: `fpas check --std-lib lib .temp-data/syntax-review-syntax/finite-dag-22.fpas`.
- Bounded observations with the verified current **debug** CLI: depth 8: 242 ms; 12: 259 ms; 16: 636 ms; 20: 8,533 ms; 22: terminated at 15,020 ms with `ETIMEDOUT`. These are diagnostic stress observations during concurrent audit work, not release benchmarks or a claimed production speed ratio. The exponential recurrence is independently visible in the source.
- Impact: tiny legal type files can stall compiler and editor analysis. This is newly relevant to AP11 whole-file finite-construction checking.
- Needed coverage: a shared acyclic type DAG and a recursive graph with shared terminating alternatives. Verify bounded graph work, preferably using visit counts/fixed-point invariants rather than a fragile wall-clock assertion. A finite-type fixed-point/SCC analysis or appropriate memoization should avoid re-expansion while preserving cycle diagnostics.
- Evidence files: `finite.cjs`, `finite22.cjs`, `finite-results.json`, `finite22-results.json`, `finite-dag-*.fpas` under `.temp-data/syntax-review-syntax/`.

The exact 23-type input can be regenerated without constructing any FPAS values:

```python
from pathlib import Path

depth = 22
source = 'program FiniteDag;\n'
for i in range(depth):
    source += f'type R{i} = record Left: R{i + 1}; Right: R{i + 1}; end record;\n'
source += f'type R{depth} = record Value: integer; end record;\nbegin end.\n'
target = Path('.temp-data/syntax-review-syntax/finite-dag-22.fpas')
target.parent.mkdir(parents=True, exist_ok=True)
target.write_text(source, encoding='utf-8')
```

<a id="c06"></a>

## C06 — P2 — A valid program sharing its name with a declaration fails object compilation

- Owner: [crates/fpas-compiler/src/lowering/context/mod.rs:217](../../../../crates/fpas-compiler/src/lowering/context/mod.rs), [crates/fpas-compiler/src/lowering/context/blocks.rs:129-132](../../../../crates/fpas-compiler/src/lowering/context/blocks.rs), [crates/fpas-unit/src/object/mod.rs:80-114,178](../../../../crates/fpas-unit/src/object/mod.rs), and [crates/fpas-compiler/src/object/mod.rs:105-106](../../../../crates/fpas-compiler/src/object/mod.rs).
- The generated entry function is named using the source program name. `define_all_private` exports it into the same object-definition namespace as source functions, globals and record/enum layouts. Object validation then reports a duplicate, even though semantic analysis permits the program heading and a declaration to share a name.
- User-visible current example: [docs/pascal/getting-started/first-program.md:6-18](../../../../docs/pascal/getting-started/first-program.md) declares both `program Greet;` and `function Greet(...)`. Its first tutorial cannot be checked through the object build path.
- Minimal reproduction:

```pascal
program Demo;
procedure Demo();
begin
  null;
end procedure;
begin
  Demo();
end.
```

- `fpas check --std-lib lib .temp-data/syntax-review-syntax/program-name-collision.fpas` fails with FP9001: `Register object construction failed: invalid register object: DuplicateName("demo").`
- Also reproduced for `program Counter; var Counter: integer := 0; ...` and `program Point; type Point = record ... end record; ...`.
- The copied CLI without source-stdlib discovery/`--std-lib` takes the direct path and accepts the same source. This difference explains why a direct compiler-only regression test would miss it. The normal repository CLI discovers the source standard library; explicitly passing `--std-lib lib` makes the reproduction independent of executable location.
- Expected: generated entry symbols must not collide with legal source declaration names; this should not require a language-name restriction.
- Needed coverage: real CLI/project object compilation for same-name program/function, program/global and program/type, including case variations. Include the first-program tutorial in executable documentation validation.
- Evidence under `.temp-data/syntax-review-syntax/`: `program-name-collision.fpas`, `program-global-name.fpas`, `program-type-name.fpas`, and `docs/first-program-0.fpas`.
- Attribution: discovered in the current snapshot through documentation verification; not proven to have been introduced by a syntax package.

## Successful cross-package probes

- Read the central README and all AP04/AP09/AP10/AP16/AP17/AP20 package and work-package documents, relevant current handbook pages, implementation modules, and focused tests.
- AP04: recursive type safety, closure capture propagation, mutable callable conservatism, unused call result checking, discarded aggregate/default expressions, returned and imported capture proofs.
- AP09: named mapping metadata, evaluation in source order, parameter-order passing, generic inference, variant fields, callable-value restrictions, named `var` forms. No additional correctness bug found in the exercised paths.
- AP10: named-only/required/default fields, lexical resolution, visibility, alias defaults, declaration environment lowering, ordered field staging, constant/discard classification, and current tests. No additional functional defect found beyond interactions reported above.
- AP16: static/computed classification, immutable binding capture, runtime initialization, case-label restrictions, and imported classification.
- AP17: writable storage/root identity, aliases, exact types, argument lowering, reference lifetimes, direct/function-value calls, named var ordering, mutating intrinsic shared paths.
- AP20: recursive pattern checking, constructor resolution, constant normalization, pattern-matrix exhaustiveness, runtime matching, binding scopes, capture traversal, and `is` condition lowering.

### Executed cross-unit integration probe

Fixture: `.temp-data/syntax-review-semantics/cross_unit/review.fpasprj`; independently compiled source units `src/model.fpas` and `src/api.fpas`, with a program `src/main.fpas`. Initial run had no sidecars. Repeated runs succeeded, and both unit sidecar timestamps stayed unchanged during a further warm run.

```text
.temp-data/syntax-review/bin/fpas.exe run --std-lib lib .temp-data/syntax-review-semantics/cross_unit/review.fpasprj
```

All of these assertions passed on cold and warm runs (exit 0):

- Exported computed constant initialization executes once before the consumer and remains readable across unit boundaries.
- Record construction through a facade's exported type alias evaluates supplied fields in written order and retains omitted scalar defaults.
- Imported named ordinary/generic calls and enum variant constructors evaluate side-effect traces in written order and bind to declared roles.
- Imported routine values preserve `var` modes; positional calls through those values and named forwarding through a second unit update caller state.
- Named `var` arguments with indices in different arrays evaluate each index once in written order and swap the expected elements.
- Imported task-free closure results and imported routine values retain discard proofs.

Negative variants of the same project produced the expected errors:

| Variant | Expected and observed |
| --- | --- |
| `Swap(var Total, var Review.Model.Total)` after unaliased import | FP3029, same root through short/qualified names |
| Named call through imported routine value `F(Value := var N, Amount := 1)` | FP3026; additionally a cascading FP3027 |
| `when Model.Computed:` scalar label | FP3014 |
| `discard Model.Unsafe(go Work())`, with returned closure capturing task | FP3021 |

Positive main was restored afterward. The probe sources were formatted successfully. These are additional audit probes, not committed regression tests.

## Scope limits

The concrete failures above were reproduced, but arbitrary nesting, generic substitutions, concurrency schedules and corrupt persisted artifacts were not exhaustively enumerated. No fixes or permanent regression tests were added. Known debugger limitations are separate from these compiler findings.

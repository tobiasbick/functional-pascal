# Compiler and semantic findings

See [scope, priorities and verification](README.md). These are confirmed current-contract defects; package attribution is not a historical introduction bisect. Commands run from the repository root. Complete minimal sources are included where practical; save them under the named ignored scratch path to repeat the checks. The copied executable used for audit is identical to the freshly built `target/debug/fpas.exe`; either can be used with `--std-lib lib`.

<a id="c01"></a>

## C01 — P1: enum comparison patterns are lowered by spelling instead of resolved constant identity

Packages: AP20.2 and AP20.3.

Code: [crates/fpas-compiler/src/lowering/case/patterns.rs:150-165](../../../../crates/fpas-compiler/src/lowering/case/patterns.rs), especially 152-153. `lower_value_test` takes the last identifier of an enum-typed designator and turns it into a variant test whenever that spelling occurs in the enum. It never checks whether semantic resolution selected an enum member or a nearer constant with that same name. The normal constant-expression comparison at 167-178 is bypassed.

Contract: [docs/pascal/language/pattern-matching/syntax.md:40-42](../../../../docs/pascal/language/pattern-matching/syntax.md) says a plain identifier compares with the constant or member it names. [docs/pascal/language/pattern-matching/exhaustiveness.md:113](../../../../docs/pascal/language/pattern-matching/exhaustiveness.md) gives named simple-enum constants their evaluated coverage. [docs/pascal/language/pattern-matching/is-test.md:23-26](../../../../docs/pascal/language/pattern-matching/is-test.md) reuses the case pattern rules.

Reproduction: `pattern_shadowed_enum_constant.fpas`:

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
.temp-data/syntax-review/bin/fpas.exe check --std-lib lib .temp-data/syntax-review-semantics/pattern_shadowed_enum_constant.fpas
.temp-data/syntax-review/bin/fpas.exe run --std-lib lib .temp-data/syntax-review-semantics/pattern_shadowed_enum_constant.fpas
```

Observed: check succeeds (exit 0); run prints `missed` and fails with FP5010 `panic: exhaustive case reached no variant` (exit 2). Expected: `matched`, then `blue`, exit 0. Semantic coverage correctly treats the local `Red` as `Shade.Blue`, while runtime matching treats it as `Shade.Red`, so the checker and emitted program disagree.

Missing regression: [crates/fpas-compiler/src/tests/control_flow/pattern_bindings.rs](../../../../crates/fpas-compiler/src/tests/control_flow/pattern_bindings.rs) covers integer comparison shadowing and nested matching but not a simple-enum constant shadowing a variant name. Add both `is` and nested case tests; include parenthesized/qualified constant controls and an actually named enum member.

<a id="c02"></a>

## C02 — P2: sibling nested calls lose transitive reference captures and trigger an internal compiler error

Package: AP17.1, with closure/capture infrastructure.

Code: [crates/fpas-sema/src/check/closures/capture/mod.rs:100-104](../../../../crates/fpas-sema/src/check/closures/capture/mod.rs) ignores Function/Procedure symbols. `capture/traversal.rs:325-328` only calls `consider_name` for a call target. Transitive captures are imported for nested declarations (`traversal.rs:39-60`) but not for a sibling routine referenced by a call. Consequently `Second` has no capture for the enclosing `var Value` even though calling `First` needs it. The reference escape checker ([crates/fpas-sema/src/check/references/escapes.rs:52-68](../../../../crates/fpas-sema/src/check/references/escapes.rs)) also relies on the incomplete capture metadata.

Contract: [docs/pascal/language/functions/var-parameters.md:101-108](../../../../docs/pascal/language/functions/var-parameters.md) permits direct named nested calls using an enclosing reference and rejects escaping routine values with FP3030.

Valid direct-call reproduction: `reference_indirect_direct.fpas`:

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

Command:

```text
.temp-data/syntax-review/bin/fpas.exe check --std-lib lib .temp-data/syntax-review-semantics/reference_indirect_direct.fpas
```

Observed: FP9001 `Local Value was not present in register-lowering scope metadata` at `First()` inside `Second`, exit 1. Expected: check succeeds, runtime prints 1.

Additional reproduction `reference_indirect_escape.fpas` changes the enclosing routine to `function Make(var Value: integer): procedure()` and returns `Second`. That invalid escape should report FP3030, but check and run both reach the same FP9001 instead. No unsafe execution was observed: compilation stops.

Missing regression: sema's `nested_routines_may_use_enclosing_var_parameters_while_the_call_runs` and compiler's `var_parameters_forward_and_reach_nested_routines_and_function_values` cover direct captures, not sibling-call transitivity. Need a valid direct sibling chain and rejection of returned/spawned wrappers.

<a id="c03"></a>

## C03 — P2: pattern bindings discard callable capability metadata

Packages: AP20 with AP16/AP04.

Code: [crates/fpas-sema/src/check/stmt/control_flow/conditions.rs:74-85](../../../../crates/fpas-sema/src/check/stmt/control_flow/conditions.rs) creates every `is` binding with `task_bound: false`, and `if_case/mod.rs:128-140` does the same for case payload bindings. Neither path propagates the matched value's discard proof. The bindings retain only names and types.

Contract: [docs/pascal/language/functions/closures.md:105-108,117-129](../../../../docs/pascal/language/functions/closures.md) forbids mutable-capturing callables crossing a task boundary and labels this a compile-time error. [docs/pascal/language/functions/discard.md:81-86](../../../../docs/pascal/language/functions/discard.md) promises known capture information through immutable bindings and aggregates.

Reproduction `pattern_task_escape.fpas`:

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
.temp-data/syntax-review/bin/fpas.exe check --std-lib lib .temp-data/syntax-review-semantics/pattern_task_escape.fpas
.temp-data/syntax-review/bin/fpas.exe run --std-lib lib .temp-data/syntax-review-semantics/pattern_task_escape.fpas
```

Observed: check succeeds (exit 0); run fails at `go F()` with FP5018 (exit 2). `case_task_escape.fpas`, using `case Wrapped of when Some(const F): ... when None: null; end case;`, has the same behavior. `direct_task_escape.fpas`, changing the callee to the known `Change`, correctly fails checking with FP3016. Expected: FP3016 during checking for all three forms. Runtime protection prevents the task-bound closure from actually crossing the boundary, so this is a missing static guarantee rather than an observed data race.

The opposite direction also fails: `pattern_discard_proof.fpas` wraps capture-free `Work` in `const Wrapped: Option of function(): integer := Some(Work);`. `discard Wrapped` is accepted, but inside `if Wrapped is Some(const F) then discard F; end if`, the extracted callable is rejected with FP3021 because its proof was dropped. This is a precision gap from the same metadata-loss site; it is not evidence of unsafe discard.

Missing regression: combine case/is payload extraction with task-bound callables and discard-proven callables. Existing AP20 tests predominantly bind scalar/record values; existing discard tests exercise aggregate construction and returns but not extraction through pattern bindings. Include `while`, nested payloads, and task-free controls when repairing the metadata propagation.

<a id="c04"></a>

## C04 — P2: record-derived compile-time constants do not contribute their values to pattern coverage

Packages: AP20.2 with AP16/AP10.

Code: [crates/fpas-sema/src/check/decl/consts/scalar.rs:18-22](../../../../crates/fpas-sema/src/check/decl/consts/scalar.rs) only looks up the full designator as a symbol and cannot evaluate record field selections. Such constants still get compile-time classification, but no scalar value. [crates/fpas-sema/src/check/stmt/control_flow/if_case/patterns/values.rs:85-98](../../../../crates/fpas-sema/src/check/stmt/control_flow/if_case/patterns/values.rs) therefore represents them as `Pat::Other` instead of a Boolean constructor.

Contract: [docs/pascal/language/basics/constants.md:33-36](../../../../docs/pascal/language/basics/constants.md) retains compile-time-known aggregate/constant forms. [docs/pascal/language/pattern-matching/exhaustiveness.md:113](../../../../docs/pascal/language/pattern-matching/exhaustiveness.md) says named compile-time Boolean and simple-enum constants contribute the same coverage as their values. Existing sema test `computed_constructor_defaults_participate_in_const_classification` explicitly accepts a record constant field as a case label.

Reproduction `pattern_record_constant.fpas`:

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

Command:

```text
.temp-data/syntax-review/bin/fpas.exe check --std-lib lib .temp-data/syntax-review-semantics/pattern_record_constant.fpas
```

Observed: FP3011 `Non-exhaustive case: missing Some(true)`, exit 1. Expected: accepted exhaustive match; `Enabled` is a compile-time constant equal to true. This may likewise miss duplicate/unreachable comparisons when values are obtained from static aggregate fields; that consequence follows the same evaluator path but was not separately reproduced.

Missing regression: cross AP10/AP16 compile-time record fields with AP20 coverage, rather than testing classification and scalar literal folding separately. Tests should cover direct field labels, transitive named constants, enum fields, and duplicate comparisons.

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

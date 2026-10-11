# Compiler and semantics repair plan

This is the **Spec** axis of the [current review](README.md). Findings apply to
revision `34397373799a5897dfcfc09e754d16fcad3c7386`. They concern already agreed
behavior and do not authorize new language rules.

## S01 — P1: decision expressions lose complementary Result type information

Status: repaired and verified. Owner: AP21.1/AP21.2 semantic branch typing.

The shared decision join reuses `merge_inferred_types` to retain compatible
payload evidence recursively and checks each branch against its expected type.
Handbook rules are unchanged. Semantic regressions cover both branch orders,
both Result payloads, three-branch conflicts and nested Option/array payloads.
The CLI diagnostic table proves rejection before lowering for all four commands;
the FPAS runner verifies selected-branch-only Result evaluation.

### Requirement and reproduction

[AP21](../ap21-decision-expressions/README.md) requires: “With an expected type
(a declared binding, parameter, field, or return type), every branch is checked
against it.” The [handbook](../../../pascal/language/control-flow/if-then-else.md#if-expressions)
requires assignment-compatible branch types and a result matching its context.

At the reviewed revision, this program passes `fpas check` but fails during `fpas run` with FP5008,
“Numeric operation requires numeric operands, got string and integer”:

```pascal
program BranchTypeErasure;
uses Std.Console;

begin
  const Value: Result of (integer, string) :=
    if false then Error('unused') else Ok('wrong') end if;
  WriteLn(Value.Unwrap() + 1);
end.
```

Replacing the initializer with this expression has the same failure:

```pascal
case 2 of
  when 1: Error('unused');
  else Ok('wrong');
end case
```

The direct initializer `Ok('wrong')` correctly fails with FP3006. Thus the
decision join admits an invalid program, rather than merely rejecting a valid
inference case.

### Cause and repair

- [decision.rs](../../../../crates/fpas-sema/src/check/expr/decision.rs),
  `shared_branch_type`, lines 49–76: compatible branches are joined by choosing
  the whole type with fewer placeholders. Equal counts keep the earlier type.
  `Result of (<error>, string)` therefore survives and erases the concrete
  success payload from the later `Result of (string, <error>)`.
- [expr/mod.rs](../../../../crates/fpas-sema/src/check/expr/mod.rs), lines 83–102:
  passing expected payload types down does not itself check scalar assignments.
- [case_expression.rs](../../../../crates/fpas-sema/src/check/stmt/control_flow/if_case/case_expression.rs)
  uses the same join.

Repair shared branch typing so it retains compatible evidence recursively and
validates every branch against the expected type. Inspect the existing
[inference merge](../../../../crates/fpas-sema/src/types/inference.rs) before
adding another merging implementation. Preserve conflicting-type rejection and
the current no-widening rule.

### Tasks and acceptance

- [x] Extend semantic decision-expression tests with both branch orders,
  the error-payload mirror, a three-branch conflict, and nested containers.
- [x] Repair the shared join/context check. The examples must fail during
  semantic analysis with FP3006 and useful expected/found information.
- [x] Retain positive `Ok(1)`/`Error('reason')`, `None`/`Some`, and empty/nonempty
  array combinations, including selected-branch-only evaluation.
- [x] Add one real CLI negative regression proving rejection before lowering
  or runtime. Extend existing fixtures rather than duplicating the same case
  at every layer. The current documented language rule remains unchanged.

## S02 — P2: joint constructor inference stops at nested generic callables

Status: confirmed; repair open. Owner: AP24.2/AP24.3 contextual inference.
AP22 exposes this through inferred local construction.

### Requirement and reproduction

The [generic types handbook](../../../pascal/language/types/generics.md), lines
65–73, says all supplied fields contribute together and generic routine values
are instantiated from the expected field signature afterward.
[AP24.2](../ap24-generic-data-structures/02-generic-records.md) claims substitution
through nested containers and callable signatures;
[AP24.3](../ap24-generic-data-structures/03-generic-enums.md) applies contextual
completion to generic callable payloads.

```pascal
program NestedCallables;

type Handler of T = record
  Callbacks: array of function(Value: T): T;
  Value: T;
end record;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

begin
  const A := Handler(Callbacks := [Identity], Value := 3);
end.
```

`fpas check` rejects `Callbacks` with FP3006: expected
`array of function(Value: integer): integer`, found
`array of function(Value: T): T`. Reversing the named fields fails identically.
An enum constructor carrying the same payload fields fails too.

Positive controls succeed: an explicit `Handler of integer` annotation and an
unannotated record with a direct callable field. Invoking the callback in both
controls returns `3`.

### Cause and repair

- [types/inference.rs](../../../../crates/fpas-sema/src/types/inference.rs),
  lines 76–82: `needs_expected_type` recognizes generic routine values only
  when the outermost type is a function or procedure.
- [construction_inference.rs](../../../../crates/fpas-sema/src/check/expr/construction_inference.rs),
  lines 80–92: that false result skips the contextual recheck after the sibling
  field has determined the concrete type argument.
- [calls/contextual.rs](../../../../crates/fpas-sema/src/check/calls/contextual.rs),
  lines 35–44, shares the predicate and needs inspection during repair.

### Tasks and acceptance

- [ ] Traverse supported containers and callable signatures when detecting
  contextual-completion needs. Distinguish an unspecialized routine value
  from a valid enclosing generic parameter; reuse the existing owner.
- [ ] Extend semantic cases for arrays of generic functions/procedures, both
  field orders and enum payloads. Inspect Option/dictionary nesting through
  the same traversal; add cases for distinct risks.
- [ ] Preserve rejection of missing/conflicting evidence and failed generic
  constraints. Do not guess a type for a bare unconstrained generic routine.
- [ ] Add one runtime regression that constructs the inferred container,
  invokes the callback and checks its result. The example and enum equivalent
  must work without a binding annotation.

## S03 — P2: scalar conversions accept type declarations as value arguments

Status: confirmed; repair open. Owners: AP18.1 and AP19.1 value validation.

### Requirement and reproduction

[Subrange conversions](../../../pascal/language/types/subranges.md) require
“exactly one positional integer or subrange value.”
[Distinct conversions](../../../pascal/language/types/distinct-types.md#conversions)
likewise require a value of the underlying or same distinct type.

Both programs fail with FP9001, “The compiler could not lower `unresolved
designator`”, instead of a semantic diagnostic:

```pascal
program SubrangeTypeValue;
type Small = 0..10;
begin
  const Value := Small(Small);
end.
```

```pascal
program DistinctTypeValue;
type UserId = distinct integer;
begin
  const Value: UserId := UserId(UserId);
end.
```

### Cause and repair

- [subrange_conversion.rs](../../../../crates/fpas-sema/src/check/expr/subrange_conversion.rs),
  lines 25–37, accepts the argument solely by its returned `Ty`.
- [distinct_conversion.rs](../../../../crates/fpas-sema/src/check/expr/distinct_conversion.rs),
  lines 38–59 and `check_wrap`, has the same missing distinction.
- [designator.rs](../../../../crates/fpas-sema/src/check/expr/designator.rs),
  lines 25–34, returns a type symbol's type in this value position.
- [subrange_membership.rs](../../../../crates/fpas-sema/src/check/expr/subrange_membership.rs),
  lines 34–62, already distinguishes parenthesized/qualified type designators.
  The right-hand type operand of membership is intentional.

### Tasks and acceptance

- [ ] Reject type declarations in conversion value positions during semantic
  analysis. Put the shared value/type-designator distinction at its proper
  owner instead of duplicating the membership predicate.
- [ ] Extend semantic negative tables for direct, parenthesized, aliased and
  imported-qualified type names, covering both conversion kinds.
- [ ] Preserve valid values, alias conversions, and `Value in RangeType`.
  The diagnostic must show a valid conversion example.
- [ ] Add one CLI regression proving a stable semantic error replaces FP9001.
  No runtime workaround or language change is needed.

## Inspected areas without another confirmed finding

AP03 closed-enum coverage and AP15 optional-handler migration were inspected
with existing tests. AP19 conversion/comparison/constraint/case rules and AP21
branch lowering, constants, coverage and callable provenance were reviewed.

AP18 positive probes passed full-width integer endpoints, forward aliases,
record fields and integer-array membership with a subrange value. AP23 review
covered ordinary/early/implicit/`try` returns, clause order, panic preservation,
current `var` writes, generic/bound calls, captures, defaults and unit reuse.
Imported contract diagnostics and generic bound-method postconditions behaved
as documented; impure omitted defaults were rejected. No contract bypass was
established.

AP24 alias/method/chained-result and nested recursive-record probes passed.
Existing generic-record, generic-enum and local-inference runner tests passed.
No independent AP22 defect was confirmed beyond its interaction with S02.
These are bounded review results, not proof of every feature combination.
Unproven source-map and capture-availability suspicions are excluded.

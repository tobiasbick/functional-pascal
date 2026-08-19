# Task 17 — Enum `case` exhaustiveness uses resolved variants

Status: in progress
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

A `case` arm `Red` that resolves to a **variable**, not variant `Color.Red`, does not count as covering that variant.

## Spec

[`docs/pascal/language/pattern-matching/exhaustiveness.md`](../../../pascal/language/pattern-matching/exhaustiveness.md): every variant must appear (or `else`).

## Bug

`crates/fpas-sema/src/check/stmt/control_flow/if_case/exhaustiveness.rs` takes the last ident of each unguarded label (`Light.Red` → `"Red"`). A local `var Red: Color := Color.Blue` shadows the variant; `case C of Red, Green, Blue` type-checks `Red` as a value match but still treats variant `Red` as covered.

## Fix

Count coverage from the **resolved** label: enum variant vs value equality. Reuse however labels are already type-checked in `if_case/labels.rs`. Do not use the raw last identifier.

## Tests

Extend `crates/fpas-sema/src/tests/stmt/exhaustiveness.rs`:

```pascal
program T;
type Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Red;
  var Red: Color := Color.Blue;
  case C of
    Red, Green, Blue: return
  end
end.
```

Expect `SEMA_NON_EXHAUSTIVE_CASE` (variant `Red` not covered as a variant). A fully qualified `Color.Red, Color.Green, Color.Blue` still `check_ok`.

## Verify

```text
cargo test -p fpas-sema
cargo fmt
```

## Done when

- Shadowed names do not fake exhaustiveness.
- Qualified labels still work.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify exhaustiveness counts only resolved enum-member symbols
- Files changed: case exhaustiveness checker and sema regressions
- Verification: not run
- Blockers: none

---
name: fpas-authoring
description: >
  Guides creating and editing Functional Pascal `.fpas` source files in this repository. Use when
  writing or fixing programs, units, demos, regression tests, `uses` imports, formatting, or file
  placement under `examples/`, `tests/`, or `apps/`. Also use when the user asks how to write FPAS,
  Pascal dialect pitfalls, `fpas fmt`, or where a source file belongs.
---

# FPAS authoring

Project-local guide for writing `.fpas` files. This is **not** the language spec — link to `docs/pascal/` for semantics you do not know.

## Required reads

Before non-trivial edits:

1. [`docs/pascal/README.md`](../../../docs/pascal/README.md) — spec index
2. [`docs/pascal/tools/fmt-style.md`](../../../docs/pascal/tools/fmt-style.md) — formatter output rules
3. [`.agents/skills/fpas-projects/SKILL.md`](../fpas-projects/SKILL.md) — when the file needs a `.fpasprj` or workspace entry
4. [`.agents/skills/fpas-change-checklist/SKILL.md`](../fpas-change-checklist/SKILL.md) — after behavior changes (docs, tests, verify)

Workflow calibration: [references/examples.md](references/examples.md).

## Where files go

| Goal | Location | Entry shape |
|------|----------|-------------|
| Runnable demo / tutorial | `examples/` | `program` — never `*_test.fpas` here |
| Regression / integration test | `tests/<theme>/` | `program` named `*_test.fpas` |
| App source | `apps/<name>/src/` | `program` or `unit` per project |
| Shared library code | library `.fpasprj` `src/` | `unit` only |

`Std.Tui` tests group under `tests/stdlib/tui/`. Bundle all regression tests via [`tests/suite.fpasprj`](../../../tests/suite.fpasprj).

## Decision workflow

1. **Single-file scratch?** → `program` file; run with `fpas run path.fpas` or `fpas check path.fpas`.
2. **Multi-file app or library?** → follow `fpas-projects` skill; add units + `.fpasprj`.
3. **Assert runtime behavior?** → `*_test.fpas` under `tests/` with `uses Std.Test`.
4. **Teach a feature?** → `examples/` demo; keep it short and runnable.

## Minimal skeletons

### Program (demo or app entry)

```pascal
program MyApp;

uses Std.Console;

begin
  WriteLn('Hello')
end.
```

### Unit (shared code)

```pascal
unit MyApp.Utils;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value
end;
```

### Regression test

```pascal
program AbsNegativeTest;

uses Std.Math, Std.Test;

begin
  AssertEquals(7, Abs(-7))
end.
```

Test entry files must be `program`, not bare `unit`. Spec: [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md).

Filesystem scratch for FPAS tests/demos: write under `.temp-data/` at the repository root (gitignored). Never leave fixtures in `crates/`, beside `tests/`, or as bare `_fpas_*` files in the cwd.

## FPAS rules models often get wrong

Do **not** assume Delphi/Free Pascal:

| Wrong (other Pascal) | FPAS |
|----------------------|------|
| `FuncName := value` return | `return value` only |
| `var x: Integer` mutable by default | `var` immutable; use `mutable var` to reassign |
| `WriteLn('x');` required before every `end` | semicolons **separate** statements; no trailing `;` before `end` / `else` / `until` |
| `uses Unit1, Unit2 in interface` | single `uses` clause; no Delphi `interface`/`implementation` split |
| untyped lambda shorthand | use an anonymous `function` / `procedure` expression with explicit parameter and result types; use a named nested routine for implicit recursion |
| `begin`/`end.` optional on programs | formatter inserts them — match [`fmt-style.md`](../../../docs/pascal/tools/fmt-style.md) |
| `{...}`, `(*...*)`, or a separate doc-comment delimiter | `//` is the only comment syntax; an adjacent standalone block is Markdown documentation |

Other habits:

- Case-insensitive keywords and identifiers.
- Strings use single quotes: `'Hello'`, escape with doubled quote: `'It''s'`.
- `Std.*` units require explicit `uses` — listing a file in `.fpasprj` does not import it.
- Qualify ambiguous short names (`Length`, `Map`, `Unwrap`, …) with the unit: `Std.Str.Length`, `Std.Array.Length`.
- Unit declarations and record members are private by default. Write `public`
  directly before each exported declaration or member. `public` is valid in
  **units** only, not `program` files; `private` is an ordinary identifier.

Canonical syntax reference: [`docs/specs/grammar.ebnf`](../../../docs/specs/grammar.ebnf). Language topics: [`docs/pascal/language/`](../../../docs/pascal/language/).

## Formatting

After editing `.fpas` under `examples/`, `tests/`, or `apps/`:

```text
fpas fmt <paths>
fpas fmt --check <paths>
```

Or batch format:

```text
scripts/format-fpas-sources.sh
scripts/format-fpas-sources.ps1
```

Golden style rules: [`docs/pascal/tools/fmt-style.md`](../../../docs/pascal/tools/fmt-style.md). Prefer formatter output over hand-aligned spacing.

## Canonical repo examples

Copy patterns from real files instead of inventing syntax:

| Pattern | Repo file |
|---------|-----------|
| Hello world | [`docs/pascal/getting-started/hello-world.md`](../../../docs/pascal/getting-started/hello-world.md) |
| Stdlib assertion test | `tests/stdlib/math/abs_negative_integer_test.fpas` |
| Headless TUI test | `tests/stdlib/tui/mvu_host_signature_test.fpas` |
| Unit + `uses` | [`docs/pascal/program-structure/units.md`](../../../docs/pascal/program-structure/units.md) |
| Library + program | `examples/pascal/monorepo/` |

## When done

- Behavior or API changed → `fpas-change-checklist`
- New test → add path to `tests/suite.fpasprj` when outside existing `include` globs
- Project manifest changed → `fpas-projects` + `fpas check`

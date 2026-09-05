# FPAS authoring examples

Calibration for `fpas-authoring`. User request → expected agent behavior.

## Example 1: New stdlib regression test

User request:

```text
add a test that Str.Trim removes spaces
```

Expected behavior:

- Create `tests/stdlib/str/trim_normal_test.fpas` (or match nearby naming in that folder).
- Use `program …Test;` + `uses Std.Str, Std.Test;` + `AssertEquals(…)`.
- Do **not** put it under `examples/`.
- Run `fpas fmt` on the new file; confirm path is covered by `tests/suite.fpasprj` globs.
- If behavior is already documented, no doc change; otherwise follow `fpas-change-checklist`.

## Example 2: Small runnable demo

User request:

```text
add an example showing dict merge
```

Expected behavior:

- Add under `examples/` (themed subdir matching existing layout).
- Use `program`, not `*_test.fpas`.
- `uses` the documented `Std.Dict` symbols; link to [`docs/pascal/std/collections/dict.md`](../../../../docs/pascal/std/collections/dict.md).
- `fpas fmt` the file; verify with `fpas check` or project run if wired in an example project.

## Example 3: Extract helper from a growing program

User request:

```text
split helpers out of main.fpas
```

Expected behavior:

- Create a `unit` file in the same project `src/` tree.
- Move routines into the unit; keep `program` as entry with `uses MyUnit`.
- Ensure the unit is picked up by the project `[sources].include` glob.
- `fpas check <project>.fpasprj` — listing in `.fpasprj` alone does not import; `uses` does.

## Example 4: Fix formatting-only drift

User request:

```text
format all tests
```

Expected behavior:

- Run `scripts/format-fpas-sources.sh` or `fpas fmt tests/`.
- Do not hand-edit spacing to match memory — match [`fmt-style.md`](../../../../docs/pascal/tools/fmt-style.md).
- No new tests or docs unless fmt exposes a syntax error.

## Example 5: Model wrote Delphi syntax

User request:

```text
fix the compile error in my.fpas
```

Expected behavior:

- Read the diagnostic; common fixes: `return` instead of `FuncName :=`, add `mutable var`, qualify `Length`/`Map`, add missing `uses Std.*`.
- Re-check with `fpas check my.fpas`.
- Grep a similar working file under `tests/` or `examples/` before guessing syntax.

## Example 6: TUI smoke test

User request:

```text
add a test that the TuiApplication host quits when an update sets TuiCmd.Quit
```

Expected behavior:

- Add under `tests/stdlib/tui/`.
- Follow `tests/stdlib/tui/mvu_host_signature_test.fpas` for deterministic host setup and message processing.

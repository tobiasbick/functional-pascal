# Task 30 — Formatter: `#` codes for unprintable string bytes; fmt globs must not write through symlinks

Status: open
Severity: P2
Difficulty: easy (fmt codes) / medium (symlinks)
Language gate: no
Depends on: none

Two independent fixes in one task because both are formatter/CLI output safety. Implement both; they do not share code.

---

## 30a — String literals

### Goal

`format_string` emits `#0`, `#9`, `#13` (or `#13#10`) for unprintable bytes instead of raw control characters inside quotes. Unix `\n` in formatted **source** follows fmt-style (LF file newlines are separate from string contents).

### Spec

[`docs/pascal/tools/fmt-style.md`](../../../pascal/tools/fmt-style.md) and number/string literal language pages: unprintable content uses `#` codes.

### Bug

`crates/fpas-fmt/src/emit/expr/literal.rs` `format_string` writes every char inside quotes. Lexer concatenates `#0` into the value; format injects NUL/CR.

### Fix

For chars that are not printable ASCII (or not valid in a Pascal quoted string), emit `#nnn` concatenation as the lexer/parser already accept. Keep `''` for apostrophe. Round-trip must parse.

### Tests

`crates/fpas-fmt/tests/`: source `WriteLn(#0);` or `'hello'#13#10` golden / `assert_round_trip`. Assert formatted text does not contain a raw NUL.

---

## 30b — Glob symlinks

### Goal

`fpas fmt '*.fpas'` must not follow a symlink and `fs::write` a target outside the tree. Directory walk already skips non-file/non-dir; glob uses `entry.is_file()` which follows links.

### Bug

`crates/fpas-cli/src/cli_fmt/paths.rs`.

### Fix

Treat symlink-to-file like the directory walk: skip, or format only if the **canonical** target is still under the fmt root. Prefer skip unless `--follow-symlinks` already exists (it should not — do not add a flag unless cli.md already has one).

### Tests

If the CLI test harness can create a symlink (Windows: `std::os::windows::fs::symlink_file` may need privilege — skip the test when symlink creation fails). On Unix, create a temp dir, symlink to a file outside, glob, assert the outside file’s mtime/content unchanged.

---

## Verify

```text
cargo test -p fpas-fmt
cargo test -p fpas-cli
cargo fmt
```

## Done when

- `#` codes round-trip without raw controls in output.
- Glob skip/protect works where the OS allows the test.
- Docs: fmt-style example for `#0` only if you add a golden block; otherwise unchanged.

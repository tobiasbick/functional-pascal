# Task 05 — Preserve `//` comments on `uses` items

Status: complete
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

`fpas fmt` must keep every `//` comment attached to `uses` unit names.

## Spec

[`docs/pascal/tools/fmt-style.md`](../../../pascal/tools/fmt-style.md) section **Comments**: all `//` comments are preserved.

## Bug

`crates/fpas-fmt/src/comments/traversal.rs` registers `uses` name spans as leading anchors (`collect_program` / `collect_unit`). `crates/fpas-fmt/src/emit/program.rs` `emit_optional_uses` only prints rewritten names — it never reads those anchors. Example:

```pascal
uses Std.Console, // io
  Std.Conv;
```

The `// io` comment is dropped (or stolen and then dropped). Comments **before** the `uses` keyword still work via `uses_anchor()`.

## Fix

Emit leading/trailing comments for each uses item while printing the comma list, **or** emit collected uses-item comments in a way `assert_round_trip` accepts. Look at how declaration lists emit leading comments and reuse that helper. Do not invent a second comment parser.

Keep `uses` wrapping/style from `emit_wrapped_comma_list` if you can attach comments without breaking wrapping. If wrapping fights comments, preserving comments wins.

## Tests

In `crates/fpas-fmt` (unit test next to `preserves_comments_before_uses_begin_and_end_of_line` in `src/comments/map.rs`, **and** a `tests/` golden or `assert_round_trip`):

```pascal
program T;
uses Std.Console, // io
  Std.Conv;
begin
end.
```

Assert the formatted source still contains `// io` and that a second format is identical (`assert_round_trip`).

Also a comment standing alone between two uses items.

## Verify

```text
cargo test -p fpas-fmt
cargo fmt
```

## Done when

- Uses-item comments survive format and a second format.
- Existing fmt corpus / comment tests still pass.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify comment-aware uses emission and round-trip regressions
- Files changed: `crates/fpas-fmt/src/comments/traversal.rs`,
  `crates/fpas-fmt/src/comments/map.rs`, `crates/fpas-fmt/src/emit/program.rs`,
  `crates/fpas-fmt/tests/comment_regressions.rs`
- Verification: full workspace definition of done passed on 2026-08-19
- Blockers: none

# Task 25 — Honor spec-absolute paths on Windows

Status: open
Severity: P2
Difficulty: easy
Language gate: no
Depends on: none

## Goal

A project path `/home/user/shared/common.fpas` (spec absolute-include example) is not joined as `root\home\user\...` on Windows. Drive-relative `C:foo` should not be treated as “relative to the project root” via `Path::join` in a surprising way — pick the behavior `projects.md` states and test it.

## Spec

[`docs/pascal/program-structure/projects.md`](../../../pascal/program-structure/projects.md) absolute include examples.

## Bug

`crates/fpas-project/src/paths.rs` `resolve_path` uses `Path::is_absolute()`. On Windows that is false for POSIX-style `/home/...` and for `C:foo`. `PathBuf::push` then treats a rooted-but-not-absolute path as current-drive `\home\user\...`.

Relative `../lib/...` and `C:\...` already work.

## Fix

Treat a path as absolute if `is_absolute()` **or** it starts with `/` **or** it is a Windows UNC / `X:\...`. Implement with a small helper; do not use `unwrap` on UTF-8.

Do not access a real `/home/user` directory in tests. Assert the **resolved PathBuf** is the input as-is (or as `PathBuf::from("/home/...")`), not `root_dir.join(...)`.

## Tests

Unit test `resolve_path("/abs/foo.fpas", Path::new(r"D:\proj"))` (or `C:\proj`) does not contain the project root as a prefix. `resolve_path("src\\a.fpas", root)` still joins.

## Verify

```text
cargo test -p fpas-project
cargo fmt
```

## Done when

- POSIX-absolute strings stay absolute on Windows.
- Normal relative paths still join.
- Docs unchanged.

# Task 34 — Do not format through globbed symlink files

Status: complete

## Progress

- Implementation commit: 74b16b7b
- Current step: complete; remove this task after the completion cleanup is committed
- Verification: cargo fmt --all -- --check, cargo build --workspace, and
  cargo test --workspace --no-fail-fast passed on 2026-08-19
- Docs: current user-facing documentation was included or confirmed by the implementation slice
- Blockers: none
Severity: P2
Difficulty: easy/medium
Language gate: no
Depends on: none

## Goal

`fpas fmt <glob>` never follows a symlink-to-file and writes its target, especially outside the
intended tree.

## Verified cause

`fpas-cli/src/cli_fmt/paths.rs::expand_glob` accepts `entry.is_file()`, which follows symlinks.
Directory walking already avoids symlink traversal, so input modes have inconsistent safety.

## Fix

Inspect `symlink_metadata` before accepting a glob result and skip every symlink. Use the same policy
for direct file arguments if they currently follow symlinks; formatting behavior should not depend
on how the same path was selected. Do not add `--follow-symlinks` as part of this fix.

If every match is skipped, return an actionable “no regular `.fpas` files” error rather than
silently succeeding.

## Tests

- On Unix, glob a symlink to an outside `.fpas` file and prove target content remains unchanged.
- On Windows, run the same test when symlink creation is permitted; skip only on the OS privilege
  error, not on formatter failure.
- Regular globbed and direct files still format.

## Verify

```text
cargo test -p fpas-cli
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- All formatter input modes share an explicit regular-file/no-symlink policy.
- A skipped-only glob fails clearly.
- CLI docs are updated only if they currently promise symlink following.

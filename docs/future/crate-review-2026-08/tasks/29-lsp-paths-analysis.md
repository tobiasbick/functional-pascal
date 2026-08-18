# Task 29 — Language service: fail closed less, bound discovery, no ParentDir-through-root

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

1. One unreadable sibling project file must not drop diagnostics for the open buffer; convert I/O errors to diagnostics or skip that file.
2. Project discovery must not walk above the workspace root when `Path::starts_with` fails (Windows `d:\` vs `D:\`).
3. `lexical_normalize` must not pop `RootDir` / drive root on `ParentDir`.

## Bug

- `crates/fpas-language-service/src/analysis/mod.rs`: `analyze_document` loads every project source with `?`. `DiagnosticPublisher::schedule` logs and skips `publishDiagnostics`.
- `crates/fpas-language-service/src/workspace/discovery.rs`: if `source.starts_with(&root)` is false, walk to filesystem root looking for `.fpasprj`.
- `crates/fpas-language-service/src/document/mod.rs` `lexical_normalize`: `ParentDir` is `pop()` with no root guard, so `/../etc/passwd` can become a relative store key.

## Fix

Analyze the open document even if a sibling fails; attach a diagnostic “could not read X”. Case-fold or canonicalize before `starts_with`; if still outside root, **do not** walk up. For normalize: if components are empty besides root, ignore further `ParentDir`.

Do not follow directory symlinks in the up-walk (folder catalog already skips them).

## Tests

Crate tests next to existing discovery/document tests:

- Missing sibling → still some diagnostics publish for the current file (or analysis `Ok` with a warning diagnostic).
- `lexical_normalize` of a path with extra `..` does not leave the drive/root.
- Discovery given a path that differs only by drive-letter case stays inside the workspace (if you can construct that without touching the real disk).

## Verify

```text
cargo test -p fpas-language-service
cargo test -p fpas-lsp
cargo fmt
```

## Done when

- Sibling I/O does not freeze stale squiggles with no update.
- `..` cannot escape the path root in the store key.
- Discovery stays in-workspace.
- Docs unchanged.

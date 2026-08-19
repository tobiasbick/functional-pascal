# Task 29 — Publish current diagnostics when a sibling source is unreadable

Status: open
Severity: P2
Difficulty: hard
Language gate: no
Depends on: none

## Goal

An unreadable or vanished sibling project source must not leave stale diagnostics for the open
buffer. The client receives a current publication that explains the project I/O failure and retains
any syntax diagnostics available from the open snapshot.

## Verified cause

`language-service/analysis/mod.rs::project_snapshots` collects every snapshot with `?`. The error
reaches `fpas-lsp/diagnostics/publisher.rs`, which logs and returns without publishing, so the editor
keeps the previous diagnostic generation indefinitely.

## Fix constraints

- Do not silently omit a sibling and run sema over an incomplete unit graph as though it were valid.
- Produce a version-current publication for the requested URI: open-buffer syntax diagnostics plus
  an actionable project-source I/O diagnostic, or a dedicated analysis-failure diagnostic if the
  service representation cannot safely combine them.
- Keep navigation/semantic tools fallible; this task is specifically about diagnostics freshness.
- Do not expose host metadata beyond the source path already owned by the loaded project.

Prefer a typed analysis failure that the publisher can convert over parsing log strings.

## Tests

- Analyze/publish once successfully, remove or make a sibling unreadable, change the open document,
  and assert a newer `publishDiagnostics` arrives rather than retaining the old generation.
- The new publication identifies the sibling I/O problem and includes an open-buffer syntax error
  when present.
- Restoring the sibling allows semantic diagnostics again.

## Verify

```text
cargo test -p fpas-language-service
cargo test -p fpas-lsp
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Every scheduled current generation either publishes diagnostics or is explicitly superseded.
- Sibling I/O failures cannot freeze stale squiggles.
- Docs unchanged.

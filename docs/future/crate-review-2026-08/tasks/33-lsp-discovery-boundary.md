# Task 33 — Keep document-driven discovery inside the workspace root

Status: complete

## Progress

- Implementation: included in this review batch
- Implementation: workspace containment and terminal-root equality use one component-aware platform
  policy without canonicalizing nonexistent buffers
- Tests: Windows case-only containment, missing inside document, genuine outside discovery, and
  non-Windows native case behavior
- Verification: full language-service, LSP, and workspace definition of done passed on 2026-08-19
- Docs: unchanged; discovery policy is unchanged apart from corrected path identity
- Blockers: none
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Project discovery for an open document never walks above the configured workspace root merely
because Windows path casing differs.

## Verified cause

`workspace/discovery.rs::discover_source_context` uses case-sensitive `Path::starts_with` to decide
whether its upward walk is bounded. Existing paths often canonicalize to matching casing, but an
unsaved/nonexistent editor path can remain lexical (`d:\...` versus canonical `D:\...`) and disable
the bound.

## Fix

Create one platform-aware path containment comparison used for the bound and terminal root check.
On Windows compare normalized components case-insensitively; on other platforms preserve native
case sensitivity. Preserve the existing policy for a genuinely outside-root document; this task
only prevents an inside path with different Windows casing from being misclassified as outside.

Do not follow directory symlinks or canonicalize nonexistent buffers as a requirement.

## Tests

- Pure Windows-style component test for drive-letter/directory case differences.
- Nonexistent document path remains bounded to the workspace root.
- A genuinely outside path retains its existing document-driven discovery behavior.
- Non-Windows case-sensitive behavior remains covered.

## Verify

```text
cargo test -p fpas-language-service
cargo test -p fpas-lsp
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Case-only differences cannot disable the boundary.
- Inside-root documents stop at the configured root even when lexical casing differs.
- Docs unchanged.

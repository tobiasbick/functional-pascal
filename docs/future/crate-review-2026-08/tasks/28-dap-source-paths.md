# Task 28 — Preserve original and portable DAP source identities

Status: complete

## Progress

- Implementation: included in this review batch
- Implementation: portable identities now retain optional non-serialized original-path aliases;
  DAP resolution is path-flavor aware, rejects ambiguity, and refreshes on reload/rollback
- Tests: Windows casing, original-to-portable lookup, breakpoint lookup, ambiguous filenames,
  reload/rollback aliases, and compiled-image source-root verification
- Verification: full fpas-debug, full fpas-cli, and workspace definition of done passed on
  2026-08-19
- Docs: unchanged; portable protocol and recording identities remain unchanged
- Blockers: none
Severity: P1
Difficulty: hard
Language gate: no
Depends on: none

## Goal

DAP source requests and breakpoints resolve when Windows path casing differs and when an out-of-tree
source has a portable `sources/{index}/{filename}` image identity. Reload must preserve the mapping.

## Verified cause

- `dap/server/breakpoints.rs::resolve_source_path` compares exact/case-sensitive suffix strings.
- `fpas-cli/src/cli_debug.rs::portable_path` replaces an out-of-tree source's real path with an
  alias.
- `DebugSourceContent` stores only the portable path and content, so the adapter cannot match the
  original path later.

## Required implementation

1. Extend debugger source metadata with an optional original host path plus the portable canonical
   identity. Keep protocol responses and executable source maps portable; do not serialize machine
   paths into `.fpascp` images or recordings.
2. Populate original paths for source/project launches. A prebuilt portable image may have no
   original path; `--source-root` resolution can provide a verified host candidate for that launch.
3. Carry the mapping through `PreparedDebugTarget`, hot reload, rollback, and DAP source lookup.
4. Compare Windows-style paths case-insensitively and separator-normalized. Preserve case-sensitive
   behavior for non-Windows native paths unless an explicit Windows drive/UNC form is detected.
5. Reject ambiguous suffix/original matches instead of picking the first.

Do not write original absolute paths into repository docs, test fixtures, program images, or
recording payloads.

## Tests

- Windows-style casing difference resolves to one portable source.
- Real out-of-tree source path resolves to its `sources/0/...` alias.
- Ambiguous same-filename sources do not bind incorrectly.
- Mapping survives a compatible reload and rollback.
- Compiled image/source-root launch still verifies source hashes.

## Verify

```text
cargo test -p fpas-debug
cargo test -p fpas-cli
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- DAP accepts real and portable identities without leaking host paths into portable artifacts.
- Case handling follows path flavor rather than unconditional lowercasing.
- Reload tests prove the mapping lifecycle.

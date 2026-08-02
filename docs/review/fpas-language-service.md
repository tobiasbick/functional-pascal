# `fpas-language-service` review follow-up

Classification: editor analysis and refactoring safety. No FPAS language change expected.
Status: LS-01 through LS-05 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LS-01 | P1 | `crates/fpas-language-service/src/analysis/cache.rs:18-26`, `src/document/store.rs:33-41,83-84`, `src/analysis/mod.rs:217-223` | Cache identity is only path plus editor version. Close/reopen with changed content and reused version returns analysis from the previous document lifetime. | Add a store-owned lifetime/content revision to cache identity or invalidate every affected cache entry on open/change/close. | Analyze version 1, close, reopen different content as version 1, and assert fresh diagnostics/symbols. |
| LS-02 | P1 | `crates/fpas-language-service/src/navigation/rename.rs:185-195` | Rename checks collisions only in the exact same scope. Renaming a global to an inner local's name can capture edited uses and change semantics. | Simulate resolution at every edit site or inspect all intervening lexical scopes for capture/shadowing, including unedited uses. | Global-to-local and local-to-outer capture cases; re-resolve the edited workspace. |
| LS-03 | P2 | `crates/fpas-language-service/src/navigation/resolve.rs:158-170` | Member-chain termination compares member text to the last text. `Root.Child.Child` stops at the first `Child`. | Track the component index with `enumerate()` and stop only on the final iteration. | Repeated member names across two or more levels for resolve, hover, definition, and completion. |
| LS-04 | P2 | `crates/fpas-language-service/src/navigation/resolve.rs:122-150` | Hierarchical imported units such as `A` and `A.B` are resolved using the first prefix, making results source-order dependent. | Collect candidates and choose the longest matching owner; diagnose true ambiguity explicitly. | Both source orders for `A` and `A.B`, plus ambiguous candidates. |
| LS-05 | P2 | `crates/fpas-language-service/src/document/line_index.rs:64-96` | `LineIndex` accepts any source with the same byte length. Equal-length different text yields wrong positions/ranges. | Bind the index to its source snapshot and remove source parameters, or validate a robust content identity. | Equal-length different line layouts, invalid UTF-8 byte positions, and roundtrip offset/position properties. |

## Implementation notes

Add cancellation tests during active folder/reference scans, not only tokens cancelled before entry. Coordinate LS-01 with LSP document sequencing so cache identity and transport ordering reinforce the same snapshot model.

## Implementation record

- LS-01 adds a store-owned monotonic revision to every parsed snapshot. Analysis cache keys use
  that revision rather than the editor-protocol version, so closing and reopening a buffer with a
  reused client version cannot recover analysis from an older document lifetime.
- LS-02 validates rename against a hypothetical renamed symbol index. Edited references must still
  resolve to the target, and existing uses of the replacement name must retain their declarations.
  Same-scope collisions, inner capture, and new shadowing are rejected; disjoint lexical scopes
  remain renameable.
- LS-03 terminates member-chain traversal by component position rather than text equality. Repeated
  names across nested record members now work for definition, hover, and completion.
- LS-04 checks every imported owner prefix and accepts exactly one complete qualified match. This
  removes document-order dependence for hierarchical units and withholds navigation for genuinely
  ambiguous complete identities. Compiler and FPAS language resolution rules are unchanged.
- LS-05 binds each public `LineIndex` to the immutable `Arc<str>` it indexes. Offset, position, line,
  and range operations can no longer be paired with unrelated equal-length source text.
- Active-scan cancellation was rechecked: folder and reference loops already contain cooperative
  checks. No timing-dependent concurrency regression was added in this slice.

## Verification

- `cargo test -p fpas-language-service --test document_store --test analysis
  --test navigation_resolution` — 17 regressions passed.
- `cargo test -p fpas-language-service --test references_rename --test rename_safety
  --test navigation_resolution` — 12 navigation and rename regressions passed.
- `cargo test -p fpas-lsp --test document_lifetime` — close/reopen transport regression passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — all workspace and doc tests passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.

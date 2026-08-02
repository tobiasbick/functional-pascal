# `fpas-lsp` review follow-up

Classification: LSP transport, concurrency, and user-visible editor behavior. No language change expected.
Status: LSP-01 through LSP-07 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LSP-01 | P1 | `crates/fpas-lsp/src/lib.rs:41`, `src/server/backend.rs:91,110,158` | Parallel request handling can apply `didOpen`, `didChange`, and `didClose` out of order. Changes can precede open or close a freshly reopened document. | Keep requests parallel but serialize mutations per URI through an ordered document event lane. | Concurrent transcripts for open/change, close/reopen, and watcher/editor races. |
| LSP-02 | P1 | `crates/fpas-lsp/src/documents.rs:205`, `src/navigation_requests.rs:28`, `src/semantic_tools_requests.rs:23` | Synchronous compiler analysis runs under a global async mutex. Most requests cannot be cancelled and can block runtime workers plus document notifications. | Introduce a shared blocking-task runner with snapshot capture, cooperative cancellation, and short lock ownership. | Cancellation and fixed responsiveness bounds for completion, symbols, tokens, diagnostics, and workspace refresh. |
| LSP-03 | P2 | `crates/fpas-lsp/src/documents.rs:57`, `src/server/navigation.rs:192`, `src/semantic_tools/code_actions.rs:68` | Rename and quick-fix edits discard the analyzed document version, so stale ranges may be applied after a change. | Use `documentChanges` with versioned `TextDocumentEdit` for open documents. | Change a document after analysis and ensure stale rename/quick-fix edits are rejected or version-labelled. |
| LSP-04 | P2 | `crates/fpas-lsp/src/documents.rs:338`, `src/server/navigation.rs:206`, `src/server/intellisense.rs:65` | Service, task, lifecycle, and position errors are all mapped to `invalid_params`, falsely blaming clients for internal failures. | Map URI/position/not-open to invalid parameters, service/join failures to internal error, and cancellation to request-cancelled. | Exact JSON-RPC codes for parameter, internal, and cancellation paths. |
| LSP-05 | P2 | `crates/fpas-lsp/src/intellisense/completion.rs:16`, `src/server/intellisense.rs:38` | Completion resolve data stores only URI and byte offset. After edits the same offset can refer to another declaration and return foreign docs. | Store source version plus a stable declaration identity; on mismatch return the item unchanged. | Resolve after version change, declaration deletion, and manipulated opaque data. |
| LSP-06 | P3 | `crates/fpas-lsp/src/diagnostics/publisher.rs:82,104` | Lower confidence: generation mutex is held while awaiting diagnostic publication, so backpressure can block close/change/shutdown. | Move publication to a dedicated ordered task that validates generation immediately before send. | Backpressured client with concurrent change/close and bounded shutdown. |
| LSP-07 | P3 | `crates/fpas-lsp/src/server/semantic_tools.rs:72` | Lower confidence: half-open ranges touching only at a boundary are considered overlapping, possibly offering a quick fix at an adjacent position. | Define overlap with half-open LSP semantics and distinguish empty ranges explicitly. | Touching, nested, disjoint, and empty range cases. |

## Implementation notes

The oversized `documents.rs` was replaced by focused lifecycle, result, task,
and error modules. Dispatch-edge sequencing preserves notification order while
query snapshots run on cancellable blocking workers without holding the primary
service lock. Disk snapshots and their exact revisions are shared between
query forks, while editor overlays remain isolated.

Rename and quick-fix results now use versioned `documentChanges` and are
withheld when the client cannot apply them safely. Completion resolve validates
the source revision plus qualified declaration identity. JSON-RPC errors are
classified by cause, diagnostic output uses a dedicated ordered publisher, and
code-action range selection follows half-open LSP semantics.

Regressions cover notification sequencing, cancellation and lock release,
open/disk edit versions, capability fallback, stale/deleted/manipulated
completion identities, exact error codes, diagnostic backpressure, and
touching/nested/disjoint/empty ranges.

Verification passed: `cargo test -p fpas-language-service --test intellisense`,
`cargo test -p fpas-lsp`, targeted Clippy with warnings denied,
`cargo fmt --all -- --check`, `cargo build`, and `cargo test --workspace`.

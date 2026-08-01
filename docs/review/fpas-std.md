# `fpas-std` review follow-up

Classification: Std runtime behavior for filesystem, console, and graph hosting. Check matching pages under `docs/pascal/std/` for every observable change.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| STD-01 | P1 | `crates/fpas-std/src/fs.rs:131-140` | `WriteTextAtomic` renames the existing Windows destination to backup before publishing temp. A crash leaves no target; restore errors are discarded. | Use Windows-native replace semantics or the agreed transactional publication primitive. Preserve and surface failed restoration. | Existing destination, publish failure, restore failure, and stale temp/backup siblings on Windows. |
| STD-02 | P2 | `crates/fpas-std/src/console/interactive.rs:100-137` | Terminal ownership flags are cleared even when releasing paste, mouse, or alternate screen fails. Retry/Drop cannot restore remaining modes. | Clear each ownership flag only after successful restoration and preserve failed state for retry/Drop. | Inject partial release failure, retry, and final Drop; terminal state must recover. |
| STD-03 | P2 | `crates/fpas-std/src/console/key_input/read.rs:171-197,247` | Timeout/Poll reads one event; if it is an ignored key-release event, the API returns no event even when a key press follows within the deadline. | Loop over ignored events, recompute remaining timeout, and drain immediately available ignored events for Poll. | Release followed by Press for both APIs, including deadline boundary. |
| STD-04 | P2 | `crates/fpas-std/src/graph/session.rs:21`, `src/graph/backend/mod.rs:109` | Dropping an open GraphSession does not clear the thread-local backend. The next VM run on the same worker can neither open nor own the orphaned resource correctly. | Bind backend ownership to session RAII and best-effort close/clear in `Drop`. | Drop without explicit Close, then reopen on the same thread; include backend close failure. |

## Implementation notes

`fs.rs` is at the project size threshold and mixes intrinsic dispatch, bounded reads, atomic replacement, and globbing. STD-01 provides a natural split into focused filesystem modules. Use the full Std change matrix when observable APIs or diagnostics change: docs, sema registration, compiler lowering, bytecode, runtime, and tests as applicable.

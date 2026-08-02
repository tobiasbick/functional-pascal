# `fpas-std` review follow-up

Classification: Std runtime behavior for filesystem, console, and graph hosting. Check matching pages under `docs/pascal/std/` for every observable change.
Status: STD-01 through STD-04 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| STD-01 | P1 | `crates/fpas-std/src/fs.rs:131-140` | `WriteTextAtomic` renames the existing Windows destination to backup before publishing temp. A crash leaves no target; restore errors are discarded. | Use Windows-native replace semantics or the agreed transactional publication primitive. Preserve and surface failed restoration. | Existing destination, publish failure, restore failure, and stale temp/backup siblings on Windows. |
| STD-02 | P2 | `crates/fpas-std/src/console/interactive.rs:100-137` | Terminal ownership flags are cleared even when releasing paste, mouse, or alternate screen fails. Retry/Drop cannot restore remaining modes. | Clear each ownership flag only after successful restoration and preserve failed state for retry/Drop. | Inject partial release failure, retry, and final Drop; terminal state must recover. |
| STD-03 | P2 | `crates/fpas-std/src/console/key_input/read.rs:171-197,247` | Timeout/Poll reads one event; if it is an ignored key-release event, the API returns no event even when a key press follows within the deadline. | Loop over ignored events, recompute remaining timeout, and drain immediately available ignored events for Poll. | Release followed by Press for both APIs, including deadline boundary. |
| STD-04 | P2 | `crates/fpas-std/src/graph/session.rs:21`, `src/graph/backend/mod.rs:109` | Dropping an open GraphSession does not clear the thread-local backend. The next VM run on the same worker can neither open nor own the orphaned resource correctly. | Bind backend ownership to session RAII and best-effort close/clear in `Drop`. | Drop without explicit Close, then reopen on the same thread; include backend close failure. |

## Implementation record

- STD-01 replaces the PID/counter temporary and Windows backup/restore sequence
  with `AtomicWriteFile`. Failed commits leave the destination unchanged;
  restore and post-commit backup cleanup phases no longer exist. `fs.rs` was
  split into dispatch, bounded-read, publication, and glob modules.
- STD-02 clears each terminal ownership flag only after that mode restores
  successfully. A partial failure retains only the failed modes for explicit
  retry, and `Console::drop` makes a final best-effort restoration attempt.
- STD-03 routes live reads through an injectable event-source boundary.
  `ReadEventTimeout` recomputes its remaining deadline after ignored release
  events, while `PollEvent` drains immediately ready ignored events before
  returning `None`.
- STD-04 gives `GraphSession` RAII teardown. Explicit close and Drop both
  detach the thread-local backend; close failure still leaves the session
  closed and permits a new session on the same thread.
- No `Std.*` signatures, compiler lowering, bytecode intrinsics, registration,
  FPAS syntax, or language semantics changed.

## Verification

- `cargo test -p fpas-std fs:: --locked` — passed: 16 tests.
- `cargo test -p fpas-std console::tests::interactive --locked` — passed: 5 tests.
- `cargo test -p fpas-std console::key_input::read::tests --locked` — passed: 4 tests.
- `cargo test -p fpas-std console::tests::key_events --locked` — passed: 16 tests.
- `cargo test -p fpas-std graph::tests::session --locked` — passed: 11 tests.
- `cargo clippy -p fpas-std --all-targets --locked -- -D warnings` — passed.
- `cargo test -p fpas-std --locked` — passed: 223 tests plus doc tests.
- `cargo run -p fpas-cli --bin fpas --locked -- test tests/stdlib/fs/fs_write_text_atomic_roundtrip_test.fpas` — passed: 1 FPAS test.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.

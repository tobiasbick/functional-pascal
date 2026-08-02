# `fpas-bundle` review follow-up

Classification: executable packaging and publication. No language change expected.
Status: BUNDLE-01 through BUNDLE-04 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| BUNDLE-01 | P1 | `crates/fpas-bundle/src/publication.rs:37-46` | Windows publication renames the current executable away before installing the replacement. A crash leaves the application path missing; a failed restoration error is discarded. | Prefer an OS replacement primitive or a transaction with explicit rollback and surfaced restoration failure. | Existing destination, injected install failure, injected restore failure, and crash-safe state assertions. |
| BUNDLE-02 | P2 | `crates/fpas-bundle/src/publication.rs:48-50` | Backup cleanup failure reports the whole publication as failed even though the new executable is installed. | Distinguish committed publication from cleanup warning, or make post-commit cleanup best-effort with an explicit diagnostic. | Simulate backup deletion failure and assert the reported state matches the installed executable. |
| BUNDLE-03 | P3 | `crates/fpas-bundle/src/publication.rs:15-19,72-75` | Temporary names are only process-locally unique; stale files plus PID reuse can block a later invocation. | Use collision-resistant identifiers or retry `create_new` on `AlreadyExists`. | Precreate a stale candidate and verify retry and cleanup. |
| BUNDLE-04 | P3 | `crates/fpas-bundle/src/format.rs:135-136` | Bundle decoding fully parses the program and the runner parses the same image again. The duplicate cost is confirmed, but impact is unmeasured. | Return the validated `ProgramImage` from bundle decoding or expose a single validate-and-decode API. | Assert one decode path and preserve all corrupt-image diagnostics; benchmark before claiming speedup. |

## Implementation record

- BUNDLE-01, BUNDLE-02, and BUNDLE-03 replace the process-local temporary and
  Windows backup/restore sequence with `AtomicWriteFile`. The destination stays
  present until one committed replacement; failed commits discard staging, and
  publication has no post-commit backup cleanup that can report a false failure.
- Publication regressions cover creation, replacement, invalid input, an
  injected commit failure, stale legacy candidates, an unremovable legacy
  backup, and Unix executable permissions. Restore-failure coverage is no
  longer applicable because publication never renames the destination away.
- BUNDLE-04 makes bundle decoding return the validated `ProgramImage`; the
  native runner consumes it directly instead of decoding the embedded image a
  second time. This is a simplification fix, not a measured performance claim.
- Format regressions cover unsupported versions, reserved bytes, invalid UTF-8,
  corrupt program images, invalid lengths, UTF-8 byte-length boundaries, and an
  exact V1 golden-byte fixture.
- User documentation updated:
  `docs/pascal/program-structure/cli.md` and
  `docs/pascal/program-structure/projects.md`.

## Verification

- `cargo fmt --all -- --check` — passed.
- `cargo test -p fpas-bundle --locked` — passed: 15 tests plus doc tests.
- `cargo check -p fpas-bundle --all-targets --locked` — passed.
- `cargo check -p fpas-cli --bin fpas-runner --locked` — passed.
- `cargo clippy -p fpas-bundle --all-targets --locked -- -D warnings` — passed.
- `cargo clippy -p fpas-cli --bin fpas-runner --locked -- -D warnings` — passed.
- `cargo test -p fpas-cli --test native_executable --locked` — passed: 2 tests.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.

A broader `cargo clippy -p fpas-bundle -p fpas-cli --all-targets --locked -- -D warnings`
also checked the unrelated `fpas` binary and stopped on the pre-existing
`clippy::question_mark` finding in
`crates/fpas-cli/src/cli_fmt/paths.rs`. That separate review backlog item was
not changed as part of this crate slice.

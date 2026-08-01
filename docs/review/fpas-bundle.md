# `fpas-bundle` review follow-up

Classification: executable packaging and publication. No language change expected.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| BUNDLE-01 | P1 | `crates/fpas-bundle/src/publication.rs:37-46` | Windows publication renames the current executable away before installing the replacement. A crash leaves the application path missing; a failed restoration error is discarded. | Prefer an OS replacement primitive or a transaction with explicit rollback and surfaced restoration failure. | Existing destination, injected install failure, injected restore failure, and crash-safe state assertions. |
| BUNDLE-02 | P2 | `crates/fpas-bundle/src/publication.rs:48-50` | Backup cleanup failure reports the whole publication as failed even though the new executable is installed. | Distinguish committed publication from cleanup warning, or make post-commit cleanup best-effort with an explicit diagnostic. | Simulate backup deletion failure and assert the reported state matches the installed executable. |
| BUNDLE-03 | P3 | `crates/fpas-bundle/src/publication.rs:15-19,72-75` | Temporary names are only process-locally unique; stale files plus PID reuse can block a later invocation. | Use collision-resistant identifiers or retry `create_new` on `AlreadyExists`. | Precreate a stale candidate and verify retry and cleanup. |
| BUNDLE-04 | P3 | `crates/fpas-bundle/src/format.rs:135-136` | Bundle decoding fully parses the program and the runner parses the same image again. The duplicate cost is confirmed, but impact is unmeasured. | Return the validated `ProgramImage` from bundle decoding or expose a single validate-and-decode API. | Assert one decode path and preserve all corrupt-image diagnostics; benchmark before claiming speedup. |

## Implementation notes

Publication currently has no crate-specific tests. Add temporary-directory coverage for new files, replacement, cleanup, Unix executable bits, Windows rollback, unsupported versions, reserved bytes, invalid UTF-8, corrupt images, and name-length boundaries. Add a golden-byte format fixture so symmetric encoder/decoder changes cannot silently break compatibility.

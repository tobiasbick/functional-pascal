# Correctness and test gaps

Paths and line numbers refer to the commit in [the overview](README.md).

## C1. Project root characters are interpreted as glob syntax

Priority P1. Reproduced on Windows.

`crates/fpas-project/src/path_glob.rs:45` joins the literal root with the requested pattern and passes the entire string to `glob`. The root is therefore interpreted as pattern syntax too. `crates/fpas-project/src/paths.rs:86` uses this helper for project source selection.

For a root named `demo[1]` and pattern `*.fpas`, expansion returns no files when the root exists alone. If sibling `demo1` exists, it returns that sibling's files. This can produce either an erroneous no-match diagnostic or compile a different source set. Square brackets are valid directory-name characters on Windows and Unix.

Keep the root literal and interpret only the supplied pattern. Escaping must preserve path-prefix and absolute-pattern handling. Do not simply escape the whole joined string, which would disable intentional wildcard matching.

Add a public project-loading regression with `sources.include = ["*.fpas"]` below a bracket-containing directory and a decoy sibling. Assert the exact selected source paths. Cover exclusions and unmatched brackets too. Existing normal include/exclude tests and the non-UTF-8-root test at `crates/fpas-project/src/paths.rs:308` do not exercise this case.

The separate non-UTF-8 fallback in `path_glob.rs:60` also traverses every descendant directory even for a one-level pattern. Test parity with the ordinary expansion path before reusing that walker as the fix: missing literal prefixes, nested unreadable directories, and directory symlinks are relevant cases. That parity investigation was not executed on Unix here.

## C2. Windows sidecar replacement is not one atomic publication

Priority P2. Confirmed by control flow, not crash injection.

`crates/fpas-unit/src/sidecar/atomic.rs:161` moves an existing destination to a backup and only then renames the temporary file to the destination. A process exit between those operations leaves the authoritative sidecar path absent. A failed second rename attempts to restore the backup but discards the restore error. A failed backup removal returns an error even though the new sidecar is already installed.

Cooperating readers normally acquire the shared lock in `sidecar/mod.rs:104`, so ordinary coordinated reads are protected during a successful write. The lock does not make the two renames a crash-atomic replacement. Sources remain authoritative, so a missing sidecar can be rebuilt; this is not source-code loss. The effect is a weaker persistence guarantee, possible leftover backups, and ambiguous publication failures.

The same repository already uses `AtomicWriteFile` for program images and bundles. Reuse the supported replacement mechanism while retaining unit-specific coordination and validation. Do not remove those boundary checks merely to reduce code.

Add deterministic failure injection for staging, commit, and cleanup. Assert that a failed pre-commit operation leaves the previous destination readable, that successful commit has a clear result, and that failures do not silently strand the previous image under a backup name. Existing lock-lifetime tests in this module do not cover failed publication or rollback. See also D1 and S1.

## C3. The Unix trailing-separator test has the wrong expected result

Priority P2. Source-confirmed test defect; Unix execution not performed.

`crates/fpas-std/src/path.rs:181` defines `base_name_returns_empty_for_trailing_separator_on_unix`. Its Unix branch expects an empty string for `dir/nested/`. The implementation at line 55 delegates to `Path::file_name`, which returns the final normal component, `nested`; the trailing separator does not erase that component. The neighboring Windows test expects `nested`.

On Windows the Unix test function still reports success because its body is removed by `cfg(unix)`. The local green result would therefore give no evidence about its assertion. The public page `docs/pascal/std/host/path.md:42` correctly delegates to Rust path parsing.

Correct the assertion, make this shared case platform-independent, and place platform attributes on whole tests when behavior truly differs. Verify `BaseName` with trailing separators, a root, and an empty input on Unix and Windows. Preserve the current documented behavior.

## C4. Small malformed sections can request large allocations

Priority P2. Source-confirmed; peak memory not profiled.

`crates/fpas-program/src/format/executable/decode.rs:285` checks the instruction count against `MAX_INSTRUCTIONS`, then reserves that many entries before reading the first instruction. A section with zero bytes and an item count of 16,000,000 requests 128,000,000 bytes of instruction capacity before returning a truncation error. `decode_strings` and `decode_source_runs` follow the same count-first reservation pattern. Counts are bounded, but they are not first bounded by the bytes actually present.

The section directory at `crates/fpas-program/src/format/sections.rs:119` validates byte ranges separately from item counts. A matching payload digest does not establish semantic consistency between those fields. The issue requires malformed input with an internally updated digest, not an ordinary valid program.

Check fixed-width section size against count before allocating. For variable-width sections, use a minimum encoded entry size and grow only as validated entries arrive, with the existing cumulative limits retained. Where allocation remains fallible, consider returning a format/resource error instead of relying on an infallible reservation.

Existing tests in `crates/fpas-program/tests/format.rs` cover truncation, malformed directories, digest mismatches, and deterministic mutations. Add explicit maximum-legal-count/minimal-payload fixtures with refreshed digests. An allocation-aware check should verify rejection before a large reservation; a simple `is_err()` assertion would pass with the current costly implementation.

## C5. Windows benchmark descendant cleanup failed validation

Priority P2. Observed failure; environment versus implementation cause remains unresolved.

The first workspace run failed `suite::runner::tests::timeout_terminates_descendants_without_waiting_for_inherited_pipes` in `fpas-bench`, with `timeout waited 2.0004827s for an inherited pipe` at `crates/fpas-bench/src/suite/runner.rs:346`. The target reported 34 passing tests and one failure. The complete follow-up run reproduced the failure with a 2.0031875-second wait.

`crates/fpas-bench/src/suite/runner/windows.rs:14` invokes `taskkill /T /F`, falls back to killing only the direct child on error, and waits for that child. In `runner.rs:140`, output draining uses `?` before termination errors are appended to the diagnostic. If a descendant keeps a pipe open past the drain timeout, the resulting pipe error can hide the useful tree-termination error. Reader threads are detached on that timeout.

The failure alone does not prove that `taskkill` is universally broken. The fixture's grandchild naturally exits after two seconds, which is consistent with the observed wait, but does not identify why tree termination failed.

Preserve termination and drain diagnostics together. Add an injected failed-tree-termination case with a descendant that retains a pipe, and a case where the direct child exits before its descendants. Verify both prompt return and descendant cleanup. Consider OS process-tree ownership only after the observed failure has been isolated. The existing descendant test is valuable and should not be weakened to accept the two-second delay.

## C6. Loose-source discovery depends on ancestor permissions

Priority P2. Reproduced in the complete suite and a targeted repeat.

`crates/fpas-language-service/tests/workspace_context.rs:102` creates a source without local metadata and expects `WorkspaceKind::Loose`. Both runs returned `Unavailable`. A separate call to `WorkspaceContext::load` for a fresh temporary source confirmed one access-denied issue and no selected manifest.

`crates/fpas-language-service/src/workspace/discovery.rs:148` searches ancestors until the filesystem root. Any workspace-discovery error becomes `Unavailable` through `discover_initial_context`, even though the source itself is readable. Consequently the test also assumes every ambient ancestor can be inspected and contains no interfering manifests.

The observed permission error is environment-dependent. It is not evidence that an ordinary manifest failed to parse. Specify a discovery boundary for loose sources, or deliberately retain strict error reporting and make the test's discovery inputs hermetic. Add a controllable ancestor-discovery seam covering an unreadable ancestor, unrelated manifests, and a real nearest owning project. Do not suppress errors for explicitly requested project manifests.

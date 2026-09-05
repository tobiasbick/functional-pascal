# Review fixes and verification

This follow-up addresses R01–R03, F01–F03, and P01–P06 from the original review.
Changes are compiler/runtime bug fixes, standard-library implementation fixes,
Notes tooling behavior, and bounded performance work. Existing language contracts
are retained; no language specification page is changed.

## Correctness

| ID | Implemented correction | Regression evidence |
| --- | --- | --- |
| R01 | Discover closures and bound methods in `repeat` conditions. | Anonymous/mutable-capture and bound-method conditions failed before the correction and pass afterward. |
| R02 | Reject non-finite results and the exclusive positive 2^63 bound in real-to-integer rounding. | All four rounding operations test both limits, the adjacent representable values, infinities, and NaN; the upper-bound test failed before the fix. |
| R03 | Share bounded, cancellation-aware TLS handshake I/O between client and server. | A peer that receives ClientHello but never replies no longer delays shutdown until the connection timeout. The regression failed before the fix; TLS hostname, trust, success, and networking tests pass. |
| F01 | Accept caret/viewport proposals without marking unchanged persisted content dirty. | Clean title/body navigation, unchanged tags, timestamp preservation, and already-dirty state are covered; the clean-navigation case failed before the fix. |
| F02 | At the wire-byte limit, probe for EOF without appending another byte. | Empty and nonempty close-delimited responses at limit-1, limit, and limit+1 through buffered and streaming APIs; the exact-boundary case failed before the fix. |
| F03 | Finalize pending SSE lines/events without synthetic input bytes. | Final data with no delimiter, CR, or LF at exact limits; oversize input and feed-after-finish errors. The unterminated exact-limit case failed before the fix. |

TLS tests use local peers and independent cleanup. Network fixtures explicitly
restore blocking mode on accepted Windows sockets. Production connection setup
retains its existing bounded TCP establishment; TLS handshake cancellation is now
observed during short I/O waits.

## Performance changes

| ID | Implemented change | Scope and evidence |
| --- | --- | --- |
| P01 | Split text once for caret lookup and once per paint; reuse the visibility lookup. | Unicode-scalar indexes and terminal display-width calculations are retained. Public headless Unicode, empty-line, trailing-newline, keyboard, pointer, and snapshot tests cover behavior. |
| P02 | Borrow dictionary storage for reads, cloning only returned components. | Length no longer copies entries. Tests retain lookup results, missing keys, insertion order, and managed payloads. |
| P03 | Add a consuming `ArrayPop` instruction for direct locals. | Unique storage is reused; shared arrays detach. Compiler tests cover aliases, globals, and captures; VM tests retain source registers on invalid/empty input. IR and bytecode validate operands. Bytecode version 15 invalidates older artifacts. |
| P04 | Collect body fragments and flatten once for buffered client and server. | Ordered byte materialization replaces repeated prefix concatenation. HTTP boundary and network integration tests guard framing, errors, and limits. |
| P05 | Start equality with the current value pair and an empty worklist. | Scalar equality does not allocate the worklist. Deep traversal remains iterative; NaN, signed zero, nested values, and dictionary ordering are tested. |
| P06 | Validate a field store, then take the destination before mutation. | A storage-address regression failed before the fix and passes afterward. Existing shared-record isolation also passes. No ordinary source producer was established, so no application speedup is attributed to this opcode. |

Global and captured array pop retain the existing general lowering. The optimized
path covers the directly stored local arrays identified by P03 without changing
capture/global ownership rules. No new dictionary representation, cross-frame text
cache, heap design, or native backend is introduced.

## Measurements

Baselines were saved with `cargo bench-fpas`: `review-initial` for the original
32-entry suite, `review-before --group review` for the five added workloads, and
`review-full-before` for all 37 entries before performance implementation.
Failed benchmark setup runs were discarded. The new fixtures check their outputs;
HTTP elapsed time includes loopback transport and is not an allocation-only measure.

The completed 37-entry comparison against `review-full-before` measured:

| Workload | Before (ms) | After (ms) | Change |
| --- | ---: | ---: | ---: |
| Dictionary reads | 262 | 9 | -96.6% |
| Scalar membership | 350 | 28 | -92.0% |
| Local array drain | 678 | 5 | -99.3% |
| Long Unicode text-area frames | 2,752 | 2,230 | -19.0% |
| Buffered HTTP | 248 | 201 | -19.0% |
| Notes frames | 12,544 | 11,536 | -8.0% |
| General TUI frames | 4,916 | 4,894 | -0.4% |

Every row was inspected. The largest increases were task spawn/wait (+9.2%) and
analysis queries (+8.2%); no row exceeded +10% in this comparison. These are single
wall-time observations, not confidence intervals. Small HTTP differences varied
between runs, and transport coalescing is not controlled by socket write sizes.
The implementation removes repeated accumulated-prefix copying; this experiment
does not establish an allocation count or a pure body-processing speedup.

The initial 32-entry run must not be confused with the later performance baseline:
it recorded Notes at 8,933 ms, while the corrected run above measured 11,536 ms.
This prompted a separate investigation instead of silently choosing the more
favorable baseline. Repeating the unmodified original commit in an isolated checkout
measured Notes at 11,455 ms and general TUI at 4,965 ms. A nearby corrected run was
11,665 ms and 4,969 ms. Replacing only the Notes update unit with its original version
on the corrected runtime was slower (12,181 ms). The initial Notes timing was not
reproducible, and the controlled repeat did not establish a material Notes regression
or an end-to-end Notes speedup. All experimental source replacements were restored.

The final full-suite recording is saved in [benchmark history](../../bench/history.md).
It provides a second after-state measurement alongside the comparison above.
Dictionary reads remained 9 ms, scalar membership 27 ms, and local array drain 5 ms.
Task spawn/wait returned to 503 ms and analysis queries to 247 ms; the earlier
increases did not persist. No recorded row is more than 3% above the full baseline.

## Documentation and validation

Updated current docs: numeric rounding bounds, HTTP/SSE limits, local array mutation
storage behavior, Notes dirty-state behavior, and benchmark workload guidance.
Repeat scope and network cancellation were already documented correctly; those
contracts remain unchanged.

Validation completed:

- `cargo fmt --all` and the changed FPAS files' `fmt --check` pass.
- `cargo build` passes.
- `cargo test --workspace --no-fail-fast`: 2,854 passed, none failed or ignored.
- Direct release `fpas test tests/`: 403 passed, one intentional skip, zero failed
  or not run. This includes Notes and network tests beyond the Cargo theme wrappers.
- All five review-report files' relative links resolve.

Expected integration fixture changes accompany the implementation: the new benchmark
group appears in CLI help assertions, bytecode-version-dependent program/bundle
digests are refreshed, and the LSP reference test includes the five additional
Notes regression call sites. The local commit includes the Codex co-author trailer;
no push is performed.

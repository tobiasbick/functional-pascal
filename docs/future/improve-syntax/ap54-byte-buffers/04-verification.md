# AP54.4: Verify lifecycle, integration and representation costs

Package: [AP54](README.md). Status: Complete. Lifecycle/integration regressions,
correctness verification, representation measurements and review are delivered.

## Contract

[AP54.1](01-contract.md) fixes the observable API, ownership, capacity, UTF-8,
artifact and debugger rules. [AP54.2](02-native-storage.md) and
[AP54.3](03-utf8-and-migration.md) deliver their implementation and primary tests.
This part adds distinct cross-boundary evidence without new APIs, syntax,
retention pools or unrelated optimizations. Buffers have no `Close` operation.

## Delivered coverage and owners

| Boundary | Evidence and owning files |
| --- | --- |
| Actual backing and capacity | `crates/fpas-build/tests/byte_buffer_memory.rs` counts native allocation bytes and allocations, including metadata, shared owners, copy-on-write detachment, proper subranges, clear and explicit trim. Shared backing counts once, a tiny slice does not retain its parent, and the final owner returns allocation counts exactly to baseline. |
| Nested messages, cycles and cancellation | `tests/stdlib/bytes/lifetime_test.fpas` retains buffers in records, enums, arrays, closure/cell cycles, queued messages and task results. Its build memory owner repeats normal and failing VM teardown under default and 1/1/1 collection schedules. |
| Pending sender ownership | The same memory owner uses public `DebugSession` task holds to prevent parent scope exit until the second sender is provably waiting on a full channel. It cancels that sender, resumes the parent and checks exact reclamation. The first nested message and captured callable remain intact; the cancelled sender publishes no message. There is no sleep race or production test hook. |
| Native file jobs | `crates/fpas-runtime/src/execution/vm/hosted/fs/fixtures/pool.fpas` retains native byte backing in cyclic callbacks. `crates/fpas-runtime/tests/gc_memory/files.rs` owns exact cross-thread allocation checks for completion, cancellation, scope exit and shutdown. Existing file-wait tests own admission, queued I/O and undelivered-result retirement. |
| Process cases and callbacks | `crates/fpas-runtime/src/execution/vm/hosted/proc/fixtures/selection.fpas` retains byte-backed callback cycles. `crates/fpas-runtime/tests/gc_memory/processes.rs` measures completion, losing cases, timeout, cancellation, scope exit and shutdown, including native pipe payloads and mutated caller inputs. |
| Complete binary/text flow | `tests/stdlib/net/byte_buffer_transfer_test.fpas` passes every byte value plus appended Unicode/NUL text through file I/O, a channel/task, bounded socket transfers and hex conversion. Raw malformed UTF-8 survives every handoff; strict decoding reports its byte offset, while the valid text suffix decodes exactly. EOF, input snapshots and partial progress remain covered. |
| Tools and artifacts | Existing `crates/fpas-debug/tests/byte_buffers.rs`, `crates/fpas-build/tests/program_artifact.rs` and runtime host-value tests retain their primary proof for complete values beyond previews, detached mutation, bounded event capture, cross-session exports and compiled-unit/program-image reuse. General snapshots and replay remain separate. |
| Portable validation fixture | `crates/fpas-cli/src/main_tests/tui/subviews.rs` normalizes embedded CRLF before splitting the demo source. Its existing owner test exposes the Windows failure and verifies focus, confirmation and quit after repair. |

The native lifecycle owners retain the same GC/allocation infrastructure and
known baselines. Thread-local runtime pools are destroyed before teardown
comparisons; unrelated bounded initialization is warmed only where its existing
owner requires it. Wrapper counts and process RSS are not used as substitutes
for actual native backing reclamation.

Existing storage/operation and UTF-8 tests remain the primary owners for growth,
zero initialization, limits, invalid ranges, atomic failure, injected allocation
failure, equality, dictionary keys, protected iteration and first-class values.
AP54.4 does not duplicate those cases at every crossed layer.

## Representation evidence

The comparison baseline is `295726a77`, immediately before native byte storage
and API migration. The native implementation is `1b33e47fe`; AP54.4 changes tests
and documentation without changing production representation code.

Reproducible sources, workload arguments, raw results, environment/toolchain
metadata, profiles and comparison reports belong under
`.temp-data/ap54-byte-buffers/`. They are not entries in `history.md` or tracked
timing reports. The built-in FPAS profiler precedes controlled measurements.

Equivalent observable work covers native file transfers, append/growth,
shared copy/mutate/slice, ASCII and multibyte UTF-8 encode/decode/text append,
Base64/hex, fragmented SSE and buffered HTTP. A small existing VM subset checks
unrelated representation costs. Both representations use the workspace release
profile with Fat-LTO and default Cargo build parallelism. Compilation and
program execution are measured separately; builds and tests do not overlap
timed workloads.

Three complete runs per representation retain raw observations and their
spread. Each toolchain is checked against its intended API before timing;
workspace-local release artifacts are invalidated when switching checkouts.
The comparison reports workload-specific costs, including unchanged VM controls
and unresolved timings near the timer's resolution, rather than a package-wide
speed claim. Checkout isolation and source-only build inputs follow the
[build guide](../../../bench/build-times.md#checkout-isolation-and-library-inputs).

Backing/capacity accounting accompanies runtime measurements. Elapsed time
does not prove physical zero-copy or allocator release to the operating system.
Repeated runs retain their spread; results near timer resolution stay
unresolved. Correctness and bounded/reclaimed storage are the acceptance gates,
without a timing threshold or a promised speedup.

## Verification and review

The workspace `dev-fast` program build, separate guarded Rust compilation and
execution, full FPAS suite, affected app/example checks, all-target/all-feature
Clippy with warnings denied, formatting, generated declarations, executable
documentation, agent-reference budget and editor contract/grammar checks pass.
Rust cases run serially with Cargo's default build parallelism. The Windows
CRLF fixture failure is repaired at its owner and its sibling tests pass.

Spec and Standards reviews compare the working tree with `1b33e47fe`. The
confirmed sender synchronization finding is repaired and re-reviewed without
remaining actionable findings. Exact commands, initial failures and successful
rechecks are retained with the raw evidence.

This part executes on Windows. Linux and macOS are not rerun here; AP54.3's
Linux evidence does not substitute for those platform runs of the added cases.
Specialized media buffers, packed arrays and binary-layout helpers remain
separate proposals.

AP54.4 and the central package row are complete. Commit, push and merge require
separate user authorization.

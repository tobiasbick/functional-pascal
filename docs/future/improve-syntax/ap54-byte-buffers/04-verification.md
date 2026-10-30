# AP54.4: Verify lifecycle, integration and representation costs

Package: [AP54](README.md). Status: not started; AP54.2 and AP54.3 are complete.
Combined lifecycle/integration and representation-cost evidence remain in this part.

## Prerequisites

- [AP54.2](02-native-storage.md) and [AP54.3](03-utf8-and-migration.md) are delivered,
  including their operation, ownership, migration tests and current documentation.
- [AP54.1](01-contract.md) completes Q73–Q76 and fixes observable expected
  outcomes. Use those rather than assumptions about the chosen Rust container;
  automatic release must not depend on a buffer `Close`.

## Deliverable

Close distinct cross-boundary test gaps and verify the complete package
acceptance criterion. Produce reproducible storage/copy and runtime evidence,
without promising a speedup or using timing thresholds as correctness tests.
This part does not authorize additional buffer APIs or unrelated optimizations.

## Lifecycle and memory coverage

| Scenario | Required evidence |
| --- | --- |
| Copy, slice and mutate | Agreed value semantics, no unexpected alias writes, bounded retention and release after the last owner |
| Growth, clear and resize | Initialized bytes, checked limits, correct retained-capacity accounting and the documented capacity-release behavior |
| Nested aggregates and cycles | Buffers in records, enums, containers and closure/cell cycles release their actual backing allocation when unreachable |
| Failed operations | Rejected size/range/UTF-8 operations leave the agreed observable state and leak no temporary storage |
| Tasks and channels | Copies, queued messages, task results, channel closure, failure and cancellation retain/release precisely the legitimate owners |
| Native jobs and selection | Queued/running/completed-but-undelivered jobs, dropped cases, callbacks and shutdown neither publish stale bytes nor retain abandoned storage |
| VM and host boundaries | Session/domain transfers, repeated VM teardown and post-failure cleanup preserve the approved ownership rules |
| Tools and artifacts | Separately compiled consumers, intrinsic declarations, first-class values, live debug inspection, retained values, detached calls and bounded event capture support buffers; only the preview is truncated, and general snapshots/replay remain separate |

Use the existing GC/allocation infrastructure, extended to see native backing
bytes and capacity. Warm up unrelated bounded runtime pools, compare against a
known baseline and count shared backing once. There is no new buffer retention
pool: require backing allocations to return to baseline after legitimate owners
are gone and ensure repeated cycles do not accumulate live storage.
Wrapper counts alone do not establish native memory release, and allocator RSS
retention alone does not establish a leak. Tests should force the relevant
ownership/failure path with synchronization rather than relying on sleep races.

## Measurements

Before changing representation, preserve a usable baseline and its revision/
profile/workload metadata. Use equivalent observable work before and after:

- Native file or loopback/process transfer of bounded binary chunks.
- Repeated append/copy/slice and growth with both unique and shared storage.
- UTF-8 encode/decode/text append for ASCII and multibyte text.
- Base64/hex and HTTP/SSE chunk handling where these expose conversion costs.
- A small existing VM benchmark subset to detect unrelated representation costs.

Measure backing bytes/capacity and ownership/copy behavior in addition to VM
program runtime. Do not claim physical zero-copy from elapsed time alone. Keep
compiler/build time separate from FPAS program execution; use the repository's
Fat-LTO benchmark profile consistently on both sides. Profile FPAS execution
with the built-in profiler before adding external instrumentation.

Raw results, environment/profile metadata and comparisons belong under
`.temp-data/ap54-byte-buffers/`, not `history.md` or tracked timing reports.
Compare like-for-like runs in the same environment; cloud measurements do not
establish desktop performance. The acceptance gate is correct behavior and
bounded/reclaimed storage, not a chosen percentage improvement.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify as needed | `crates/fpas-runtime/src/value/` and owning VM lifecycle tests | Distinct backing-allocation, transfer, cancellation and collection gaps |
| Modify as needed | `crates/fpas-build/tests/`, `tests/stdlib/` and native-job test owners | End-to-end memory and behavioral regressions missing from earlier parts |
| Modify as needed | Buffer and migrated API handbook pages, generated declarations, agent guidance | Correct documented discrepancies found by final acceptance |
| Modify | This file and `README.md` | Describe delivered coverage and actual follow-ups; mark only verified work complete |
| Modify | `../README.md`, `../../fpas2-language-directions.md`, `../../std-roadmap.md` | Package status and compact delivered byte-buffer status, without claiming all packed/media work is complete |

Before coding, list only the concrete remaining coverage gaps and their owning
files. Do not add duplicate tests that merely repeat AP54.2/AP54.3.

## Acceptance and verification

The complete package acceptance in the README passes. Run the workspace
development build, separate Rust test compilation/execution, full FPAS suite,
affected app/example checks, strict all-target/all-feature Clippy, format checks,
generated declarations, executable documentation and relevant editor tests.
Follow the repository's build/resource-guard workflow. Investigate and fix
encountered failures; do not classify them away as pre-existing.

Review against the approved contract and repository standards, fix confirmed
findings and rerun the relevant checks. Preserve any untested platform coverage
in the delivery report. Mark AP54.4 and the central package row complete only
after the required evidence is available. Update the FPAS2 plan for this selected
byte scope; specialized media, packed arrays and binary-layout helpers remain
separate proposals. Commit, push and merge only when the user authorizes them.

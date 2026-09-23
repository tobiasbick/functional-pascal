# Rust review resource regressions

These measurements close the targeted resource-evidence gaps in the
[2026-09-23 Rust review](../future/rust-code-review-2026-09-23/README.md).
They count actual operations and requested allocation bytes in Windows Rust tests.
They are not elapsed-time benchmarks and do not establish a general speedup.

## Fixtures and measurement boundaries

- **Source exclusions:** resolve 100 and 200 files with half excluded. Count calls
  at the actual canonicalization boundary, including include deduplication,
  exclusion deduplication, and filtering. Fixture writes are outside the counter.
- **Register allocation:** run the production allocator on generated IR. Vary
  fixed locals, result-producing instructions, and simultaneous live values
  independently. Count allocations only inside `Allocation::build`, and verify
  the resulting register count. The budget is proportional to IR input size.
- **Closed project sources:** open the main buffer and leave forty units closed.
  Warm semantic analysis first; count disk reads and UTF-8 bytes during each
  diagnostic request. Also check two simultaneous query forks, direct disk edits
  without watcher events, explicit invalidation, and open-buffer precedence.
- **Warm unit builds:** build a library with 32 units and 496 direct imports, then
  reuse its sidecars. Count actual direct-interface copies for compilation;
  decoding and final interface ownership are deliberately outside that counter.
- **Successful parsing:** compare six Parse/Conv intrinsics on short inputs and
  equivalent inputs padded with 128 KiB of whitespace. Argument construction is
  outside allocation counting and the borrowed argument stays alive throughout
  the measurement. Compare results, allocation counts, and allocated bytes.
- **Malformed program images:** refresh payload digests after increasing table
  counts. Require the expected format error and less than 64 KiB of allocations
  over the entire decode operation, including the header and preceding sections.

Allocation counting uses `allocation-counter` as a development-only dependency;
no unsafe code is added to the workspace. Its counters observe the current
thread. The other counters exist only under `cfg(test)`. No counters or allocator
overrides are added to production binaries.

## Measured comparisons

| Case | Previous path | Corrected path |
| --- | --- | --- |
| 100 sources, 50 exclusions | 4,025 canonicalizations | 250 canonicalizations |
| 200 sources, 100 exclusions | Not rerun after the smaller baseline assertion failed | 500 canonicalizations |
| Register allocation: 16 locals, 512 values, one live temporary | 3,328 allocations / 165,552 bytes | 1,281 allocations / 38,616 bytes |
| One diagnostic request, forty closed units | 80 reads / 780 bytes | 40 reads / 390 bytes |
| Warm library build, 496 imports | 496 direct interface copies | 0 direct interface copies |
| `TryInt`, short / padded text | 28 / 131,100 allocated bytes | 26 / 26 allocated bytes |
| Malformed instruction table claiming 16 million entries | 128,003,052 allocated bytes | 3,052 allocated bytes |

The old exclusion, allocation, parsing, and build paths were reinstated from the
review base or equivalent local reversions while keeping the new test probes.
For the instruction decoder only the instruction-count precheck was removed;
other decoder checks remained enabled. These are focused comparisons, not two
complete executable benchmark baselines.

The corrected allocator measurements vary one dimension at a time:

| Fixed locals | Produced values | Live temporaries | Allocations | Allocated bytes |
| --- | --- | --- | --- | --- |
| 16 | 512 | 1 | 1,281 | 38,616 |
| 512 | 512 | 1 | 1,362 | 46,056 |
| 1,024 | 512 | 1 | 1,449 | 54,360 |
| 512 | 1,024 | 1 | 2,647 | 86,200 |
| 512 | 512 | 64 | 1,438 | 51,624 |
| 512 | 512 | 128 | 1,447 | 54,376 |

These counts do not measure the CPU cost of scanning live values. They demonstrate
that fixed locals are no longer reinserted into a newly allocated occupancy tree
for every result. The smallest old-path fixture already exceeds the allocation
budget; larger old-path cases were not needed for the negative regression run.

## Regression sensitivity

Nine controlled negative runs all failed their intended assertions:

1. Restore unbounded `wait()` after failed termination: both process-failure tests
   reject the blocking call while their real child fixtures are still alive.
2. Restore catch-all ancestor-error suppression: invalid manifests, missing
   dependencies, and ambiguous owners incorrectly become loose contexts.
3. Remove the instruction-byte precheck: the allocation budget detects the large
   reservation even though the final format error remains unchanged.
4. Restore nested exclusion canonicalization: the linear operation budget fails.
5. Restore per-instruction occupancy-tree rebuilding: the IR allocation budget fails.
6. Restore duplicate diagnostic snapshot collection: the read/byte budget doubles.
7. Restore owned parse arguments: padding increases successful-parse allocation bytes.
8. Restore eager direct-interface copies: the warm-build counter becomes 496.
9. Bound an unavailable context at the source directory instead of the failing
   ancestor: later diagnostic requests lose the manifest or ownership error.

All source files were restored after those controlled negative runs. Commands for
normal resource verification are listed in the [benchmark guide](README.md#rust-review-resource-regressions).

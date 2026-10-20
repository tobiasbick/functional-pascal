# Benchmark history

Committed snapshots from `cargo bench-fpas record`. Absolute times are machine-specific; use them
to track relative progress on the same machine and to see which changes moved which benches.

Do **not** record hostnames, usernames, paths, or other machine-identifying metadata.

Update after a meaningful performance change:

```sh
cargo bench-fpas record "short note about the change"
cargo bench-fpas record "vm-only note" --group vm
```

Newest entries are prepended below this header.


## 2026-10-20 — AP47.5 typed host and parsing errors

Compared the unchanged pre-migration source archive (`a63232a0f`) with the final
AP47.5 build using all 72 original workloads in three serial before/after pairs.
The [complete comparison](ap47-host-parsing.md) records every row, raw timer
samples, boundaries and regression controls. Sums of row medians were
10728 → 10727 ms for VM, 2469 → 2317 ms for concurrency, 3425 → 3311 ms for
tooling and 2922 → 2875 ms for startup. Cloud scatter prevents a general speedup
claim; no VM row crossed the selected regression gate.

A reproducible scalar-membership slowdown in the first build was repaired with
an exact Integer array comparison, preserving mixed/structural equality. Its
final full-suite median is 58 → 10 ms (-82.76%). The initial callback increase
did not repeat. Final Mandelbrot outliers received isolated controls: render did
not repeat its increase; paint's +9.95% over eight pairs reversed to -11.11% over
twelve more, with all twenty yielding 1205.5 → 1189 ms (-1.37%). The report retains
both series rather than hiding their scatter. Dedicated error workloads remain
scheduled after AP47.6's original-suite comparison.

## 2026-10-20 — AP47.4 constructor values and MapError

Compared unchanged AP47.3 (`c3dee7ac7`) with AP47.4 in three paired serial runs
of the original VM and concurrency groups. The
[complete comparison](ap47-maperror.md) preserves every sample and the measurement
boundaries. Sums of row medians were 11611 → 11055 ms for VM and 2576 → 2103 ms
for concurrency. Large scatter prevents attributing those lower totals to an
optimization.

Two initial VM medians exceeded 5%. Eight isolated alternating pairs reproduced
neither increase: typed Boolean 351 → 343 ms (-2.28%), record field access
551 → 563 ms (+2.18%), with matching checksums. No repeatable VM regression
crossed the selected gate. Dedicated error workloads remain scheduled after Std
migration, preserving the original suite IDs and arguments.

## 2026-10-19 — AP47.3 native error language

Native error declarations, Error-constrained routines and Message() connect the
runtime metadata to source programs, cached units and ordinary callable values.
The unchanged suite is compared against AP47.2 (`7f0fd9f`) in three serial runs
per group. [The complete comparison](ap47-language.md) retains every raw sample.

Sums of row medians change from 6,500 to 6,551 ms for VM (+0.78%), 6,545 to
6,313 ms for tooling (-3.54%), and 3,162 to 3,131 ms for startup (-0.98%).
These are not whole-suite wall times. Alternating controls retain smaller
increases in integer loops (+2.46%) and string concatenation (+4.75%); every
VM row remains below the selected 5% gate. The configured codec rows are below
timer resolution; 50,000-decode controls measure 88/90 ms for unit objects and
67/66 ms for program images. No new feature benchmark group is registered here.

The following table is an additional current VM `record` capture.

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 718 | throughput: 69637883 iters/s |
| typed_real | 155 | - |
| typed_boolean | 240 | - |
| typed_string | 129 | - |
| global_access | 98 | throughput: 51020408 global updates/s |
| record_field_access | 332 | throughput: 45180722 field accesses/s |
| closure_call | 143 | throughput: 20979020 closure calls/s |
| mutable_capture | 180 | throughput: 16666666 captured updates/s |
| branch_dispatch | 342 | throughput: 58479532 branches/s |
| dynamic_numeric | 286 | throughput: 17482517 dynamic numeric ops/s |
| array_push | 54 | throughput: 37037037 pushes/s |
| array_length | 23 | throughput: 21739130 lengths/s |
| string_concat | 925 | throughput: 5405405 concats/s |
| string_length | 21 | throughput: 23809523 lengths/s |
| intrinsic_dispatch | 656 | throughput: 22865853 intrinsic calls/s |
| function_call | 260 | throughput: 23076923 calls/s |
| array_callbacks | 832 | throughput: 11538461 callbacks/s |
| record_update | 86 | throughput: 11627906 updates/s |
| unicode_char_at | 263 | throughput: 11406844 chars/s |
| wrapper_payload | 718 | throughput: 13927576 wrappers/s |
| string_search | 67 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |

## 2026-10-19 — AP47.2 native error metadata and portable artifacts

Native errors reuse shared enum values and essential portable metadata. Iterative
release starts at error-enum ownership boundaries, keeping ordinary aggregate
buffer release on its existing path. Source declarations and Std migration remain
planned; no feature benchmark group has been registered yet.

Three serial runs per selection keep the unchanged suite comparable. The sum of
23 VM row medians changes from 6,566 to 6,574 ms (+0.12%); the sum of all 72 row
medians changes from 39,048 to 38,920 ms (-0.33%). These are sums of row medians,
not whole-suite wall times.

The short array-push row has VM samples of 50/54/50 before and 50/53/54 ms after
(medians 50/53 ms, +6%). Its full-suite median stays at 53 ms. Interleaved fixed
old/current binaries measure 50/55/62 versus 53/52/63 ms (medians 55/53 ms).
The earlier broad aggregate traversal measured a 56 ms VM median; restricting
its entry to errors removes that ordinary-array work. No repeatable increase
above the 5% VM gate remains in the matched controls.

Edited-project analysis measures 2,990/3,021 ms in the full-suite comparison;
overlapping queries measure 616/651 ms. The larger initial editor outliers do
not reproduce in interleaved release harnesses built from the fixed pre-change
commit and current sources: edited-query medians are 2,974/2,963 ms, overlapping
cache hits 751/757 ms. Notes-frame medians change from 2,876 to 2,819 ms and
buffered HTTP from 186 to 175 ms in the final full-suite comparison.

The table below is an additional recorded current snapshot; comparison figures
above use the three-run medians and separate matched controls.

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 404 | throughput: enabled/disabled median change 0% |
| failure_dump_calls | 424 | throughput: enabled/disabled median change -4.347826086956516% |
| failure_dump_globals | 628 | throughput: enabled/disabled median change 2.941176470588232% |
| failure_dump_spawn_wait | 988 | throughput: enabled/disabled median change 5.769230769230774% |
| failure_dump_suspension | 9169 | throughput: enabled/disabled median change -0.38759689922480334% |
| failure_dump_failure | 287 | throughput: enabled/disabled median change 7.142857142857139% |
| failure_dump_large_report | 759 | throughput: enabled/disabled median change 385.7142857142857% |
| mandelbrot_render | 1164 | - |
| mandelbrot_paint | 1122 | - |
| integer_loop | 706 | throughput: 70821529 iters/s |
| typed_real | 153 | - |
| typed_boolean | 241 | - |
| typed_string | 130 | - |
| global_access | 105 | throughput: 47619047 global updates/s |
| record_field_access | 331 | throughput: 45317220 field accesses/s |
| closure_call | 144 | throughput: 20833333 closure calls/s |
| mutable_capture | 181 | throughput: 16574585 captured updates/s |
| branch_dispatch | 353 | throughput: 56657223 branches/s |
| dynamic_numeric | 301 | throughput: 16611295 dynamic numeric ops/s |
| array_push | 51 | throughput: 39215686 pushes/s |
| array_length | 23 | throughput: 21739130 lengths/s |
| string_concat | 894 | throughput: 5592841 concats/s |
| string_length | 23 | throughput: 21739130 lengths/s |
| intrinsic_dispatch | 657 | throughput: 22831050 intrinsic calls/s |
| function_call | 263 | throughput: 22813688 calls/s |
| array_callbacks | 843 | throughput: 11387900 callbacks/s |
| record_update | 86 | throughput: 11627906 updates/s |
| unicode_char_at | 265 | throughput: 11320754 chars/s |
| wrapper_payload | 735 | throughput: 13605442 wrappers/s |
| task_spawn_wait | 497 | throughput: 201207 tasks/s |
| task_array_callbacks | 674 | throughput: 5697329 callbacks/s |
| tui_headless | 1475 | throughput: 338 frames/s |
| notes_headless | 3002 | throughput: 83 frames/s |
| tui_styled_cells | 346 | - |
| analysis_queries | 225 | - |
| string_search | 69 | - |
| compiler_lowering | 1959 | - |
| substring_ascii | 6 | - |
| substring_unicode | 12 | - |
| project_queries | 975 | - |
| project_edits | 3614 | - |
| project_overlapping_queries | 597 | - |
| project_build_cold | 1901 | - |
| project_build_warm | 1254 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 3 | - |
| scalar_membership | 27 | - |
| array_pop | 1 | - |
| text_area_locate | 841 | - |
| http_body_accumulation | 172 | - |
| local_index_write | 16 | throughput: 40000000 writes/s |
| cell_grid_headless | 94 | throughput: 1063 paints/s |
| partial_surface_fill | 55 | - |
| source_utf8_ascii_2048 | 7 | - |
| source_utf8_ascii_4096 | 15 | - |
| source_utf8_ascii_8192 | 31 | - |
| source_utf8_unicode_2048 | 27 | - |
| source_utf8_unicode_4096 | 53 | - |
| source_utf8_unicode_8192 | 106 | - |
| source_notes_ascii_128 | 3 | - |
| source_notes_ascii_256 | 7 | - |
| source_notes_ascii_512 | 15 | - |
| source_queue_ascii_256 | 62 | - |
| source_queue_ascii_512 | 120 | - |
| source_queue_ascii_1024 | 243 | - |
| source_navigation_long_512 | 25 | - |
| source_navigation_long_1024 | 45 | - |
| source_navigation_long_2048 | 87 | - |
| source_navigation_lines_512 | 6 | - |
| source_navigation_lines_1024 | 10 | - |
| source_navigation_lines_2048 | 18 | - |

## 2026-10-19 — Keep inactive trace hooks out of VM instruction dispatch

Local tracepoints added instruction-boundary hooks to ordinary execution, even
without an installed trace. Dispatch now selects a const-specialized path once
per batch, keeping trace calls and interruption checks out of untraced execution.
The traced path still covers synchronous callbacks that inherit a trace without
being debugger-owned tasks.

Three complete-suite runs before and after the dispatch-only change reduced the
sum of the 23 VM row medians from 9,418 to 6,470 ms (-31.3%); the October 17
historical VM total was 6,484 ms. The integer-loop median fell from 1,422 to
683 ms (-52.0%). Alternating saved pre-fix and fixed binaries independently
measured 1,406–1,433 versus 693–719 ms with identical results. The existing
integer-loop guard at 1,000 ms failed before the fix and passed afterwards on
this machine; that threshold is not a portable CI requirement.

The three-run TUI total improved by 22.8%. Mandelbrot changed by -1.7%, tooling
by +1.8%, and startup by +2.9%; no individual row median regressed by more than
5%. These group totals sum row medians, not whole-suite wall time. The table
below is a separate VM recording run, rather than those three-run medians.

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 686 | throughput: 72886297 iters/s |
| typed_real | 153 | - |
| typed_boolean | 243 | - |
| typed_string | 123 | - |
| global_access | 98 | throughput: 51020408 global updates/s |
| record_field_access | 332 | throughput: 45180722 field accesses/s |
| closure_call | 144 | throughput: 20833333 closure calls/s |
| mutable_capture | 179 | throughput: 16759776 captured updates/s |
| branch_dispatch | 342 | throughput: 58479532 branches/s |
| dynamic_numeric | 292 | throughput: 17123287 dynamic numeric ops/s |
| array_push | 53 | throughput: 37735849 pushes/s |
| array_length | 23 | throughput: 21739130 lengths/s |
| string_concat | 882 | throughput: 5668934 concats/s |
| string_length | 21 | throughput: 23809523 lengths/s |
| intrinsic_dispatch | 650 | throughput: 23076923 intrinsic calls/s |
| function_call | 258 | throughput: 23255813 calls/s |
| array_callbacks | 832 | throughput: 11538461 callbacks/s |
| record_update | 86 | throughput: 11627906 updates/s |
| unicode_char_at | 269 | throughput: 11152416 chars/s |
| wrapper_payload | 719 | throughput: 13908205 wrappers/s |
| string_search | 69 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |

## 2026-10-17 — Borrowed closure and callback function values

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 709 | throughput: 70521861 iters/s |
| typed_real | 154 | - |
| typed_boolean | 243 | - |
| typed_string | 121 | - |
| global_access | 101 | throughput: 49504950 global updates/s |
| record_field_access | 327 | throughput: 45871559 field accesses/s |
| closure_call | 144 | throughput: 20833333 closure calls/s |
| mutable_capture | 189 | throughput: 15873015 captured updates/s |
| branch_dispatch | 349 | throughput: 57306590 branches/s |
| dynamic_numeric | 297 | throughput: 16835016 dynamic numeric ops/s |
| array_push | 55 | throughput: 36363636 pushes/s |
| array_length | 22 | throughput: 22727272 lengths/s |
| string_concat | 911 | throughput: 5488474 concats/s |
| string_length | 21 | throughput: 23809523 lengths/s |
| intrinsic_dispatch | 641 | throughput: 23400936 intrinsic calls/s |
| function_call | 259 | throughput: 23166023 calls/s |
| array_callbacks | 809 | throughput: 11866501 callbacks/s |
| record_update | 91 | throughput: 10989010 updates/s |
| unicode_char_at | 271 | throughput: 11070110 chars/s |
| wrapper_payload | 683 | throughput: 14641288 wrappers/s |
| string_search | 70 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |

## 2026-10-15 — In-place integer and Boolean register stores

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 706 | throughput: 70821529 iters/s |
| typed_real | 168 | - |
| typed_boolean | 273 | - |
| typed_string | 141 | - |
| global_access | 103 | throughput: 48543689 global updates/s |
| record_field_access | 325 | throughput: 46153846 field accesses/s |
| closure_call | 157 | throughput: 19108280 closure calls/s |
| mutable_capture | 200 | throughput: 15000000 captured updates/s |
| branch_dispatch | 357 | throughput: 56022408 branches/s |
| dynamic_numeric | 304 | throughput: 16447368 dynamic numeric ops/s |
| array_push | 55 | throughput: 36363636 pushes/s |
| array_length | 24 | throughput: 20833333 lengths/s |
| string_concat | 931 | throughput: 5370569 concats/s |
| string_length | 21 | throughput: 23809523 lengths/s |
| intrinsic_dispatch | 649 | throughput: 23112480 intrinsic calls/s |
| function_call | 280 | throughput: 21428571 calls/s |
| array_callbacks | 863 | throughput: 11123986 callbacks/s |
| record_update | 89 | throughput: 11235955 updates/s |
| unicode_char_at | 267 | throughput: 11235955 chars/s |
| wrapper_payload | 697 | throughput: 14347202 wrappers/s |
| string_search | 69 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |

## 2026-10-15 — Failure dump balanced paired latency baseline

- Group: `failure-dump`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 392 | throughput: enabled/disabled median change 4.761904761904759% |
| failure_dump_calls | 355 | throughput: enabled/disabled median change 5.263157894736835% |
| failure_dump_globals | 701 | throughput: enabled/disabled median change 0% |
| failure_dump_spawn_wait | 966 | throughput: enabled/disabled median change 3.773584905660371% |
| failure_dump_suspension | 9161 | throughput: enabled/disabled median change 0.38986354775828147% |
| failure_dump_failure | 218 | throughput: enabled/disabled median change 18.181818181818187% |
| failure_dump_large_report | 706 | throughput: enabled/disabled median change 384.61538461538464% |

Local release baseline on a working tree based on `c49c0ae20`, using Rust
1.99.0. Both modes pass the same unique report path, so they share the
operating system's per-command-line process start cost. Three complete runs
on a quiet system agree. `elapsed_ms` sums both modes' nine measured samples;
the percentage compares rounded millisecond medians within the run and is not
a before/after implementation speedup. Scalar, call and global rows differ by
at most one timer tick. The [paired results and measurement limits](failure-dump.md#local-measurements-2026-10-15)
include saved sample arrays, report sizes and in-process live-peer profiling.

## 2026-10-14 — In-place typed real register stores

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 1024 | throughput: 48828125 iters/s |
| typed_real | 173 | - |
| typed_boolean | 287 | - |
| typed_string | 165 | - |
| global_access | 132 | throughput: 37878787 global updates/s |
| record_field_access | 326 | throughput: 46012269 field accesses/s |
| closure_call | 162 | throughput: 18518518 closure calls/s |
| mutable_capture | 218 | throughput: 13761467 captured updates/s |
| branch_dispatch | 535 | throughput: 37383177 branches/s |
| dynamic_numeric | 301 | throughput: 16611295 dynamic numeric ops/s |
| array_push | 60 | throughput: 33333333 pushes/s |
| array_length | 24 | throughput: 20833333 lengths/s |
| string_concat | 938 | throughput: 5330490 concats/s |
| string_length | 24 | throughput: 20833333 lengths/s |
| intrinsic_dispatch | 699 | throughput: 21459227 intrinsic calls/s |
| function_call | 280 | throughput: 21428571 calls/s |
| array_callbacks | 871 | throughput: 11021814 callbacks/s |
| record_update | 87 | throughput: 11494252 updates/s |
| unicode_char_at | 274 | throughput: 10948905 chars/s |
| wrapper_payload | 745 | throughput: 13422818 wrappers/s |
| string_search | 78 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |

## 2026-10-14 — Direct VM integer, real and global access fast paths

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 1032 | throughput: 48449612 iters/s |
| typed_real | 264 | - |
| typed_boolean | 270 | - |
| typed_string | 162 | - |
| global_access | 128 | throughput: 39062500 global updates/s |
| record_field_access | 333 | throughput: 45045045 field accesses/s |
| closure_call | 160 | throughput: 18750000 closure calls/s |
| mutable_capture | 215 | throughput: 13953488 captured updates/s |
| branch_dispatch | 535 | throughput: 37383177 branches/s |
| dynamic_numeric | 301 | throughput: 16611295 dynamic numeric ops/s |
| array_push | 62 | throughput: 32258064 pushes/s |
| array_length | 24 | throughput: 20833333 lengths/s |
| string_concat | 926 | throughput: 5399568 concats/s |
| string_length | 24 | throughput: 20833333 lengths/s |
| intrinsic_dispatch | 689 | throughput: 21770682 intrinsic calls/s |
| function_call | 289 | throughput: 20761245 calls/s |
| array_callbacks | 826 | throughput: 11622276 callbacks/s |
| record_update | 86 | throughput: 11627906 updates/s |
| unicode_char_at | 274 | throughput: 10948905 chars/s |
| wrapper_payload | 739 | throughput: 13531799 wrappers/s |
| string_search | 76 | - |
| substring_ascii | 7 | - |
| substring_unicode | 11 | - |

## 2026-10-14 — Constant-time task scope child completion

- Group: `concurrency`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| task_spawn_wait | 504 | throughput: 198412 tasks/s |
| task_array_callbacks | 713 | throughput: 5385694 callbacks/s |

## 2026-10-12 — Full-suite baseline

- Group: `all`
- Suite: [`suite.toml`](suite.toml)
- Commit: `7d8bfecf89ad85e7b325a349f2b00001ad3e5080`
- Command: `cargo bench-fpas run`
- Local snapshot: `baseline-2026-10-12` (gitignored)

This completed 65-benchmark run is the new starting point. Earlier history entries
and local measurement snapshots were removed. The values are taken from the existing
run, without another measurement. Times exclude the preparatory Rust build; 0 ms is
below the reported millisecond resolution. The local snapshot preserves elapsed times
and throughput; full per-workload stdout was not retained.

Compare the full suite with `cargo bench-fpas compare baseline-2026-10-12`.

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| mandelbrot_render | 856 | - |
| mandelbrot_paint | 887 | - |
| integer_loop | 1083 | throughput: 46168051 iters/s |
| typed_real | 218 | - |
| typed_boolean | 241 | - |
| typed_string | 159 | - |
| global_access | 118 | throughput: 42372881 global updates/s |
| record_field_access | 344 | throughput: 43604651 field accesses/s |
| closure_call | 141 | throughput: 21276595 closure calls/s |
| mutable_capture | 215 | throughput: 13953488 captured updates/s |
| branch_dispatch | 667 | throughput: 29985007 branches/s |
| dynamic_numeric | 284 | throughput: 17605633 dynamic numeric ops/s |
| array_push | 61 | throughput: 32786885 pushes/s |
| array_length | 24 | throughput: 20833333 lengths/s |
| string_concat | 966 | throughput: 5175983 concats/s |
| string_length | 23 | throughput: 21739130 lengths/s |
| intrinsic_dispatch | 685 | throughput: 21897810 intrinsic calls/s |
| function_call | 242 | throughput: 24793388 calls/s |
| array_callbacks | 757 | throughput: 12681638 callbacks/s |
| record_update | 86 | throughput: 11627906 updates/s |
| unicode_char_at | 369 | throughput: 8130081 chars/s |
| wrapper_payload | 658 | throughput: 15197568 wrappers/s |
| task_spawn_wait | 469 | throughput: 213219 tasks/s |
| task_array_callbacks | 718 | throughput: 5348189 callbacks/s |
| tui_headless | 1546 | throughput: 323 frames/s |
| notes_headless | 2950 | throughput: 84 frames/s |
| tui_styled_cells | 352 | - |
| analysis_queries | 235 | - |
| string_search | 89 | - |
| compiler_lowering | 1789 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |
| project_queries | 935 | - |
| project_edits | 2843 | - |
| project_overlapping_queries | 648 | - |
| project_build_cold | 1754 | - |
| project_build_warm | 1079 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 4 | - |
| scalar_membership | 27 | - |
| array_pop | 1 | - |
| text_area_locate | 900 | - |
| http_body_accumulation | 105 | - |
| local_index_write | 16 | throughput: 40000000 writes/s |
| cell_grid_headless | 104 | throughput: 961 paints/s |
| partial_surface_fill | 62 | - |
| source_utf8_ascii_2048 | 8 | - |
| source_utf8_ascii_4096 | 16 | - |
| source_utf8_ascii_8192 | 34 | - |
| source_utf8_unicode_2048 | 28 | - |
| source_utf8_unicode_4096 | 58 | - |
| source_utf8_unicode_8192 | 115 | - |
| source_notes_ascii_128 | 3 | - |
| source_notes_ascii_256 | 7 | - |
| source_notes_ascii_512 | 16 | - |
| source_queue_ascii_256 | 60 | - |
| source_queue_ascii_512 | 122 | - |
| source_queue_ascii_1024 | 242 | - |
| source_navigation_long_512 | 27 | - |
| source_navigation_long_1024 | 50 | - |
| source_navigation_long_2048 | 99 | - |
| source_navigation_lines_512 | 7 | - |
| source_navigation_lines_1024 | 11 | - |
| source_navigation_lines_2048 | 19 | - |

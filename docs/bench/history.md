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

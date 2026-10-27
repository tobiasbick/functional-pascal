# Benchmark history

Snapshots from the `cargo bench-fpas` harness. Absolute times are machine-specific; use them
to track relative progress on the same machine and to see which changes moved which benches.

Do **not** record hostnames, usernames, paths, or other machine-identifying metadata.

Update after a meaningful performance change:

```sh
cargo bench-fpas record "short note about the change"
cargo bench-fpas record "vm-only note" --group vm
```

Newest entries are prepended below this header.

## 2026-10-23 — Full suite after AP48/AP49: parallel-task, aggregate-store and collection-threshold fixes (on ace31bf21)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)
- Measurement: one local release run per workload with `cargo bench-fpas record`;
  all 85 workloads succeeded. Interpreted with a rerun of the `failure-dump`,
  `tooling` and `tui` groups.
- Against the previous full run on `ea5a9ee7d`: 45045 ms versus 41593 ms in total,
  but 23949 ms versus 23926 ms without the seven `failure_dump_*` rows. Those rows
  are dominated by process starts and moved 20–50 % in both directions between the
  run and its rerun. Their enabled/disabled change stayed near 0 % except for
  `failure_dump_failure` and `failure_dump_large_report`, which write a report.
- `compiler_lowering` is reproducibly slower: 495 and 497 ms versus 405 ms. The
  AP48/AP49 commits do not change the compiler; the toolchain pin and dependency
  update `297604f78` lies in between. The cause is not diagnosed.
- Against the pre-AP48 reference `76ac7d6df`: 20727 ms versus 41863 ms over the 65
  common rows without `failure_dump_*`. `mandelbrot_render` 411 versus 1093 ms,
  `mutable_capture` 202 versus 196 ms, `closure_call` 135 versus 145 ms.

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 1465 | throughput: enabled/disabled median change 0% |
| failure_dump_calls | 1510 | throughput: enabled/disabled median change 0% |
| failure_dump_globals | 1740 | throughput: enabled/disabled median change -1.0309278350515427% |
| failure_dump_spawn_wait | 2383 | throughput: enabled/disabled median change 0.7633587786259568% |
| failure_dump_suspension | 10755 | throughput: enabled/disabled median change -2.463054187192114% |
| failure_dump_failure | 1427 | throughput: enabled/disabled median change 6.578947368421055% |
| failure_dump_large_report | 1816 | throughput: enabled/disabled median change 68% |
| mandelbrot_render | 411 | - |
| mandelbrot_paint | 546 | - |
| integer_loop | 582 | throughput: 85910652 iters/s |
| typed_real | 121 | - |
| typed_boolean | 231 | - |
| typed_string | 96 | - |
| global_access | 104 | throughput: 48076923 global updates/s |
| record_field_access | 304 | throughput: 49342105 field accesses/s |
| closure_call | 135 | throughput: 22222222 closure calls/s |
| mutable_capture | 202 | throughput: 14851485 captured updates/s |
| branch_dispatch | 271 | throughput: 73800738 branches/s |
| dynamic_numeric | 276 | throughput: 18115942 dynamic numeric ops/s |
| array_push | 54 | throughput: 37037037 pushes/s |
| array_length | 5 | throughput: 100000000 lengths/s |
| string_concat | 771 | throughput: 6485084 concats/s |
| string_length | 5 | throughput: 100000000 lengths/s |
| intrinsic_dispatch | 141 | throughput: 106382978 intrinsic calls/s |
| function_call | 242 | throughput: 24793388 calls/s |
| array_callbacks | 701 | throughput: 13694721 callbacks/s |
| record_update | 85 | throughput: 11764705 updates/s |
| unicode_char_at | 242 | throughput: 12396694 chars/s |
| wrapper_payload | 328 | throughput: 30487804 wrappers/s |
| task_spawn_wait | 560 | throughput: 178571 tasks/s |
| task_array_callbacks | 287 | throughput: 13379790 callbacks/s |
| tui_headless | 1425 | throughput: 350 frames/s |
| notes_headless | 3096 | throughput: 80 frames/s |
| tui_styled_cells | 351 | - |
| analysis_queries | 231 | - |
| string_search | 68 | - |
| compiler_lowering | 495 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |
| project_queries | 1038 | - |
| project_edits | 851 | - |
| project_overlapping_queries | 674 | - |
| project_build_cold | 2352 | - |
| project_build_warm | 1506 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 2 | - |
| scalar_membership | 3 | - |
| array_pop | 1 | - |
| text_area_locate | 789 | - |
| http_body_accumulation | 190 | - |
| local_index_write | 14 | throughput: 45714285 writes/s |
| cell_grid_headless | 97 | throughput: 1030 paints/s |
| partial_surface_fill | 55 | - |
| source_utf8_ascii_2048 | 5 | - |
| source_utf8_ascii_4096 | 10 | - |
| source_utf8_ascii_8192 | 20 | - |
| source_utf8_unicode_2048 | 19 | - |
| source_utf8_unicode_4096 | 40 | - |
| source_utf8_unicode_8192 | 79 | - |
| source_notes_ascii_128 | 2 | - |
| source_notes_ascii_256 | 4 | - |
| source_notes_ascii_512 | 10 | - |
| source_queue_ascii_256 | 59 | - |
| source_queue_ascii_512 | 110 | - |
| source_queue_ascii_1024 | 241 | - |
| source_navigation_long_512 | 23 | - |
| source_navigation_long_1024 | 43 | - |
| source_navigation_long_2048 | 82 | - |
| source_navigation_lines_512 | 5 | - |
| source_navigation_lines_1024 | 8 | - |
| source_navigation_lines_2048 | 13 | - |
| errors_enum_construction | 24 | - |
| errors_typed_construction | 24 | - |
| errors_generic_direct | 26 | - |
| errors_generic_helper | 53 | - |
| errors_generic_callable | 58 | - |
| errors_map_ok | 260 | - |
| errors_case_ok | 176 | - |
| errors_map_error | 817 | - |
| errors_case_error | 630 | - |
| errors_constructor_error | 720 | - |
| errors_cause_message | 1 | - |
| errors_cause_console | 432 | - |
| errors_deep_image | 1 | - |

## 2026-10-23 — VM comparison: median of three runs (on 4ad57426c)

- Group: `vm`
- Suite: [`suite.toml`](suite.toml)
- Measurement: three sequential local release runs, 23 workloads each; all 69 executions succeeded.
  `elapsed_ms` is the per-workload median; throughput comes from a run with that elapsed time.
- Recorded from existing `save` snapshots, without rerunning `record`:
  `vm-20261023-80e55e662-run1`, `vm-20261023-80e55e662-run2`,
  `vm-20261023-80e55e662-run3`. Preparation started on `80e55e662`;
  the intervening commit to `4ad57426c` changes only AP48 documentation.
- Reference: the full-suite run on `ea5a9ee7d`. VM workload sources, selection
  and arguments are unchanged. This is a historical comparison, not a paired rerun
  of the old executable. Historical load and power conditions are unverified;
  dependencies changed and the current checkout pins Rust 1.99.1. Differences cannot
  be attributed solely to VM changes. Build time is excluded from these measurements.
- Sum of VM row times: 5132 ms reference, 4974 ms current medians, a 3.08% reduction.
  The geometric mean of elapsed-time ratios is 3.04% lower. Neither aggregate is a
  general application-throughput measurement; 5–10 ms rows have coarse resolution.
- `mutable_capture` is the only higher median: 234 ms reference versus 345 ms,
  up 47.44%. Individual runs were 347, 343 and 345 ms. The cause is not diagnosed.

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 577 | throughput: 86655112 iters/s |
| typed_real | 119 | - |
| typed_boolean | 224 | - |
| typed_string | 99 | - |
| global_access | 100 | throughput: 50000000 global updates/s |
| record_field_access | 313 | throughput: 47923322 field accesses/s |
| closure_call | 127 | throughput: 23622047 closure calls/s |
| mutable_capture | 345 | throughput: 8695652 captured updates/s |
| branch_dispatch | 268 | throughput: 74626865 branches/s |
| dynamic_numeric | 259 | throughput: 19305019 dynamic numeric ops/s |
| array_push | 54 | throughput: 37037037 pushes/s |
| array_length | 5 | throughput: 100000000 lengths/s |
| string_concat | 729 | throughput: 6858710 concats/s |
| string_length | 5 | throughput: 100000000 lengths/s |
| intrinsic_dispatch | 139 | throughput: 107913669 intrinsic calls/s |
| function_call | 227 | throughput: 26431718 calls/s |
| array_callbacks | 682 | throughput: 14076246 callbacks/s |
| record_update | 82 | throughput: 12195121 updates/s |
| unicode_char_at | 235 | throughput: 12765957 chars/s |
| wrapper_payload | 305 | throughput: 32786885 wrappers/s |
| string_search | 64 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |

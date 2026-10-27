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

## 2026-10-27: Full suite with PATH FPAS (on 0f31bb8b0)

- Group: `all`; all 87 workloads succeeded (74 FPAS processes and 13 native drivers).
- Suite: [`suite.toml`](suite.toml), unchanged workload selection and arguments.
- One unprofiled local release run per workload, with each workload's existing warmup
  and internal sampling. This table contains single-run results, not the earlier VM medians.
- FPAS workloads used the executable resolved from `PATH`. Its SHA-256 remained
  `1ccc2f114d02839d0943d682226c6f5fc53c9471d25a58be350574cd71e46c43`
  before and after the run. Native drivers were built from this revision before measuring;
  `FPAS_BENCH_CLI` selected the same PATH executable for their child processes.
- The suite entries were invoked directly because `cargo bench-fpas record` always
  rebuilds and selects its own CLI. No Rust build ran during measurement. The results
  were saved in the harness snapshot format as `full-path-20261027` and recorded here.
- Reference: the complete post-change run in `dist-acp-bench-after.log`, captured on
  2026-10-27 at 21:44 Europe/Berlin. This is a historical comparison, not an alternating
  rerun of the previous executable. It does not isolate individual commits.
- `outlier` means an absolute elapsed-time change strictly greater than 3%, evaluated
  before rounding. There are 35 such rows, including five short rows. These are observed
  changes in one run, not confirmed regressions or improvements.
- `short` marks either time at or below 10 ms (19 rows). Their integer-millisecond
  resolution makes percentage comparisons coarse. A reported 0 ms means below the
  reporting resolution; its percentage change is not defined.
- `failure_dump_*` includes child-process startup. The native tooling and project-build
  rows measure compiler, editor or artifact work rather than FPAS program execution.
- Sum of row times: 41328 ms reference, 41544 ms current. This mixes different
  measurement boundaries and is not an application-throughput score.

| bench | reference_ms | elapsed_ms | change | flags | throughput |
|-------|-------------:|-----------:|-------:|-------|------------|
| acp_codec_roundtrip | 109 | 109 | 0.000% | - | - |
| mcp_catalog | 8 | 9 | +12.500% | outlier, short | - |
| failure_dump_scalar | 1249 | 1132 | -9.367% | outlier | throughput: enabled/disabled median change 0% |
| failure_dump_calls | 1461 | 1216 | -16.769% | outlier | throughput: enabled/disabled median change 0% |
| failure_dump_globals | 1438 | 2137 | +48.609% | outlier | throughput: enabled/disabled median change 0.952380952380949% |
| failure_dump_spawn_wait | 1288 | 1595 | +23.835% | outlier | throughput: enabled/disabled median change 19.718309859154928% |
| failure_dump_suspension | 10000 | 10111 | +1.110% | - | throughput: enabled/disabled median change -0.914076782449726% |
| failure_dump_failure | 1059 | 1534 | +44.854% | outlier | throughput: enabled/disabled median change 2.3809523809523796% |
| failure_dump_large_report | 1609 | 1906 | +18.459% | outlier | throughput: enabled/disabled median change 95.23809523809524% |
| mandelbrot_render | 451 | 473 | +4.878% | outlier | - |
| mandelbrot_paint | 510 | 431 | -15.490% | outlier | - |
| integer_loop | 608 | 590 | -2.961% | - | throughput: 84745762 iters/s |
| typed_real | 122 | 120 | -1.639% | - | - |
| typed_boolean | 244 | 233 | -4.508% | outlier | - |
| typed_string | 103 | 96 | -6.796% | outlier | - |
| global_access | 114 | 103 | -9.649% | outlier | throughput: 48543689 global updates/s |
| record_field_access | 323 | 303 | -6.192% | outlier | throughput: 49504950 field accesses/s |
| closure_call | 181 | 138 | -23.757% | outlier | throughput: 21739130 closure calls/s |
| mutable_capture | 203 | 200 | -1.478% | - | throughput: 15000000 captured updates/s |
| branch_dispatch | 278 | 270 | -2.878% | - | throughput: 74074074 branches/s |
| dynamic_numeric | 290 | 324 | +11.724% | outlier | throughput: 15432098 dynamic numeric ops/s |
| array_push | 54 | 54 | 0.000% | - | throughput: 37037037 pushes/s |
| array_length | 5 | 5 | 0.000% | short | throughput: 100000000 lengths/s |
| string_concat | 779 | 762 | -2.182% | - | throughput: 6561679 concats/s |
| string_length | 5 | 5 | 0.000% | short | throughput: 100000000 lengths/s |
| intrinsic_dispatch | 138 | 136 | -1.449% | - | throughput: 110294117 intrinsic calls/s |
| function_call | 238 | 238 | 0.000% | - | throughput: 25210084 calls/s |
| array_callbacks | 721 | 723 | +0.277% | - | throughput: 13278008 callbacks/s |
| record_update | 86 | 88 | +2.326% | - | throughput: 11363636 updates/s |
| unicode_char_at | 236 | 238 | +0.847% | - | throughput: 12605042 chars/s |
| wrapper_payload | 333 | 323 | -3.003% | outlier | throughput: 30959752 wrappers/s |
| task_spawn_wait | 567 | 561 | -1.058% | - | throughput: 178253 tasks/s |
| task_array_callbacks | 294 | 289 | -1.701% | - | throughput: 13287197 callbacks/s |
| tui_headless | 347 | 345 | -0.576% | - | throughput: 1449 frames/s |
| notes_headless | 122 | 114 | -6.557% | outlier | throughput: 2192 frames/s |
| tui_styled_cells | 379 | 363 | -4.222% | outlier | - |
| analysis_queries | 241 | 243 | +0.830% | - | - |
| string_search | 72 | 70 | -2.778% | - | - |
| compiler_lowering | 545 | 455 | -16.514% | outlier | - |
| substring_ascii | 7 | 7 | 0.000% | short | - |
| substring_unicode | 12 | 10 | -16.667% | outlier, short | - |
| project_queries | 1325 | 1088 | -17.887% | outlier | - |
| project_edits | 977 | 884 | -9.519% | outlier | - |
| project_overlapping_queries | 816 | 675 | -17.279% | outlier | - |
| project_build_cold | 4972 | 4665 | -6.175% | outlier | - |
| project_build_warm | 2257 | 2040 | -9.615% | outlier | - |
| unit_artifact_shared_types | 0 | 0 | n/a | short | - |
| program_artifact_shared_types | 0 | 0 | n/a | short | - |
| dictionary_reads | 2 | 2 | 0.000% | short | - |
| scalar_membership | 3 | 3 | 0.000% | short | - |
| array_pop | 1 | 1 | 0.000% | short | - |
| text_area_locate | 0 | 0 | n/a | short | - |
| http_body_accumulation | 187 | 186 | -0.535% | - | - |
| local_index_write | 19 | 15 | -21.053% | outlier | throughput: 42666666 writes/s |
| cell_grid_headless | 43 | 43 | 0.000% | - | throughput: 2325 paints/s |
| partial_surface_fill | 57 | 55 | -3.509% | outlier | - |
| source_utf8_ascii_2048 | 5 | 4 | -20.000% | outlier, short | - |
| source_utf8_ascii_4096 | 10 | 10 | 0.000% | short | - |
| source_utf8_ascii_8192 | 20 | 20 | 0.000% | - | - |
| source_utf8_unicode_2048 | 20 | 19 | -5.000% | outlier | - |
| source_utf8_unicode_4096 | 41 | 40 | -2.439% | - | - |
| source_utf8_unicode_8192 | 82 | 80 | -2.439% | - | - |
| source_notes_ascii_128 | 2 | 3 | +50.000% | outlier, short | - |
| source_notes_ascii_256 | 5 | 5 | 0.000% | short | - |
| source_notes_ascii_512 | 10 | 11 | +10.000% | outlier, short | - |
| source_queue_ascii_256 | 23 | 24 | +4.348% | outlier | - |
| source_queue_ascii_512 | 47 | 46 | -2.128% | - | - |
| source_queue_ascii_1024 | 93 | 92 | -1.075% | - | - |
| source_navigation_long_512 | 23 | 23 | 0.000% | - | - |
| source_navigation_long_1024 | 44 | 43 | -2.273% | - | - |
| source_navigation_long_2048 | 86 | 88 | +2.326% | - | - |
| source_navigation_lines_512 | 7 | 7 | 0.000% | short | - |
| source_navigation_lines_1024 | 11 | 11 | 0.000% | - | - |
| source_navigation_lines_2048 | 19 | 18 | -5.263% | outlier | - |
| errors_enum_construction | 24 | 24 | 0.000% | - | - |
| errors_typed_construction | 26 | 26 | 0.000% | - | - |
| errors_generic_direct | 27 | 27 | 0.000% | - | - |
| errors_generic_helper | 54 | 57 | +5.556% | outlier | - |
| errors_generic_callable | 57 | 58 | +1.754% | - | - |
| errors_map_ok | 266 | 262 | -1.504% | - | - |
| errors_case_ok | 183 | 187 | +2.186% | - | - |
| errors_map_error | 826 | 852 | +3.148% | outlier | - |
| errors_case_error | 656 | 640 | -2.439% | - | - |
| errors_constructor_error | 741 | 725 | -2.159% | - | - |
| errors_cause_message | 1 | 1 | 0.000% | short | - |
| errors_cause_console | 430 | 424 | -1.395% | - | - |
| errors_deep_image | 1 | 1 | 0.000% | short | - |

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

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

## 2026-10-26 — Debugger console waits: ordinary TUI controls

- Group: `tui`
- Suite: [`suite.toml`](suite.toml)
- Paired unprofiled DAP console runs are recorded separately in the
  [debugger runtime report](debugger-runtime.md#timed-terminal-input-2026-10-26).
  F2-route medians fall from 2678 to 1646 ms at 80x24 and from 5816 to 3603 ms
  at 144x45, reductions of 38.5% and 38.1%.
- The rows below are ordinary-execution controls, not attributed debugger gains.

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| tui_headless | 1414 | throughput: 353 frames/s |
| notes_headless | 2641 | throughput: 94 frames/s |
| tui_styled_cells | 341 | - |

## 2026-10-24 — Bounded debugger root publication

- Scope: paired local DAP gameplay measurements, separate from the configured
  `fpas run` suite. See [method, samples and limits](debugger-runtime.md).
- One fixed Into the Dungeon client image, seed 25, region size 1, virtual terminal
  4 by 4; three alternating release runs per runtime. Startup and save loading
  are outside the timer; protocol, inspection and controller polling are included.
- Eight acknowledged westward steps: median 17905 ms before, 1521 ms after
  (91.5% lower wall time, factor 11.8). Every run reaches the same position at
  full health. A separate 64-step smoke check completes in 9219 ms.
- The debugger retains execution access for at most 256 resume-loop iterations
  and publishes before returning or waiting. Instruction scheduling, breakpoint
  checks, tracing and execution limits retain their existing boundaries.
- Temporary target phase timers identified the cost and were removed before
  these measurements. Supported attribution is available through
  [`fpas debug --profile`](../pascal/tools/profiler.md#debug-targets).
- All 85 configured workloads complete against the saved `dap-before` baseline.
  Initial Notes/integer/Mandelbrot deviations were checked with three paired
  controls. Notes, integer and paint medians are within 0.6%; a longer render
  control changes from 5579 to 5491 ms. The linked report retains the initial
  deviations and explains the longer control rather than replacing suite rows.

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
- Reference: the preceding Nachtlauf on `ea5a9ee7d`. VM workload sources, selection
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

## 2026-10-23 — Nachtlauf: Inline-Schnellpfade, Cancellation-Fastpath, flache Arrays, Lowering-Typtabelle, IR-Lookups, Source-Index (auf ea5a9ee7d)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 1029 | throughput: enabled/disabled median change 1.7857142857142918% |
| failure_dump_calls | 1043 | throughput: enabled/disabled median change 0% |
| failure_dump_globals | 1308 | throughput: enabled/disabled median change 1.3888888888888857% |
| failure_dump_spawn_wait | 1686 | throughput: enabled/disabled median change 101.61290322580646% |
| failure_dump_suspension | 9934 | throughput: enabled/disabled median change -0.7207207207207205% |
| failure_dump_failure | 1279 | throughput: enabled/disabled median change 32.8125% |
| failure_dump_large_report | 1388 | throughput: enabled/disabled median change 101.9607843137255% |
| mandelbrot_render | 682 | - |
| mandelbrot_paint | 747 | - |
| integer_loop | 604 | throughput: 82781456 iters/s |
| typed_real | 148 | - |
| typed_boolean | 233 | - |
| typed_string | 102 | - |
| global_access | 103 | throughput: 48543689 global updates/s |
| record_field_access | 324 | throughput: 46296296 field accesses/s |
| closure_call | 135 | throughput: 22222222 closure calls/s |
| mutable_capture | 234 | throughput: 12820512 captured updates/s |
| branch_dispatch | 295 | throughput: 67796610 branches/s |
| dynamic_numeric | 276 | throughput: 18115942 dynamic numeric ops/s |
| array_push | 54 | throughput: 37037037 pushes/s |
| array_length | 5 | throughput: 100000000 lengths/s |
| string_concat | 750 | throughput: 6666666 concats/s |
| string_length | 5 | throughput: 100000000 lengths/s |
| intrinsic_dispatch | 139 | throughput: 107913669 intrinsic calls/s |
| function_call | 260 | throughput: 23076923 calls/s |
| array_callbacks | 721 | throughput: 13314840 callbacks/s |
| record_update | 90 | throughput: 11111111 updates/s |
| unicode_char_at | 261 | throughput: 11494252 chars/s |
| wrapper_payload | 310 | throughput: 32258064 wrappers/s |
| task_spawn_wait | 562 | throughput: 177935 tasks/s |
| task_array_callbacks | 294 | throughput: 13061224 callbacks/s |
| tui_headless | 1462 | throughput: 341 frames/s |
| notes_headless | 2803 | throughput: 89 frames/s |
| tui_styled_cells | 354 | - |
| analysis_queries | 221 | - |
| string_search | 67 | - |
| compiler_lowering | 405 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |
| project_queries | 971 | - |
| project_edits | 798 | - |
| project_overlapping_queries | 625 | - |
| project_build_cold | 2272 | - |
| project_build_warm | 1463 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 2 | - |
| scalar_membership | 3 | - |
| array_pop | 1 | - |
| text_area_locate | 810 | - |
| http_body_accumulation | 179 | - |
| local_index_write | 15 | throughput: 42666666 writes/s |
| cell_grid_headless | 101 | throughput: 990 paints/s |
| partial_surface_fill | 58 | - |
| source_utf8_ascii_2048 | 5 | - |
| source_utf8_ascii_4096 | 11 | - |
| source_utf8_ascii_8192 | 23 | - |
| source_utf8_unicode_2048 | 21 | - |
| source_utf8_unicode_4096 | 43 | - |
| source_utf8_unicode_8192 | 86 | - |
| source_notes_ascii_128 | 2 | - |
| source_notes_ascii_256 | 5 | - |
| source_notes_ascii_512 | 11 | - |
| source_queue_ascii_256 | 55 | - |
| source_queue_ascii_512 | 110 | - |
| source_queue_ascii_1024 | 223 | - |
| source_navigation_long_512 | 24 | - |
| source_navigation_long_1024 | 44 | - |
| source_navigation_long_2048 | 84 | - |
| source_navigation_lines_512 | 5 | - |
| source_navigation_lines_1024 | 8 | - |
| source_navigation_lines_2048 | 13 | - |
| errors_enum_construction | 24 | - |
| errors_typed_construction | 24 | - |
| errors_generic_direct | 26 | - |
| errors_generic_helper | 53 | - |
| errors_generic_callable | 56 | - |
| errors_map_ok | 259 | - |
| errors_case_ok | 180 | - |
| errors_map_error | 786 | - |
| errors_case_error | 640 | - |
| errors_constructor_error | 724 | - |
| errors_cause_message | 1 | - |
| errors_cause_console | 419 | - |
| errors_deep_image | 1 | - |

## 2026-10-22 — Lazy Task-Roots, Transfer-Summary, In-place-Zellen, atomare Druckzähler (auf fefd38b19)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 1423 | throughput: enabled/disabled median change -1.2658227848101262% |
| failure_dump_calls | 1390 | throughput: enabled/disabled median change 1.3157894736842053% |
| failure_dump_globals | 2046 | throughput: enabled/disabled median change 2.6785714285714306% |
| failure_dump_spawn_wait | 2342 | throughput: enabled/disabled median change 96.42857142857142% |
| failure_dump_suspension | 10164 | throughput: enabled/disabled median change 0.5319148936170279% |
| failure_dump_failure | 1218 | throughput: enabled/disabled median change 4.545454545454547% |
| failure_dump_large_report | 1892 | throughput: enabled/disabled median change 74.66666666666666% |
| mandelbrot_render | 734 | - |
| mandelbrot_paint | 740 | - |
| integer_loop | 786 | throughput: 63613231 iters/s |
| typed_real | 169 | - |
| typed_boolean | 271 | - |
| typed_string | 143 | - |
| global_access | 121 | throughput: 41322314 global updates/s |
| record_field_access | 399 | throughput: 37593984 field accesses/s |
| closure_call | 155 | throughput: 19354838 closure calls/s |
| mutable_capture | 243 | throughput: 12345679 captured updates/s |
| branch_dispatch | 381 | throughput: 52493438 branches/s |
| dynamic_numeric | 308 | throughput: 16233766 dynamic numeric ops/s |
| array_push | 68 | throughput: 29411764 pushes/s |
| array_length | 6 | throughput: 83333333 lengths/s |
| string_concat | 855 | throughput: 5847953 concats/s |
| string_length | 6 | throughput: 83333333 lengths/s |
| intrinsic_dispatch | 177 | throughput: 84745762 intrinsic calls/s |
| function_call | 277 | throughput: 21660649 calls/s |
| array_callbacks | 769 | throughput: 12483745 callbacks/s |
| record_update | 111 | throughput: 9009009 updates/s |
| unicode_char_at | 276 | throughput: 10869565 chars/s |
| wrapper_payload | 339 | throughput: 29498525 wrappers/s |
| task_spawn_wait | 546 | throughput: 183150 tasks/s |
| task_array_callbacks | 312 | throughput: 12307692 callbacks/s |
| tui_headless | 1676 | throughput: 298 frames/s |
| notes_headless | 3464 | throughput: 72 frames/s |
| tui_styled_cells | 435 | - |
| analysis_queries | 231 | - |
| string_search | 74 | - |
| compiler_lowering | 2209 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |
| project_queries | 1049 | - |
| project_edits | 4313 | - |
| project_overlapping_queries | 792 | - |
| project_build_cold | 2570 | - |
| project_build_warm | 1518 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 2 | - |
| scalar_membership | 4 | - |
| array_pop | 1 | - |
| text_area_locate | 907 | - |
| http_body_accumulation | 228 | - |
| local_index_write | 17 | throughput: 37647058 writes/s |
| cell_grid_headless | 131 | throughput: 763 paints/s |
| partial_surface_fill | 76 | - |
| source_utf8_ascii_2048 | 6 | - |
| source_utf8_ascii_4096 | 12 | - |
| source_utf8_ascii_8192 | 24 | - |
| source_utf8_unicode_2048 | 24 | - |
| source_utf8_unicode_4096 | 50 | - |
| source_utf8_unicode_8192 | 106 | - |
| source_notes_ascii_128 | 2 | - |
| source_notes_ascii_256 | 6 | - |
| source_notes_ascii_512 | 14 | - |
| source_queue_ascii_256 | 76 | - |
| source_queue_ascii_512 | 142 | - |
| source_queue_ascii_1024 | 283 | - |
| source_navigation_long_512 | 25 | - |
| source_navigation_long_1024 | 49 | - |
| source_navigation_long_2048 | 93 | - |
| source_navigation_lines_512 | 6 | - |
| source_navigation_lines_1024 | 8 | - |
| source_navigation_lines_2048 | 14 | - |
| errors_enum_construction | 28 | - |
| errors_typed_construction | 30 | - |
| errors_generic_direct | 30 | - |
| errors_generic_helper | 59 | - |
| errors_generic_callable | 67 | - |
| errors_map_ok | 290 | - |
| errors_case_ok | 197 | - |
| errors_map_error | 922 | - |
| errors_case_error | 766 | - |
| errors_constructor_error | 834 | - |
| errors_cause_message | 1 | - |
| errors_cause_console | 453 | - |
| errors_deep_image | 1 | - |

## 2026-10-21 — Nach Optimierungen (O(1)-Aufrufprüfung, Task-Neutralität, Callbacks im Batch, 7f3ee572b)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 1094 | throughput: enabled/disabled median change 1.6949152542372872% |
| failure_dump_calls | 1512 | throughput: enabled/disabled median change 1.2048192771084274% |
| failure_dump_globals | 1413 | throughput: enabled/disabled median change 0% |
| failure_dump_spawn_wait | 2402 | throughput: enabled/disabled median change 0% |
| failure_dump_suspension | 9980 | throughput: enabled/disabled median change 1.2658227848101262% |
| failure_dump_failure | 965 | throughput: enabled/disabled median change 3.9215686274509807% |
| failure_dump_large_report | 1422 | throughput: enabled/disabled median change 96.15384615384616% |
| mandelbrot_render | 1185 | - |
| mandelbrot_paint | 1193 | - |
| integer_loop | 775 | throughput: 64516129 iters/s |
| typed_real | 163 | - |
| typed_boolean | 268 | - |
| typed_string | 140 | - |
| global_access | 120 | throughput: 41666666 global updates/s |
| record_field_access | 434 | throughput: 34562211 field accesses/s |
| closure_call | 163 | throughput: 18404907 closure calls/s |
| mutable_capture | 548 | throughput: 5474452 captured updates/s |
| branch_dispatch | 374 | throughput: 53475935 branches/s |
| dynamic_numeric | 302 | throughput: 16556291 dynamic numeric ops/s |
| array_push | 66 | throughput: 30303030 pushes/s |
| array_length | 23 | throughput: 21739130 lengths/s |
| string_concat | 952 | throughput: 5252100 concats/s |
| string_length | 23 | throughput: 21739130 lengths/s |
| intrinsic_dispatch | 685 | throughput: 21897810 intrinsic calls/s |
| function_call | 281 | throughput: 21352313 calls/s |
| array_callbacks | 760 | throughput: 12631578 callbacks/s |
| record_update | 112 | throughput: 8928571 updates/s |
| unicode_char_at | 273 | throughput: 10989010 chars/s |
| wrapper_payload | 828 | throughput: 12077294 wrappers/s |
| task_spawn_wait | 812 | throughput: 123152 tasks/s |
| task_array_callbacks | 1059 | throughput: 3626062 callbacks/s |
| tui_headless | 1722 | throughput: 290 frames/s |
| notes_headless | 3381 | throughput: 73 frames/s |
| tui_styled_cells | 418 | - |
| analysis_queries | 222 | - |
| string_search | 73 | - |
| compiler_lowering | 2101 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |
| project_queries | 999 | - |
| project_edits | 3728 | - |
| project_overlapping_queries | 664 | - |
| project_build_cold | 2305 | - |
| project_build_warm | 1422 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 3 | - |
| scalar_membership | 3 | - |
| array_pop | 1 | - |
| text_area_locate | 893 | - |
| http_body_accumulation | 228 | - |
| local_index_write | 21 | throughput: 30476190 writes/s |
| cell_grid_headless | 129 | throughput: 775 paints/s |
| partial_surface_fill | 76 | - |
| source_utf8_ascii_2048 | 8 | - |
| source_utf8_ascii_4096 | 16 | - |
| source_utf8_ascii_8192 | 32 | - |
| source_utf8_unicode_2048 | 28 | - |
| source_utf8_unicode_4096 | 56 | - |
| source_utf8_unicode_8192 | 116 | - |
| source_notes_ascii_128 | 3 | - |
| source_notes_ascii_256 | 8 | - |
| source_notes_ascii_512 | 18 | - |
| source_queue_ascii_256 | 73 | - |
| source_queue_ascii_512 | 149 | - |
| source_queue_ascii_1024 | 296 | - |
| source_navigation_long_512 | 26 | - |
| source_navigation_long_1024 | 47 | - |
| source_navigation_long_2048 | 92 | - |
| source_navigation_lines_512 | 7 | - |
| source_navigation_lines_1024 | 11 | - |
| source_navigation_lines_2048 | 19 | - |
| errors_enum_construction | 28 | - |
| errors_typed_construction | 30 | - |
| errors_generic_direct | 30 | - |
| errors_generic_helper | 58 | - |
| errors_generic_callable | 61 | - |
| errors_map_ok | 327 | - |
| errors_case_ok | 241 | - |
| errors_map_error | 897 | - |
| errors_case_error | 754 | - |
| errors_constructor_error | 811 | - |
| errors_cause_message | 1 | - |
| errors_cause_console | 442 | - |
| errors_deep_image | 1 | - |

## 2026-10-21 — Aktueller Stand (AP48, Optimierungen 07a38a95f)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 1165 | throughput: enabled/disabled median change -1.538461538461533% |
| failure_dump_calls | 1193 | throughput: enabled/disabled median change -1.538461538461533% |
| failure_dump_globals | 1461 | throughput: enabled/disabled median change 0% |
| failure_dump_spawn_wait | 2556 | throughput: enabled/disabled median change 0.7142857142857082% |
| failure_dump_suspension | 9966 | throughput: enabled/disabled median change -0.5357142857142918% |
| failure_dump_failure | 1064 | throughput: enabled/disabled median change 8.92857142857143% |
| failure_dump_large_report | 1553 | throughput: enabled/disabled median change 86.20689655172413% |
| mandelbrot_render | 1243 | - |
| mandelbrot_paint | 1153 | - |
| integer_loop | 787 | throughput: 63532401 iters/s |
| typed_real | 163 | - |
| typed_boolean | 255 | - |
| typed_string | 136 | - |
| global_access | 117 | throughput: 42735042 global updates/s |
| record_field_access | 368 | throughput: 40760869 field accesses/s |
| closure_call | 144 | throughput: 20833333 closure calls/s |
| mutable_capture | 542 | throughput: 5535055 captured updates/s |
| branch_dispatch | 359 | throughput: 55710306 branches/s |
| dynamic_numeric | 300 | throughput: 16666666 dynamic numeric ops/s |
| array_push | 65 | throughput: 30769230 pushes/s |
| array_length | 24 | throughput: 20833333 lengths/s |
| string_concat | 921 | throughput: 5428881 concats/s |
| string_length | 22 | throughput: 22727272 lengths/s |
| intrinsic_dispatch | 677 | throughput: 22156573 intrinsic calls/s |
| function_call | 270 | throughput: 22222222 calls/s |
| array_callbacks | 1682 | throughput: 5707491 callbacks/s |
| record_update | 99 | throughput: 10101010 updates/s |
| unicode_char_at | 275 | throughput: 10909090 chars/s |
| wrapper_payload | 816 | throughput: 12254901 wrappers/s |
| task_spawn_wait | 11472 | throughput: 8716 tasks/s |
| task_array_callbacks | 1335 | throughput: 2876404 callbacks/s |
| tui_headless | 1722 | throughput: 290 frames/s |
| notes_headless | 3376 | throughput: 74 frames/s |
| tui_styled_cells | 407 | - |
| analysis_queries | 224 | - |
| string_search | 72 | - |
| compiler_lowering | 2073 | - |
| substring_ascii | 6 | - |
| substring_unicode | 11 | - |
| project_queries | 987 | - |
| project_edits | 4012 | - |
| project_overlapping_queries | 746 | - |
| project_build_cold | 2463 | - |
| project_build_warm | 1420 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 4 | - |
| scalar_membership | 3 | - |
| array_pop | 1 | - |
| text_area_locate | 880 | - |
| http_body_accumulation | 247 | - |
| local_index_write | 21 | throughput: 30476190 writes/s |
| cell_grid_headless | 123 | throughput: 813 paints/s |
| partial_surface_fill | 78 | - |
| source_utf8_ascii_2048 | 8 | - |
| source_utf8_ascii_4096 | 16 | - |
| source_utf8_ascii_8192 | 32 | - |
| source_utf8_unicode_2048 | 28 | - |
| source_utf8_unicode_4096 | 56 | - |
| source_utf8_unicode_8192 | 113 | - |
| source_notes_ascii_128 | 3 | - |
| source_notes_ascii_256 | 8 | - |
| source_notes_ascii_512 | 17 | - |
| source_queue_ascii_256 | 73 | - |
| source_queue_ascii_512 | 147 | - |
| source_queue_ascii_1024 | 305 | - |
| source_navigation_long_512 | 27 | - |
| source_navigation_long_1024 | 49 | - |
| source_navigation_long_2048 | 94 | - |
| source_navigation_lines_512 | 7 | - |
| source_navigation_lines_1024 | 12 | - |
| source_navigation_lines_2048 | 21 | - |
| errors_enum_construction | 29 | - |
| errors_typed_construction | 34 | - |
| errors_generic_direct | 31 | - |
| errors_generic_helper | 60 | - |
| errors_generic_callable | 62 | - |
| errors_map_ok | 328 | - |
| errors_case_ok | 248 | - |
| errors_map_error | 956 | - |
| errors_case_error | 763 | - |
| errors_constructor_error | 893 | - |
| errors_cause_message | 1 | - |
| errors_cause_console | 432 | - |
| errors_deep_image | 1 | - |

## 2026-10-21 — Referenz 17.10. (76ac7d6df)

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| failure_dump_scalar | 504 | throughput: enabled/disabled median change 3.7037037037037095% |
| failure_dump_calls | 536 | throughput: enabled/disabled median change 0% |
| failure_dump_globals | 739 | throughput: enabled/disabled median change 0% |
| failure_dump_spawn_wait | 1087 | throughput: enabled/disabled median change 6.779661016949149% |
| failure_dump_suspension | 9210 | throughput: enabled/disabled median change -0.38759689922480334% |
| failure_dump_failure | 365 | throughput: enabled/disabled median change 15.78947368421052% |
| failure_dump_large_report | 770 | throughput: enabled/disabled median change 220% |
| mandelbrot_render | 1093 | - |
| mandelbrot_paint | 1057 | - |
| integer_loop | 755 | throughput: 66225165 iters/s |
| typed_real | 164 | - |
| typed_boolean | 253 | - |
| typed_string | 132 | - |
| global_access | 100 | throughput: 50000000 global updates/s |
| record_field_access | 322 | throughput: 46583850 field accesses/s |
| closure_call | 145 | throughput: 20689655 closure calls/s |
| mutable_capture | 196 | throughput: 15306122 captured updates/s |
| branch_dispatch | 363 | throughput: 55096418 branches/s |
| dynamic_numeric | 294 | throughput: 17006802 dynamic numeric ops/s |
| array_push | 55 | throughput: 36363636 pushes/s |
| array_length | 23 | throughput: 21739130 lengths/s |
| string_concat | 933 | throughput: 5359056 concats/s |
| string_length | 21 | throughput: 23809523 lengths/s |
| intrinsic_dispatch | 654 | throughput: 22935779 intrinsic calls/s |
| function_call | 266 | throughput: 22556390 calls/s |
| array_callbacks | 804 | throughput: 11940298 callbacks/s |
| record_update | 90 | throughput: 11111111 updates/s |
| unicode_char_at | 270 | throughput: 11111111 chars/s |
| wrapper_payload | 692 | throughput: 14450867 wrappers/s |
| task_spawn_wait | 539 | throughput: 185528 tasks/s |
| task_array_callbacks | 698 | throughput: 5501432 callbacks/s |
| tui_headless | 1521 | throughput: 328 frames/s |
| notes_headless | 3309 | throughput: 75 frames/s |
| tui_styled_cells | 353 | - |
| analysis_queries | 377 | - |
| string_search | 69 | - |
| compiler_lowering | 1891 | - |
| substring_ascii | 6 | - |
| substring_unicode | 10 | - |
| project_queries | 2037 | - |
| project_edits | 5891 | - |
| project_overlapping_queries | 852 | - |
| project_build_cold | 3479 | - |
| project_build_warm | 10056 | - |
| unit_artifact_shared_types | 0 | - |
| program_artifact_shared_types | 0 | - |
| dictionary_reads | 3 | - |
| scalar_membership | 30 | - |
| array_pop | 1 | - |
| text_area_locate | 848 | - |
| http_body_accumulation | 169 | - |
| local_index_write | 15 | throughput: 42666666 writes/s |
| cell_grid_headless | 93 | throughput: 1075 paints/s |
| partial_surface_fill | 55 | - |
| source_utf8_ascii_2048 | 7 | - |
| source_utf8_ascii_4096 | 15 | - |
| source_utf8_ascii_8192 | 30 | - |
| source_utf8_unicode_2048 | 26 | - |
| source_utf8_unicode_4096 | 53 | - |
| source_utf8_unicode_8192 | 108 | - |
| source_notes_ascii_128 | 3 | - |
| source_notes_ascii_256 | 7 | - |
| source_notes_ascii_512 | 15 | - |
| source_queue_ascii_256 | 60 | - |
| source_queue_ascii_512 | 121 | - |
| source_queue_ascii_1024 | 242 | - |
| source_navigation_long_512 | 24 | - |
| source_navigation_long_1024 | 45 | - |
| source_navigation_long_2048 | 89 | - |
| source_navigation_lines_512 | 6 | - |
| source_navigation_lines_1024 | 10 | - |
| source_navigation_lines_2048 | 18 | - |

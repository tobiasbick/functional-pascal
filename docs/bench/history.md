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

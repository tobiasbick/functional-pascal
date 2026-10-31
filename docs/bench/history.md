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

## 2026-10-31 — Full suite baseline with repeated VM measurements

- Group: `all`
- Suite: [`suite.toml`](suite.toml)
- All 87 workloads succeeded across 179 process measurements.
- Local, unprofiled release run with Fat-LTO and one codegen unit. The harness
  built the release CLI before measurement; no other builds or tests ran during
  the measured workloads.
- VM rows contain five independent process measurements with each program's
  existing warmup. Other groups retain one measurement per workload.
- `min_ms` and `max_ms` are the observed range, not a confidence interval.
  Single-sample rows do not estimate variability. A reported 0 ms is below the
  integer-millisecond reporting resolution.
- This is a fresh baseline after increasing the short VM workloads and resetting
  history. Future comparisons require the same arguments and sample counts.

| bench | samples | median_ms | min_ms | max_ms | args |
|-------|---------|-----------|--------|--------|------|
| acp_codec_roundtrip | 1 | 97.0 | 97 | 97 | ["acp","5000"] |
| mcp_catalog | 1 | 8.0 | 8 | 8 | ["mcp","2000"] |
| failure_dump_scalar | 1 | 1078.0 | 1078 | 1078 | ["scalar","9"] |
| failure_dump_calls | 1 | 1125.0 | 1125 | 1125 | ["calls","9"] |
| failure_dump_globals | 1 | 1565.0 | 1565 | 1565 | ["globals","9"] |
| failure_dump_spawn_wait | 1 | 1265.0 | 1265 | 1265 | ["spawn-wait","9"] |
| failure_dump_suspension | 1 | 10001.0 | 10001 | 10001 | ["suspension","9"] |
| failure_dump_failure | 1 | 999.0 | 999 | 999 | ["failure","9"] |
| failure_dump_large_report | 1 | 1584.0 | 1584 | 1584 | ["large-report","9"] |
| mandelbrot_render | 1 | 452.0 | 452 | 452 | ["12","render"] |
| mandelbrot_paint | 1 | 515.0 | 515 | 515 | ["12","paint"] |
| integer_loop | 5 | 558.0 | 552 | 600 | ["50000000"] |
| typed_real | 5 | 764.0 | 757 | 777 | ["35000000","real"] |
| typed_boolean | 5 | 775.0 | 758 | 851 | ["25000000","boolean"] |
| typed_string | 5 | 728.0 | 727 | 758 | ["35000000","string"] |
| global_access | 5 | 725.0 | 723 | 726 | ["35000000"] |
| record_field_access | 5 | 785.0 | 763 | 909 | ["8000000"] |
| closure_call | 5 | 723.0 | 695 | 797 | ["16000000"] |
| mutable_capture | 5 | 709.0 | 700 | 768 | ["12000000"] |
| branch_dispatch | 5 | 731.0 | 726 | 756 | ["50000000"] |
| dynamic_numeric | 5 | 740.0 | 739 | 749 | ["15000000"] |
| array_push | 5 | 762.0 | 754 | 811 | ["28000000"] |
| array_length | 5 | 728.0 | 727 | 729 | ["70000000"] |
| string_concat | 5 | 755.0 | 748 | 768 | ["5000000"] |
| string_length | 5 | 732.0 | 726 | 739 | ["70000000"] |
| intrinsic_dispatch | 5 | 685.0 | 675 | 730 | ["25000000"] |
| function_call | 5 | 692.0 | 686 | 706 | ["9000000"] |
| array_callbacks | 5 | 636.0 | 630 | 648 | ["50000"] |
| record_update | 5 | 632.0 | 624 | 703 | ["8000000"] |
| unicode_char_at | 5 | 674.0 | 665 | 680 | ["9000000"] |
| wrapper_payload | 5 | 715.0 | 713 | 840 | ["12000000"] |
| task_spawn_wait | 1 | 549.0 | 549 | 549 | ["100000"] |
| task_array_callbacks | 1 | 268.0 | 268 | 268 | ["20000"] |
| tui_headless | 1 | 373.0 | 373 | 373 | ["500"] |
| notes_headless | 1 | 29.0 | 29 | 29 | ["250"] |
| tui_styled_cells | 1 | 358.0 | 358 | 358 | ["40"] |
| analysis_queries | 1 | 233.0 | 233 | 233 | ["1000","500"] |
| string_search | 5 | 730.0 | 727 | 772 | ["6000000"] |
| compiler_lowering | 1 | 479.0 | 479 | 479 | ["30","1000"] |
| substring_ascii | 5 | 829.0 | 828 | 842 | ["1200000","ascii"] |
| substring_unicode | 5 | 757.0 | 753 | 807 | ["700000","unicode"] |
| project_queries | 1 | 1102.0 | 1102 | 1102 | ["40","20","warm"] |
| project_edits | 1 | 1099.0 | 1099 | 1099 | ["3","20","edits"] |
| project_overlapping_queries | 1 | 729.0 | 729 | 729 | ["20","20","overlap"] |
| project_build_cold | 1 | 4207.0 | 4207 | 4207 | ["3","cold"] |
| project_build_warm | 1 | 2071.0 | 2071 | 2071 | ["10","warm"] |
| unit_artifact_shared_types | 1 | 1.0 | 1 | 1 | ["500","16"] |
| program_artifact_shared_types | 1 | 0.0 | 0 | 0 | ["500","16"] |
| dictionary_reads | 1 | 2.0 | 2 | 2 | ["20000","1024"] |
| scalar_membership | 1 | 3.0 | 3 | 3 | ["4000","4096"] |
| array_pop | 1 | 1.0 | 1 | 1 | ["4","12000"] |
| text_area_locate | 1 | 0.0 | 0 | 0 | ["80","8192"] |
| http_body_accumulation | 1 | 266.0 | 266 | 266 | ["40","1024"] |
| local_index_write | 1 | 14.0 | 14 | 14 | ["5000","128"] |
| cell_grid_headless | 1 | 43.0 | 43 | 43 | ["100"] |
| partial_surface_fill | 1 | 44.0 | 44 | 44 | ["100"] |
| source_utf8_ascii_2048 | 1 | 0.0 | 0 | 0 | ["utf8","2048","x"] |
| source_utf8_ascii_4096 | 1 | 0.0 | 0 | 0 | ["utf8","4096","x"] |
| source_utf8_ascii_8192 | 1 | 0.0 | 0 | 0 | ["utf8","8192","x"] |
| source_utf8_unicode_2048 | 1 | 0.0 | 0 | 0 | ["utf8","2048","é中"] |
| source_utf8_unicode_4096 | 1 | 0.0 | 0 | 0 | ["utf8","4096","é中"] |
| source_utf8_unicode_8192 | 1 | 0.0 | 0 | 0 | ["utf8","8192","é中"] |
| source_notes_ascii_128 | 1 | 2.0 | 2 | 2 | ["notes","128","x"] |
| source_notes_ascii_256 | 1 | 4.0 | 4 | 4 | ["notes","256","x"] |
| source_notes_ascii_512 | 1 | 9.0 | 9 | 9 | ["notes","512","x"] |
| source_queue_ascii_256 | 1 | 27.0 | 27 | 27 | ["queue","256","x"] |
| source_queue_ascii_512 | 1 | 55.0 | 55 | 55 | ["queue","512","x"] |
| source_queue_ascii_1024 | 1 | 112.0 | 112 | 112 | ["queue","1024","x"] |
| source_navigation_long_512 | 1 | 23.0 | 23 | 23 | ["512","long"] |
| source_navigation_long_1024 | 1 | 43.0 | 43 | 43 | ["1024","long"] |
| source_navigation_long_2048 | 1 | 82.0 | 82 | 82 | ["2048","long"] |
| source_navigation_lines_512 | 1 | 6.0 | 6 | 6 | ["512","lines"] |
| source_navigation_lines_1024 | 1 | 10.0 | 10 | 10 | ["1024","lines"] |
| source_navigation_lines_2048 | 1 | 17.0 | 17 | 17 | ["2048","lines"] |
| errors_enum_construction | 1 | 23.0 | 23 | 23 | ["plain","100000"] |
| errors_typed_construction | 1 | 25.0 | 25 | 25 | ["typed","100000"] |
| errors_generic_direct | 1 | 26.0 | 26 | 26 | ["direct","100000"] |
| errors_generic_helper | 1 | 54.0 | 54 | 54 | ["generic","100000"] |
| errors_generic_callable | 1 | 82.0 | 82 | 82 | ["callable","100000"] |
| errors_map_ok | 1 | 258.0 | 258 | 258 | ["ok-map","1000000"] |
| errors_case_ok | 1 | 168.0 | 168 | 168 | ["ok-case","1000000"] |
| errors_map_error | 1 | 766.0 | 766 | 766 | ["error-map","1000000"] |
| errors_case_error | 1 | 618.0 | 618 | 618 | ["error-case","1000000"] |
| errors_constructor_error | 1 | 699.0 | 699 | 699 | ["error-constructor","1000000"] |
| errors_cause_message | 1 | 1.0 | 1 | 1 | ["1000","message"] |
| errors_cause_console | 1 | 432.0 | 432 | 432 | ["1000","console"] |
| errors_deep_image | 1 | 1.0 | 1 | 1 | ["8","deep"] |

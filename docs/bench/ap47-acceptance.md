# AP47.7 integrated acceptance measurements

The control is completed AP47.6 (`328c5c71b`); the after version includes the
captured-generic-error closure fix and AP47.7 acceptance work. Both use Rust
1.99.1 and the repository release profile (fat LTO, one codegen unit). Three
full-suite baselines were saved before changing the compiler; three final
full-suite runs followed verification. All original 72 IDs, arguments, timeouts
and groups remained unchanged. Only after this comparison and its isolated
controls were the 13 error rows registered, bringing the suite to 85 rows.

Each version used its own prebuilt release benchmark harness. The harness
obtained the release CLI from Cargo before timed rows; no unrelated builds or
tests overlapped measurements. Isolated controls use the saved pre-fix release
CLI and harness versus the final release binaries, with identical unchanged
sources, library and working directory. Eight pairs alternate which version
runs first. Preparation, fixture checks and cleanup retain the boundaries in
[the benchmark reference](README.md) and existing workload descriptions.

All workload checks passed. Values below are milliseconds; totals are sums of
row medians, not whole-suite wall time. The VM gate covers repeatable increases
over 5% for unchanged rows whose baseline median is at least 20 ms. Shorter
rows retain raw samples. These cloud samples support a regression check within
the measured boundaries, without establishing a general speedup.

The earlier runtime/language and migration comparisons remain recorded in
[AP47.2](history.md#2026-10-19--ap472-native-error-metadata-and-portable-artifacts),
[AP47.3](ap47-language.md), [AP47.4](ap47-maperror.md),
[AP47.5](ap47-host-parsing.md) and [AP47.6](ap47-tasks-network.md).

## Original suite: failure-dump

Each row internally pairs disabled/enabled capture. Its elapsed value covers the
complete workload; it is not the capture overhead percentage.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `failure_dump_scalar` | 354/339/323 | 337/296/292 | 339/296 | -12.68% |
| `failure_dump_calls` | 381/402/369 | 384/415/347 | 381/384 | +0.79% |
| `failure_dump_globals` | 767/762/771 | 755/1188/1089 | 767/1089 | +41.98% |
| `failure_dump_spawn_wait` | 2475/2180/2046 | 2206/2257/2158 | 2180/2206 | +1.19% |
| `failure_dump_suspension` | 1343/1395/1462 | 1417/1486/1339 | 1395/1417 | +1.58% |
| `failure_dump_failure` | 108/115/159 | 129/154/105 | 115/129 | +12.17% |
| `failure_dump_large_report` | 745/734/832 | 780/790/688 | 745/780 | +4.70% |

Sum of row medians: 5922 → 6301 ms (+6.40%).

## Original suite: mandelbrot

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `mandelbrot_render` | 1270/1269/1252 | 1671/1353/1209 | 1269/1353 | +6.62% |
| `mandelbrot_paint` | 1297/1265/1241 | 1415/1394/1250 | 1265/1394 | +10.20% |

Sum of row medians: 2534 → 2747 ms (+8.41%).

## Original suite: vm

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 1319/1274/1378 | 1182/1239/1144 | 1319/1182 | -10.39% |
| `typed_real` | 247/250/257 | 236/408/234 | 250/236 | -5.60% |
| `typed_boolean` | 366/374/366 | 343/391/351 | 366/351 | -4.10% |
| `typed_string` | 235/247/238 | 207/226/201 | 238/207 | -13.03% |
| `global_access` | 185/179/188 | 171/217/171 | 185/171 | -7.57% |
| `record_field_access` | 674/606/578 | 592/600/583 | 606/592 | -2.31% |
| `closure_call` | 397/298/253 | 243/247/223 | 298/243 | -18.46% |
| `mutable_capture` | 353/308/300 | 317/313/296 | 308/313 | +1.62% |
| `branch_dispatch` | 683/613/635 | 552/743/569 | 635/569 | -10.39% |
| `dynamic_numeric` | 536/612/526 | 680/663/487 | 536/663 | +23.69% |
| `array_push` | 86/90/91 | 144/89/91 | 90/91 | +1.11% |
| `array_length` | 34/38/35 | 72/47/36 | 35/47 | +34.29% |
| `string_concat` | 1519/1770/1868 | 2183/1757/1513 | 1770/1757 | -0.73% |
| `string_length` | 34/33/87 | 36/37/33 | 34/36 | +5.88% |
| `intrinsic_dispatch` | 992/977/1091 | 1172/1051/980 | 992/1051 | +5.95% |
| `function_call` | 452/437/465 | 461/467/434 | 452/461 | +1.99% |
| `array_callbacks` | 1378/1365/1403 | 1516/1444/1429 | 1378/1444 | +4.79% |
| `record_update` | 152/189/154 | 153/158/154 | 154/154 | +0.00% |
| `unicode_char_at` | 431/432/432 | 444/462/433 | 432/444 | +2.78% |
| `wrapper_payload` | 1038/1035/1074 | 1047/1468/1080 | 1038/1080 | +4.05% |
| `string_search` | 117/120/133 | 114/134/112 | 120/114 | -5.00% |
| `substring_ascii` | 10/9/9 | 9/9/9 | 9/9 | +0.00% |
| `substring_unicode` | 38/21/20 | 23/19/20 | 21/20 | -4.76% |

Sum of row medians: 11266 → 11235 ms (-0.28%).

## Original suite: concurrency

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `task_spawn_wait` | 813/643/720 | 841/793/477 | 720/793 | +10.14% |
| `task_array_callbacks` | 1557/1544/1813 | 1503/1506/1499 | 1557/1503 | -3.47% |

Sum of row medians: 2277 → 2296 ms (+0.83%).

## Original suite: tui

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `tui_headless` | 2770/2882/2771 | 3289/2762/2327 | 2771/2762 | -0.32% |
| `notes_headless` | 4687/5210/4910 | 5001/5317/4600 | 4910/5001 | +1.85% |
| `tui_styled_cells` | 590/593/607 | 562/732/552 | 593/562 | -5.23% |

Sum of row medians: 8274 → 8325 ms (+0.62%).

## Original suite: tooling

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `analysis_queries` | 30/32/62 | 31/66/29 | 32/31 | -3.12% |
| `compiler_lowering` | 2424/2519/2902 | 2543/3778/2395 | 2519/2543 | +0.95% |
| `project_queries` | 165/131/168 | 140/137/130 | 165/137 | -16.97% |
| `project_edits` | 747/780/948 | 831/627/658 | 780/658 | -15.64% |
| `project_overlapping_queries` | 96/256/219 | 100/94/93 | 219/94 | -57.08% |

Sum of row medians: 3715 → 3463 ms (-6.78%).

## Original suite: startup

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `project_build_cold` | 2195/2342/2474 | 2307/2327/2118 | 2342/2307 | -1.49% |
| `project_build_warm` | 1017/877/919 | 1442/1202/842 | 919/1202 | +30.79% |
| `unit_artifact_shared_types` | 1/5/1 | 3/1/1 | 1/1 | +0.00% |
| `program_artifact_shared_types` | 0/0/2 | 0/0/1 | 0/0 | n/a (0 ms baseline) |

Sum of row medians: 3262 → 3510 ms (+7.60%).

## Original suite: review

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `dictionary_reads` | 5/6/5 | 11/6/5 | 5/6 | +20.00% |
| `scalar_membership` | 8/8/9 | 10/11/10 | 8/10 | +25.00% |
| `array_pop` | 1/2/1 | 5/2/2 | 1/2 | +100.00% |
| `text_area_locate` | 1505/1635/1513 | 1650/1433/1370 | 1513/1433 | -5.29% |
| `http_body_accumulation` | 202/196/195 | 202/199/196 | 196/199 | +1.53% |

Sum of row medians: 1723 → 1650 ms (-4.24%).

## Original suite: local-index

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `local_index_write` | 25/27/28 | 29/27/25 | 27/27 | +0.00% |
| `cell_grid_headless` | 162/158/161 | 208/156/155 | 161/156 | -3.11% |
| `partial_surface_fill` | 93/99/95 | 98/102/106 | 95/102 | +7.37% |

Sum of row medians: 283 → 285 ms (+0.71%).

## Original suite: source-review

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_utf8_ascii_2048` | 13/12/14 | 12/11/12 | 13/12 | -7.69% |
| `source_utf8_ascii_4096` | 28/25/23 | 23/24/27 | 25/24 | -4.00% |
| `source_utf8_ascii_8192` | 49/53/49 | 47/47/54 | 49/47 | -4.08% |
| `source_utf8_unicode_2048` | 42/44/45 | 42/43/46 | 44/43 | -2.27% |
| `source_utf8_unicode_4096` | 94/86/87 | 83/85/84 | 87/84 | -3.45% |
| `source_utf8_unicode_8192` | 172/185/183 | 181/169/170 | 183/170 | -7.10% |
| `source_notes_ascii_128` | 5/5/5 | 5/5/5 | 5/5 | +0.00% |
| `source_notes_ascii_256` | 24/11/12 | 12/12/11 | 12/12 | +0.00% |
| `source_notes_ascii_512` | 25/26/33 | 26/26/26 | 26/26 | +0.00% |
| `source_queue_ascii_256` | 102/101/106 | 106/100/96 | 102/100 | -1.96% |
| `source_queue_ascii_512` | 223/207/223 | 197/213/199 | 223/199 | -10.76% |
| `source_queue_ascii_1024` | 469/421/433 | 421/432/421 | 433/421 | -2.77% |

Sum of row medians: 1202 → 1143 ms (-4.91%).

## Original suite: source-navigation

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_navigation_long_512` | 40/40/40 | 40/39/41 | 40/40 | +0.00% |
| `source_navigation_long_1024` | 78/77/77 | 74/75/73 | 77/74 | -3.90% |
| `source_navigation_long_2048` | 140/150/145 | 145/144/143 | 145/144 | -0.69% |
| `source_navigation_lines_512` | 10/15/10 | 9/11/9 | 10/9 | -10.00% |
| `source_navigation_lines_1024` | 17/16/17 | 15/17/15 | 17/15 | -11.76% |
| `source_navigation_lines_2048` | 27/27/30 | 26/27/26 | 27/26 | -3.70% |

Sum of row medians: 316 → 308 ms (-2.53%).

## Isolated regression controls

Every row with a baseline median of at least 20 ms and a full-suite increase
over 5% received eight alternating pairs. Raw values and medians are retained
below, including the non-VM rows. Isolated medians decide whether an increase
repeats; a lower group total cannot excuse an individual regression.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `failure_dump_globals` | 804/740/750/758/747/753/760/753 | 724/732/737/736/776/737/727/755 | 753/736.5 | -2.19% |
| `failure_dump_failure` | 104/105/104/106/98/105/100/115 | 104/104/111/102/111/105/104/103 | 104.5/104 | -0.48% |
| `mandelbrot_render` | 1272/1074/1053/1098/1249/1134/1064/1246 | 1215/1035/1180/1203/1040/1091/1216/1254 | 1116/1191.5 | +6.77% |
| `mandelbrot_paint` | 1202/1223/1098/1256/1108/1180/1099/1249 | 1239/1100/1258/1268/1280/1263/1238/1096 | 1191/1248.5 | +4.83% |
| `dynamic_numeric` | 536/516/524/542/554/524/507/513 | 495/487/487/489/492/476/472/480 | 524/487 | -7.06% |
| `array_length` | 34/33/33/36/35/33/34/35 | 33/34/33/35/33/40/37/38 | 34/34.5 | +1.47% |
| `string_length` | 39/35/34/33/33/35/33/33 | 37/32/34/32/34/36/33/33 | 33.5/33.5 | +0.00% |
| `intrinsic_dispatch` | 981/989/953/966/953/988/983/981 | 1003/974/987/974/964/959/979/948 | 981/974 | -0.71% |
| `task_spawn_wait` | 820/824/771/797/774/784/693/773 | 763/788/829/813/790/691/804/796 | 779/793 | +1.80% |
| `project_build_warm` | 843/805/843/927/1047/832/921/851 | 829/820/822/829/1096/969/942/843 | 847/836 | -1.30% |
| `partial_surface_fill` | 99/98/91/93/172/181/99/94 | 102/91/88/102/96/98/89/96 | 98.5/96 | -2.54% |

No repeatable VM regression crossed the selected gate. The non-VM Mandelbrot
render row remains an exception: its first eight isolated pairs measure +6.77%.
Sixteen additional alternating pairs measure +6.41%; all 24 pooled medians are
1154/1238.5 ms (+7.32%). The median of the 24 individual paired percentage changes
is +4.20%, with 15 positive pairs. This is reported separately rather than hidden
by a group total or labeled a speedup. Other isolated increases stay below 5%.

| Additional render control | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| 16 alternating pairs | 1099/1079/1246/1262/1255/1263/1141/1141/1108/1167/1048/1047/1221/1220/1259/1355 | 1282/1266/2064/1274/1209/1219/1394/1281/1091/1254/1707/1266/1223/1114/1841/1312 | 1193.5/1270 | +6.41% |


To investigate this exception, both CLIs built an owned copy of the real
Mandelbrot project. The two 1291491-byte images have byte-identical instruction,
constant, string, record, enum, source-map, entry and distinct-type sections.
All 562 functions have identical runtime fields, including code ranges, arity,
captures, registers and task flags. After canonical type-ID normalization, all
record/enum field types, globals, function debug bindings/results/captures and
attached projections are also equal: the remaining metadata difference is type-ID ordering. Normal calls,
closure construction, Spawn/TaskState and globals do not read these debugger
types. No extra semantic rendering work or per-instruction polling was found.

Eight mirrored-order 2×2 rounds run each image through each saved CLI. The timer
still covers the same twelve production renders; admission is outside it.

| CLI / image | Raw samples | Median ms |
| --- | --- | --- |
| `before-before` | 1105/1179/1223/1584/1144/1070/2030/1067 | 1161.5 |
| `before-after` | 1263/1380/1076/1086/1252/1101/1241/1253 | 1246.5 |
| `after-before` | 1089/1219/1225/1240/1067/1038/1085/1068 | 1087 |
| `after-after` | 1264/1244/1081/1279/1246/1237/1227/1241 | 1242.5 |


Here the newer CLI is faster/equal for a fixed image, while the new-image path
correlates with higher medians on both binaries. That correlation does not
establish a cause. A second eight-round A/A matrix uses identical copies of the
new image at both paths, with the same mirrored order and checksum:

| CLI / identical image path | Raw samples | Median ms |
| --- | --- | --- |
| `before-before` | 1150/1189/1237/1060/1051/1089/1203/1211 | 1169.5 |
| `before-after` | 1076/1165/1243/1203/1068/1068/1067/1209 | 1120.5 |
| `after-before` | 1076/1122/1039/1193/1063/1090/1193/1029 | 1083 |
| `after-after` | 1096/1219/1020/1189/1076/1060/1040/1040 | 1068 |


The A/A control bounds the interpretation of path/environment scatter; it does
not replace the original positive render measurements. Cache, allocation and
scheduler interactions remain hypotheses. The explicit hard acceptance gate
covers the unchanged `vm` group, whose flagged rows passed isolated controls.
The render figures remain a limitation of this cloud acceptance record; no
runtime tradeoff or performance improvement is claimed from them.

## New errors group: same-build controls

These features have no invented old-checkout baseline. Three serial samples use
one final release build, with the fixed IDs and arguments in
[suite.toml](suite.toml). [errors.md](errors.md) gives each timer boundary,
warmup, exact checksum and matched control. Each source row constructs/checks
100000 values, or 1000000 Results for MapError/case rows. Rendering covers 1000
outputs at depth 16. The deep image row performs eight real production decodes
and verifies a typed depth-512 terminal cause after each decode; its timer is
structural tracking, not a wall-clock correctness assertion.

| Workload | Samples | Median |
| --- | --- | --- |
| `errors_enum_construction` | 45/37/37 | 37 |
| `errors_typed_construction` | 49/39/40 | 40 |
| `errors_generic_direct` | 45/43/44 | 44 |
| `errors_generic_helper` | 65/64/64 | 64 |
| `errors_generic_callable` | 73/66/66 | 66 |
| `errors_map_ok` | 379/366/353 | 366 |
| `errors_case_ok` | 391/390/329 | 390 |
| `errors_map_error` | 1140/1149/1152 | 1149 |
| `errors_case_error` | 1374/1117/1049 | 1117 |
| `errors_constructor_error` | 997/1008/951 | 997 |
| `errors_cause_message` | 1/2/1 | 1 |
| `errors_cause_console` | 543/647/523 | 543 |
| `errors_deep_image` | 2/2/2 | 2 |

Matched ratios use medians from these same-build rows:

| Comparison | Ratio |
| --- | --- |
| Typed / ordinary enum construction | 1.081× |
| Generic helper / direct constructor callable | 1.455× |
| Generic helper with constructor callable / direct callable | 1.500× |
| MapError Ok / explicit case Ok | 0.938× |
| MapError function Error / explicit case Error | 1.029× |
| MapError constructor Error / MapError function Error | 0.868× |

Message samples are only 1–2 ms at the required 1000 renders, so no precise
Console/Message ratio is inferred. Console capture/bookkeeping is a distinct
combined workload. These ratios include loop/control-flow, payload extraction
and checksum work;
they do not isolate allocation or native-call nanoseconds. Generic comparisons
include hidden-constructor binding and helper call overhead. Ordinary function
mappers are counted: zero for Ok and exactly N for Error. Constructor rows
validate typed results without instrumenting constructors. Console includes real
in-memory capture/bookkeeping; complete captured output is checked outside the
timer and no rendered text is sent to a terminal. Message and Console use the
same production renderer.

The exact construction fixture's metadata diagnostic reports:

```text
value_bytes: 16
concrete_instances: 2
shared_factories: 2
factory_instruction_bytes: 48
error_metadata_json_bytes: 861
construction_image_bytes: 27448
```

Two concrete instances share two constructor factories across all direct,
generic and callable uses. Factory size is instruction storage; the JSON size
covers serialized error identity/variant descriptions, not heap usage or general
debug tables. The portable image size includes the whole construction program.
The 16-byte Value representation is unchanged.

## Fresh expanded-suite baseline

After registration, `cargo bench-fpas save ap47-7-expanded-full` completed all 85
rows with group selection `all`. Its 13 feature rows are an additional sample,
not an old/new feature comparison. The baseline is local under
`.temp-data/bench/ap47-7-expanded-full.json`; future runs must use the same IDs,
arguments and selection. A 72-row snapshot is intentionally rejected against the
85-row suite. Group/row validation was not weakened.

| Workload | Baseline ms |
| --- | --- |
| `failure_dump_scalar` | 290 |
| `failure_dump_calls` | 361 |
| `failure_dump_globals` | 718 |
| `failure_dump_spawn_wait` | 2166 |
| `failure_dump_suspension` | 1323 |
| `failure_dump_failure` | 104 |
| `failure_dump_large_report` | 690 |
| `mandelbrot_render` | 1207 |
| `mandelbrot_paint` | 1319 |
| `integer_loop` | 1134 |
| `typed_real` | 240 |
| `typed_boolean` | 350 |
| `typed_string` | 200 |
| `global_access` | 167 |
| `record_field_access` | 552 |
| `closure_call` | 222 |
| `mutable_capture` | 299 |
| `branch_dispatch` | 532 |
| `dynamic_numeric` | 491 |
| `array_push` | 85 |
| `array_length` | 35 |
| `string_concat` | 1436 |
| `string_length` | 35 |
| `intrinsic_dispatch` | 961 |
| `function_call` | 408 |
| `array_callbacks` | 1319 |
| `record_update` | 147 |
| `unicode_char_at` | 416 |
| `wrapper_payload` | 1019 |
| `task_spawn_wait` | 786 |
| `task_array_callbacks` | 1933 |
| `tui_headless` | 2336 |
| `notes_headless` | 4741 |
| `tui_styled_cells` | 558 |
| `analysis_queries` | 29 |
| `string_search` | 114 |
| `compiler_lowering` | 2357 |
| `substring_ascii` | 9 |
| `substring_unicode` | 19 |
| `project_queries` | 128 |
| `project_edits` | 616 |
| `project_overlapping_queries` | 119 |
| `project_build_cold` | 2080 |
| `project_build_warm` | 813 |
| `unit_artifact_shared_types` | 1 |
| `program_artifact_shared_types` | 0 |
| `dictionary_reads` | 6 |
| `scalar_membership` | 9 |
| `array_pop` | 2 |
| `text_area_locate` | 1322 |
| `http_body_accumulation` | 190 |
| `local_index_write` | 26 |
| `cell_grid_headless` | 156 |
| `partial_surface_fill` | 88 |
| `source_utf8_ascii_2048` | 11 |
| `source_utf8_ascii_4096` | 23 |
| `source_utf8_ascii_8192` | 48 |
| `source_utf8_unicode_2048` | 41 |
| `source_utf8_unicode_4096` | 82 |
| `source_utf8_unicode_8192` | 173 |
| `source_notes_ascii_128` | 5 |
| `source_notes_ascii_256` | 11 |
| `source_notes_ascii_512` | 25 |
| `source_queue_ascii_256` | 97 |
| `source_queue_ascii_512` | 197 |
| `source_queue_ascii_1024` | 384 |
| `source_navigation_long_512` | 41 |
| `source_navigation_long_1024` | 73 |
| `source_navigation_long_2048` | 145 |
| `source_navigation_lines_512` | 11 |
| `source_navigation_lines_1024` | 15 |
| `source_navigation_lines_2048` | 26 |
| `errors_enum_construction` | 38 |
| `errors_typed_construction` | 41 |
| `errors_generic_direct` | 42 |
| `errors_generic_helper` | 59 |
| `errors_generic_callable` | 66 |
| `errors_map_ok` | 346 |
| `errors_case_ok` | 312 |
| `errors_map_error` | 1072 |
| `errors_case_error` | 1072 |
| `errors_constructor_error` | 997 |
| `errors_cause_message` | 2 |
| `errors_cause_console` | 536 |
| `errors_deep_image` | 2 |

Local original evidence is named `ap47-7-before-full-1..3` and
`ap47-7-after-full-1..3`; new group snapshots are `ap47-7-errors-1..3`.
All tables preserve their raw timer samples for review; no machine-identifying
metadata is published.

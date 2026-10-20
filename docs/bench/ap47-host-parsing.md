# AP47.5 host and parsing regression measurements

The unchanged pre-migration source archive at `a63232a0f` is the control; the
comparison is the AP47.5 implementation. Both use Rust 1.99.1 and the repository
release profile (fat LTO, one codegen unit). All 72 original suite IDs, arguments,
timeouts and group selections are unchanged. Three full-suite pairs ran serially
in before/after order, with no unrelated builds or tests during measured work.
Each tree used its own prebuilt benchmark harness; the harness still obtained its
release CLI from Cargo before timing. Build preparation finished before the pairs.

All runs completed successfully with the workloads' existing result checks.
These rows guard existing runtime, host, compiler and editor behavior; they do
not isolate typed error construction or rendering. Dedicated error workloads
remain scheduled after AP47.6's original-suite comparison. Cloud measurements
support a regression check within these boundaries, not a general speedup claim.

All values are milliseconds. Each table retains all three samples, the medians
and their relative change. Group totals sum row medians rather than measuring
whole-suite wall time. The VM gate applies to repeatable increases above 5% when
the control median is at least 20 ms; shorter rows retain their raw values.

## failure-dump

Each row internally pairs disabled/enabled failure capture; the reported elapsed value is the complete workload timer, not the capture overhead percentage. Raw snapshots retain both internal sample arrays and their medians.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `failure_dump_scalar` | 309/440/334 | 341/303/313 | 334/313 | -6.29% |
| `failure_dump_calls` | 363/398/359 | 372/378/379 | 363/378 | +4.13% |
| `failure_dump_globals` | 767/811/781 | 769/765/793 | 781/769 | -1.54% |
| `failure_dump_spawn_wait` | 2231/2128/2357 | 2283/2058/2085 | 2231/2085 | -6.54% |
| `failure_dump_suspension` | 1326/1311/1324 | 1360/1358/1339 | 1324/1358 | +2.57% |
| `failure_dump_failure` | 105/106/101 | 110/106/107 | 105/107 | +1.90% |
| `failure_dump_large_report` | 702/682/670 | 689/702/734 | 682/702 | +2.93% |

Sum of row medians: 5820 → 5712 ms (-1.86%).

## mandelbrot

Render/paint rows cover the existing task renderer and deterministic headless host, with their existing warmup and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `mandelbrot_render` | 1187/1156/1040 | 1194/1226/1221 | 1156/1221 | +5.62% |
| `mandelbrot_paint` | 1092/1119/1271 | 1238/1223/1281 | 1119/1238 | +10.63% |

Sum of row medians: 2275 → 2459 ms (+8.09%).

## vm

These unchanged interpreter workloads cover scalar execution, calls, callbacks, containers and strings.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 1226/1275/1357 | 1241/1268/1219 | 1275/1241 | -2.67% |
| `typed_real` | 242/356/247 | 233/251/232 | 247/233 | -5.67% |
| `typed_boolean` | 360/363/360 | 364/387/371 | 360/371 | +3.06% |
| `typed_string` | 214/212/223 | 214/216/215 | 214/215 | +0.47% |
| `global_access` | 203/181/189 | 184/197/229 | 189/197 | +4.23% |
| `record_field_access` | 582/548/570 | 549/555/598 | 570/555 | -2.63% |
| `closure_call` | 246/243/255 | 245/249/250 | 246/249 | +1.22% |
| `mutable_capture` | 294/295/307 | 298/294/304 | 295/298 | +1.02% |
| `branch_dispatch` | 616/615/618 | 642/616/620 | 616/620 | +0.65% |
| `dynamic_numeric` | 529/538/548 | 555/551/539 | 538/551 | +2.42% |
| `array_push` | 86/87/92 | 89/88/91 | 87/89 | +2.30% |
| `array_length` | 34/37/34 | 33/34/33 | 34/33 | -2.94% |
| `string_concat` | 1418/1462/1499 | 1456/1462/1450 | 1462/1456 | -0.41% |
| `string_length` | 33/33/33 | 33/39/33 | 33/33 | +0.00% |
| `intrinsic_dispatch` | 954/980/967 | 991/1011/1094 | 967/1011 | +4.55% |
| `function_call` | 453/457/459 | 462/478/462 | 457/462 | +1.09% |
| `array_callbacks` | 1398/1324/1376 | 1356/1370/1345 | 1376/1356 | -1.45% |
| `record_update` | 148/147/150 | 147/153/155 | 148/153 | +3.38% |
| `unicode_char_at` | 418/416/434 | 417/430/438 | 418/430 | +2.87% |
| `wrapper_payload` | 1037/1048/1120 | 1032/1062/1024 | 1048/1032 | -1.53% |
| `string_search` | 130/120/115 | 111/112/114 | 120/112 | -6.67% |
| `substring_ascii` | 9/9/9 | 9/11/10 | 9/10 | +11.11% |
| `substring_unicode` | 19/19/19 | 20/24/19 | 19/20 | +5.26% |

Sum of row medians: 10728 → 10727 ms (-0.01%).

## concurrency

These rows cover task spawning/waiting and task array callbacks, preserving existing ownership and result checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `task_spawn_wait` | 766/786/814 | 656/804/795 | 786/795 | +1.15% |
| `task_array_callbacks` | 1683/1546/1917 | 1508/1674/1522 | 1683/1522 | -9.57% |

Sum of row medians: 2469 → 2317 ms (-6.16%).

## tui

These rows cover the headless TUI and application frames; they exclude interactive terminal latency.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `tui_headless` | 2426/2371/2943 | 2413/2353/2533 | 2426/2413 | -0.54% |
| `notes_headless` | 4495/4709/4704 | 4614/4647/4862 | 4704/4647 | -1.21% |
| `tui_styled_cells` | 564/590/568 | 566/569/564 | 568/566 | -0.35% |

Sum of row medians: 7698 → 7626 ms (-0.94%).

## tooling

Native harness child processes measure semantic analysis, compiler lowering and real source-standard-library editor queries. Parsing/fixture preparation and warmups remain outside each workload timer.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `analysis_queries` | 37/37/35 | 29/29/29 | 37/29 | -21.62% |
| `compiler_lowering` | 2500/2522/2346 | 2450/2541/2343 | 2500/2450 | -2.00% |
| `project_queries` | 164/155/149 | 132/136/126 | 155/132 | -14.84% |
| `project_edits` | 583/624/865 | 598/613/606 | 624/606 | -2.88% |
| `project_overlapping_queries` | 97/109/110 | 92/120/94 | 109/94 | -13.76% |

Sum of row medians: 3425 → 3311 ms (-3.33%).

## startup

Project cold/warm builds include source validation, artifact loading/admission and publication. The artifact rows separately measure repeated production object/image decoding, validation and destruction; fixture construction is excluded.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `project_build_cold` | 2039/2115/2083 | 2063/2241/2072 | 2083/2072 | -0.53% |
| `project_build_warm` | 814/849/838 | 798/851/802 | 838/802 | -4.30% |
| `unit_artifact_shared_types` | 1/1/1 | 1/1/1 | 1/1 | +0.00% |
| `program_artifact_shared_types` | 0/0/0 | 1/0/0 | 0/0 | n/a (zero control) |

Sum of row medians: 2922 → 2875 ms (-1.61%).

## review

The existing dictionary, membership, array drain, text-area and local HTTP rows retain their original boundaries. HTTP includes transport/body processing and excludes compilation.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `dictionary_reads` | 5/6/6 | 5/5/6 | 6/5 | -16.67% |
| `scalar_membership` | 56/61/58 | 10/10/9 | 58/10 | -82.76% |
| `array_pop` | 2/1/1 | 1/1/2 | 1/1 | +0.00% |
| `text_area_locate` | 1502/1349/1330 | 1398/1365/1336 | 1349/1365 | +1.19% |
| `http_body_accumulation` | 205/207/203 | 191/207/187 | 205/191 | -6.83% |

Sum of row medians: 1619 → 1572 ms (-2.90%).

## local-index

Local writes and headless grid/partial-surface painting retain existing checksums/snapshots and untimed setup.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `local_index_write` | 29/26/26 | 28/25/26 | 26/26 | +0.00% |
| `cell_grid_headless` | 151/156/153 | 152/153/160 | 153/153 | +0.00% |
| `partial_surface_fill` | 91/88/88 | 89/93/89 | 88/89 | +1.14% |

Sum of row medians: 267 → 268 ms (+0.37%).

## source-review

Source UTF-8, Notes and queue helpers run the existing source-level workloads with their original inputs and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_utf8_ascii_2048` | 11/11/11 | 11/11/11 | 11/11 | +0.00% |
| `source_utf8_ascii_4096` | 25/23/23 | 23/23/23 | 23/23 | +0.00% |
| `source_utf8_ascii_8192` | 46/48/46 | 46/47/48 | 46/47 | +2.17% |
| `source_utf8_unicode_2048` | 45/43/45 | 42/44/44 | 45/44 | -2.22% |
| `source_utf8_unicode_4096` | 87/84/86 | 85/88/87 | 86/87 | +1.16% |
| `source_utf8_unicode_8192` | 183/177/182 | 175/175/172 | 182/175 | -3.85% |
| `source_notes_ascii_128` | 5/5/5 | 5/5/5 | 5/5 | +0.00% |
| `source_notes_ascii_256` | 11/11/11 | 12/11/11 | 11/11 | +0.00% |
| `source_notes_ascii_512` | 25/24/24 | 25/25/26 | 24/25 | +4.17% |
| `source_queue_ascii_256` | 97/99/98 | 101/102/101 | 98/101 | +3.06% |
| `source_queue_ascii_512` | 197/195/206 | 200/197/194 | 197/197 | +0.00% |
| `source_queue_ascii_1024` | 397/395/389 | 404/403/408 | 395/404 | +2.28% |

Sum of row medians: 1123 → 1130 ms (+0.62%).

## source-navigation

Navigation rows retain the existing long-line and multi-line source workloads and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_navigation_long_512` | 39/41/40 | 40/40/40 | 40/40 | +0.00% |
| `source_navigation_long_1024` | 74/73/77 | 74/71/75 | 74/74 | +0.00% |
| `source_navigation_long_2048` | 145/142/139 | 138/141/141 | 142/141 | -0.70% |
| `source_navigation_lines_512` | 9/10/10 | 9/11/10 | 10/10 | +0.00% |
| `source_navigation_lines_1024` | 15/15/15 | 15/15/15 | 15/15 | +0.00% |
| `source_navigation_lines_2048` | 28/26/29 | 25/26/28 | 28/26 | -7.14% |

Sum of row medians: 309 → 306 ms (-0.97%).

## Isolated regression controls

Rows whose full-suite medians rose above 5% with a control of at least 20 ms
were repeated in eight isolated pairs, alternating before/after and after/before
order. Each used the exact suite driver, arguments, internal warmup, timer and
existing result checks. No builds or other tests ran alongside these pairs.
Raw output and result checks are retained with the local control samples.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `mandelbrot_render` | 1217/1078/1202/1045/1219/1238/1042/1226 | 1216/1180/1018/1215/1199/1032/1203/1079 | 1209.5/1189.5 | -1.65% |
| `mandelbrot_paint` | 1101/1077/1265/1084/1095/1251/1110/1268 | 1173/1241/1065/1226/1205/1285/1249/1074 | 1105.5/1215.5 | +9.95% |
| `mandelbrot_paint`, 12 repeat pairs | 1095/1265/1245/1181/1105/1230/1269/1094/1094/1307/1247/1273 | 1034/1125/1059/1062/1255/1064/1061/1239/1267/1075/1243/1211 | 1237.5/1100 | -11.11% |

The full-suite VM sum is 10728 → 10727 ms (-0.01%); no VM row with a
control median of at least 20 ms exceeds 5%. The only full-suite increases above
5% at that size were the two Mandelbrot rows. Render did not repeat its increase.
Paint's first eight pairs rose 9.95%, but twelve more alternating pairs reversed
the result to -11.11%; all twenty pairs together have medians 1205.5 → 1189 ms
(-1.37%). Their strongly scattered samples do not establish a repeatable
slowdown or speedup. No build or test ran alongside either control series.
The repaired scalar membership row is 58 → 10 ms (-82.76%) in the final full
suite; that result applies to this exact Integer array workload.

## Regression found and repaired

The first AP47.5 build reproducibly slowed the unchanged `scalar_membership`
row: full-suite samples were 54/57/54 → 64/60/66 ms (medians 54 → 64,
+18.52%). Its eight isolated pairs and two subsequent diagnostics are retained
below. Inspected equality/membership machine functions had unchanged sizes;
selected equality instructions matched apart from relocations, while addresses
shifted. No new native-error admission work runs inside this timer. Binary layout
sensitivity is plausible, but the exact cause was not established.

The final implementation dispatches an Integer array needle once and compares
Integer elements directly, preserving exact value/type equality. Other needles,
dictionaries, strings and structural comparisons keep their existing route.
The regression test covers heterogeneous, nested and empty arrays plus integer
extrema. The final full-suite tables above include this repair and its new
release build, rather than mixing measurements from two implementations.

| Initial-build diagnostic | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| Scalar, 8 isolated pairs | 55/58/55/56/55/56/56/57 | 59/61/66/60/62/69/65/61 | 56/61.5 | +9.82% |
| Scalar, 12 repeat pairs | 62/55/61/56/60/55/55/56/55/56/56/53 | 63/72/64/61/61/56/66/62/61/65/62/61 | 56/62 | +10.71% |
| Scalar, 12 pairs pinned to one eligible CPU | 54/59/56/58/59/59/62/60/59/56/56/58 | 63/62/63/65/64/62/63/63/70/63/66/64 | 58.5/63 | +7.69% |
| Task callbacks, 8 isolated pairs | 1630/1496/1565/1788/1508/1519/1814/1505 | 1755/1496/2182/1441/2004/1556/1895/1980 | 1542/1825 | +18.35% |
| Task callbacks, 12 repeat pairs | 2087/1521/1498/1616/1494/1975/1549/1505/1475/1594/2256/1612 | 1501/1526/1489/1701/1968/1539/1591/1841/1536/1499/1509/1508 | 1571.5/1531 | -2.58% |

The pinned scalar diagnostic changes only process scheduling affinity for both
versions; it is separate from the primary inherited-affinity comparison. The
initial callback increase did not repeat in twelve additional alternating pairs;
its scatter and contrary repeat are shown instead of treating the initial median
as an established regression. All diagnostics kept the suite's exact workload
arguments and existing checks. Initial snapshots have an additional `initial-`
filename component; diagnostic JSON files are retained as
`.temp-data/ap47.5-initial-{bench-summary,isolated-controls,followup-controls}.json`.

## Reproduction and retained evidence

Run `cargo bench-fpas save <label>` from each selected tree, without a group
filter, three times. Reusing each tree's built harness with the same `save`
arguments avoids toggling Cargo dependency features; it still builds/selects the
release CLI. Do not run builds/tests in parallel with measured rows. Isolated
controls execute the corresponding CLI/native driver with the suite's exact
arguments and alternate version order.

The six snapshots are named `ap47-5-before-full-<1..3>.json` in the control and
`ap47-5-after-full-<1..3>.json` in the working tree, under `.temp-data/bench/`.
They retain each workload's stdout. The working tree also retains
`.temp-data/ap47.5-bench-summary.json`,
`.temp-data/ap47.5-isolated-controls.json` and
`.temp-data/ap47.5-final-followup-controls.json`, including isolated stdout and checks.
The committed tables preserve all before/after timer samples without machine
identifiers. The suite itself has not gained an `errors` group.

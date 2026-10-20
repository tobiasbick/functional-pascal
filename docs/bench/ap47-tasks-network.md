# AP47.6 task and network regression measurements

The pre-migration version is `be512d862`; the comparison is the AP47.6
implementation. An unchanged source archive supplies the isolated controls. Both use Rust 1.99.1 and the repository
release profile (fat LTO, one codegen unit). All 72 original suite IDs, arguments,
timeouts and group selections are unchanged. Three full-suite baselines ran before
implementation; three final runs followed serially after verification, with no
unrelated builds or tests during measured work. Each version used its own prebuilt
benchmark harness; the harness still obtained its release CLI from Cargo before
timing. Release CLI preparation finished before each timed suite.

All runs completed successfully with the workloads' existing result checks.
These rows guard existing runtime, host, compiler and editor behavior; they do
not isolate typed error construction or rendering. Dedicated error workloads
remain in the planned AP47.7 acceptance package after this original-suite
comparison. Cloud measurements support a regression check within these boundaries,
not a general speedup claim.

All values are milliseconds. Each table retains all three samples, the medians
and their relative change. Group totals sum row medians rather than measuring
whole-suite wall time. The VM gate applies to repeatable increases above 5% when
the control median is at least 20 ms; shorter rows retain their raw values.

## failure-dump

Each row internally pairs disabled/enabled failure capture; the reported elapsed value is the complete workload timer, not the capture overhead percentage. Raw snapshots retain both internal sample arrays and their medians.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `failure_dump_scalar` | 336/331/308 | 431/302/433 | 331/431 | +30.21% |
| `failure_dump_calls` | 397/414/386 | 416/371/390 | 397/390 | -1.76% |
| `failure_dump_globals` | 781/807/1179 | 809/766/776 | 807/776 | -3.84% |
| `failure_dump_spawn_wait` | 2409/2087/2740 | 2220/1948/2141 | 2409/2141 | -11.12% |
| `failure_dump_suspension` | 1331/1370/1452 | 1322/1383/1348 | 1370/1348 | -1.61% |
| `failure_dump_failure` | 109/102/111 | 104/113/104 | 109/104 | -4.59% |
| `failure_dump_large_report` | 710/691/1179 | 732/724/763 | 710/732 | +3.10% |

Sum of row medians: 6133 → 5922 ms (-3.44%).

## mandelbrot

Render/paint rows cover the existing task renderer and deterministic headless host, with their existing warmup and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `mandelbrot_render` | 1224/1056/1243 | 1062/1300/1223 | 1224/1223 | -0.08% |
| `mandelbrot_paint` | 1115/1288/1394 | 1248/1767/1393 | 1288/1393 | +8.15% |

Sum of row medians: 2512 → 2616 ms (+4.14%).

## vm

These unchanged interpreter workloads cover scalar execution, calls, callbacks, containers and strings.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 1273/1279/1608 | 1228/1229/1461 | 1279/1229 | -3.91% |
| `typed_real` | 241/245/236 | 236/236/256 | 241/236 | -2.07% |
| `typed_boolean` | 372/380/381 | 360/352/355 | 380/355 | -6.58% |
| `typed_string` | 221/224/226 | 231/222/213 | 224/222 | -0.89% |
| `global_access` | 191/217/189 | 179/185/186 | 191/185 | -3.14% |
| `record_field_access` | 572/579/564 | 589/574/568 | 572/574 | +0.35% |
| `closure_call` | 247/252/261 | 248/236/251 | 252/248 | -1.59% |
| `mutable_capture` | 299/301/295 | 305/294/308 | 299/305 | +2.01% |
| `branch_dispatch` | 952/644/882 | 617/598/613 | 882/613 | -30.50% |
| `dynamic_numeric` | 699/611/733 | 515/530/534 | 699/530 | -24.18% |
| `array_push` | 197/137/268 | 90/84/90 | 197/90 | -54.31% |
| `array_length` | 98/35/40 | 33/34/35 | 40/34 | -15.00% |
| `string_concat` | 1732/1896/1541 | 1486/1447/1504 | 1732/1486 | -14.20% |
| `string_length` | 38/32/33 | 34/33/36 | 33/34 | +3.03% |
| `intrinsic_dispatch` | 1015/983/1006 | 973/973/985 | 1006/973 | -3.28% |
| `function_call` | 478/479/478 | 440/442/439 | 478/440 | -7.95% |
| `array_callbacks` | 1382/1387/1421 | 1334/1735/1343 | 1387/1343 | -3.17% |
| `record_update` | 150/153/168 | 148/149/150 | 153/149 | -2.61% |
| `unicode_char_at` | 428/431/457 | 415/481/423 | 431/423 | -1.86% |
| `wrapper_payload` | 1056/1050/1196 | 1016/1069/1084 | 1056/1069 | +1.23% |
| `string_search` | 111/114/116 | 117/115/160 | 114/117 | +2.63% |
| `substring_ascii` | 22/11/9 | 9/11/9 | 11/9 | -18.18% |
| `substring_unicode` | 44/21/20 | 19/26/18 | 21/19 | -9.52% |

Sum of row medians: 11678 → 10683 ms (-8.52%).

## concurrency

These rows cover task spawning/waiting and task array callbacks, preserving existing ownership and result checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `task_spawn_wait` | 656/810/724 | 815/676/960 | 724/815 | +12.57% |
| `task_array_callbacks` | 1496/1587/2054 | 1799/1525/1526 | 1587/1526 | -3.84% |

Sum of row medians: 2311 → 2341 ms (+1.30%).

## tui

These rows cover the headless TUI and application frames; they exclude interactive terminal latency.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `tui_headless` | 3177/2856/2674 | 2485/2528/2523 | 2856/2523 | -11.66% |
| `notes_headless` | 4817/4705/4928 | 4664/5249/4688 | 4817/4688 | -2.68% |
| `tui_styled_cells` | 601/609/580 | 605/585/576 | 601/585 | -2.66% |

Sum of row medians: 8274 → 7796 ms (-5.78%).

## tooling

Native harness child processes measure semantic analysis, compiler lowering and real source-standard-library editor queries. Parsing/fixture preparation and warmups remain outside each workload timer.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `analysis_queries` | 31/30/29 | 36/29/30 | 30/30 | +0.00% |
| `compiler_lowering` | 2702/2472/2752 | 2545/2362/2609 | 2702/2545 | -5.81% |
| `project_queries` | 161/140/216 | 134/127/138 | 161/134 | -16.77% |
| `project_edits` | 666/658/927 | 656/649/701 | 666/656 | -1.50% |
| `project_overlapping_queries` | 98/468/112 | 105/96/95 | 112/96 | -14.29% |

Sum of row medians: 3671 → 3461 ms (-5.72%).

## startup

Project cold/warm builds include source validation, artifact loading/admission and publication. The artifact rows separately measure repeated production object/image decoding, validation and destruction; fixture construction is excluded.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `project_build_cold` | 2141/2469/3226 | 2805/2255/2146 | 2469/2255 | -8.67% |
| `project_build_warm` | 936/884/1162 | 947/1026/911 | 936/947 | +1.18% |
| `unit_artifact_shared_types` | 2/1/1 | 1/1/1 | 1/1 | +0.00% |
| `program_artifact_shared_types` | 1/0/0 | 0/0/0 | 0/0 | n/a (zero control) |

Sum of row medians: 3406 → 3203 ms (-5.96%).

## review

The existing dictionary, membership, array drain, text-area and local HTTP rows retain their original boundaries. HTTP includes transport/body processing and excludes compilation.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `dictionary_reads` | 6/5/5 | 5/5/5 | 5/5 | +0.00% |
| `scalar_membership` | 20/9/10 | 7/11/7 | 10/7 | -30.00% |
| `array_pop` | 2/2/1 | 2/2/1 | 2/2 | +0.00% |
| `text_area_locate` | 1641/1448/1378 | 1399/1357/1373 | 1448/1373 | -5.18% |
| `http_body_accumulation` | 206/202/202 | 238/192/199 | 202/199 | -1.49% |

Sum of row medians: 1667 → 1586 ms (-4.86%).

## local-index

Local writes and headless grid/partial-surface painting retain existing checksums/snapshots and untimed setup.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `local_index_write` | 27/27/26 | 24/25/25 | 27/25 | -7.41% |
| `cell_grid_headless` | 171/171/153 | 167/153/158 | 171/158 | -7.60% |
| `partial_surface_fill` | 96/90/93 | 91/92/92 | 93/92 | -1.08% |

Sum of row medians: 291 → 275 ms (-5.50%).

## source-review

Source UTF-8, Notes and queue helpers run the existing source-level workloads with their original inputs and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_utf8_ascii_2048` | 14/17/11 | 14/12/12 | 14/12 | -14.29% |
| `source_utf8_ascii_4096` | 26/23/25 | 24/24/25 | 25/24 | -4.00% |
| `source_utf8_ascii_8192` | 48/51/48 | 48/47/112 | 48/48 | +0.00% |
| `source_utf8_unicode_2048` | 43/46/43 | 42/42/48 | 43/42 | -2.33% |
| `source_utf8_unicode_4096` | 89/88/89 | 91/88/91 | 89/91 | +2.25% |
| `source_utf8_unicode_8192` | 180/172/260 | 310/179/175 | 180/179 | -0.56% |
| `source_notes_ascii_128` | 5/5/9 | 10/5/7 | 5/7 | +40.00% |
| `source_notes_ascii_256` | 11/12/28 | 27/11/12 | 12/12 | +0.00% |
| `source_notes_ascii_512` | 24/25/25 | 49/25/25 | 25/25 | +0.00% |
| `source_queue_ascii_256` | 102/106/102 | 106/104/110 | 102/106 | +3.92% |
| `source_queue_ascii_512` | 205/204/216 | 226/213/214 | 205/214 | +4.39% |
| `source_queue_ascii_1024` | 424/465/945 | 516/417/408 | 465/417 | -10.32% |

Sum of row medians: 1213 → 1177 ms (-2.97%).

## source-navigation

Navigation rows retain the existing long-line and multi-line source workloads and checks.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `source_navigation_long_512` | 40/39/45 | 42/83/42 | 40/42 | +5.00% |
| `source_navigation_long_1024` | 82/72/77 | 72/141/76 | 77/76 | -1.30% |
| `source_navigation_long_2048` | 155/141/187 | 142/145/145 | 155/145 | -6.45% |
| `source_navigation_lines_512` | 11/9/19 | 10/9/10 | 11/10 | -9.09% |
| `source_navigation_lines_1024` | 15/15/32 | 15/16/15 | 15/15 | +0.00% |
| `source_navigation_lines_2048` | 25/27/61 | 28/31/28 | 27/28 | +3.70% |

Sum of row medians: 325 → 316 ms (-2.77%).

## Isolated regression controls

Rows whose full-suite medians rose by at least 5% with a control of at least 20 ms
were repeated in eight isolated pairs, alternating before/after and after/before
order. Each used the exact suite driver, arguments, internal warmup, timer and
existing result checks. No builds or other tests ran alongside these pairs.
Raw output and result checks are retained with the local control samples.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `failure_dump_scalar` | 309/584/428/315/321/314/317/313 | 306/313/364/300/310/308/299/305 | 316/307 | -2.85% |
| `mandelbrot_paint` | 1306/1557/1225/1076/1073/2060/1269/1086 | 1298/1167/1086/1130/1255/1280/1145/1152 | 1247/1159.5 | -7.02% |
| `task_spawn_wait` | 816/816/698/654/774/658/786/639 | 518/667/906/588/799/641/794/814 | 736/730.5 | -0.75% |
| `source_navigation_long_512` | 39/41/39/39/40/39/41/44 | 41/42/38/39/43/41/38/39 | 39.5/40 | +1.27% |

No unchanged VM row crossed the selected regression gate. The full-suite
increases did not repeat in the isolated controls: failure capture, Mandelbrot
painting and task spawning had lower medians; long-line navigation remained
within 5%. The opposing samples demonstrate cloud scatter, rather than an
attributable optimization or a repeatable slowdown.

## Reproduction and retained evidence

Run `cargo bench-fpas save <label>` from each selected tree, without a group
filter, three times. Reusing each tree's built harness with the same `save`
arguments avoids toggling Cargo dependency features; it still builds/selects the
release CLI. Do not run builds/tests in parallel with measured rows. Isolated
controls execute the corresponding CLI/native driver with the suite's exact
arguments and alternate version order.

The six snapshots are named `ap47-6-{before,after}-full-<1..3>.json` in the working
tree under `.temp-data/bench/`. The before snapshots were saved before implementation;
isolated controls use the unchanged source archive at the commit above.
They retain each workload's stdout. The working tree also retains
`.temp-data/ap47.6-bench-summary.json` and
`.temp-data/ap47.6-isolated-controls.json`, including isolated stdout and checks.
The committed tables preserve all before/after timer samples without machine
identifiers. The suite itself has not gained an `errors` group.

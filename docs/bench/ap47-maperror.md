# AP47.4 constructor and callback regression measurements

The unchanged AP47.3 source archive at `c3dee7ac7` is the control. The comparison is the AP47.4 implementation. Both use Rust 1.99.1, the repository release profile (fat LTO and one codegen unit), the same unchanged suite, and identical workload arguments. Three paired runs per group ran serially without unrelated builds or tests during timing. The repository benchmark harness obtained each release CLI through Cargo; its prebuilt binary was reused to avoid rebuilding a different dependency-feature graph for every invocation.

These VM and concurrency rows guard existing execution and callback paths. They do not measure the new MapError operation against an explicit case. Dedicated error workloads remain scheduled after the original Std migration comparison.

All samples are milliseconds. Totals are sums of row medians, not whole-suite wall times. The VM gate covers repeatable increases over 5% when the control median is at least 20 ms; shorter rows retain their raw samples.

## vm

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 1183/1255/1473 | 1295/1175/1640 | 1255/1295 | +3.19% |
| `typed_real` | 241/242/354 | 242/241/275 | 242/242 | +0.00% |
| `typed_boolean` | 353/360/792 | 410/345/390 | 360/390 | +8.33% |
| `typed_string` | 230/274/493 | 254/230/250 | 274/250 | -8.76% |
| `global_access` | 169/172/409 | 199/176/175 | 172/176 | +2.33% |
| `record_field_access` | 584/568/684 | 871/642/575 | 584/642 | +9.93% |
| `closure_call` | 261/234/269 | 369/237/232 | 261/237 | -9.20% |
| `mutable_capture` | 293/313/344 | 580/295/308 | 313/308 | -1.60% |
| `branch_dispatch` | 599/607/778 | 702/583/589 | 607/589 | -2.97% |
| `dynamic_numeric` | 873/525/531 | 957/529/538 | 531/538 | +1.32% |
| `array_push` | 160/99/429 | 245/85/90 | 160/90 | -43.75% |
| `array_length` | 34/34/57 | 45/35/34 | 34/35 | +2.94% |
| `string_concat` | 1728/1491/2145 | 1629/1631/1467 | 1728/1629 | -5.73% |
| `string_length` | 36/36/48 | 36/39/35 | 36/36 | +0.00% |
| `intrinsic_dispatch` | 1111/1003/992 | 1014/1214/984 | 1003/1014 | +1.10% |
| `function_call` | 434/425/426 | 419/550/434 | 426/434 | +1.88% |
| `array_callbacks` | 1389/1822/1684 | 1353/1489/1390 | 1684/1390 | -17.46% |
| `record_update` | 151/170/174 | 150/146/150 | 170/150 | -11.76% |
| `unicode_char_at` | 424/625/470 | 420/421/429 | 470/421 | -10.43% |
| `wrapper_payload` | 1061/1697/1143 | 1047/1039/1135 | 1143/1047 | -8.40% |
| `string_search` | 129/125/133 | 114/113/116 | 129/114 | -11.63% |
| `substring_ascii` | 10/9/11 | 9/9/9 | 10/9 | -10.00% |
| `substring_unicode` | 19/18/19 | 19/19/57 | 19/19 | +0.00% |

Sum of row medians: 11611 → 11055 ms (-4.79%).

## concurrency

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `task_spawn_wait` | 777/772/999 | 200/470/813 | 777/470 | -39.51% |
| `task_array_callbacks` | 2097/1508/1799 | 2035/1534/1633 | 1799/1633 | -9.23% |

Sum of row medians: 2576 → 2103 ms (-18.36%).

## Reproduction and retained samples

Use `cargo bench-fpas save <label> --group vm` and the same command with `--group concurrency`. When reusing the built harness, run its `save` command with the same arguments from the selected tree; the harness still builds and selects the release CLI. Prepare builds and complete language tests before timing.

Snapshots are named `ap47-4-before-<group>-<1..3>.json` in the control and `ap47-4-after-<group>-<1..3>.json` in the working tree, under `.temp-data/bench/`. Each retains the checked workload stdout.

## Isolated regression controls

The initial full-group medians exceeded the VM gate for `typed_boolean` and
`record_field_access`, with substantial sample scatter. Eight additional pairs
ran each unchanged workload independently, alternating before/after and
after/before order. They used the suite's exact CLI command, source, arguments,
internal warmup and timer. All runs exited successfully and retained equal
checksums between versions. No builds or other tests ran during these controls.

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `typed_boolean` | 349/345/376/350/344/353/353/352 | 349/344/348/341/342/341/345/341 | 351/343 | -2.28% |
| `record_field_access` | 549/548/548/588/601/552/553/550 | 545/561/641/586/553/565/639/561 | 551/563 | +2.18% |

The increases above 5% did not repeat in the isolated controls; no reproducible
VM regression crosses the selected gate. The full-group totals remain in this
report unchanged. Their scatter does not support attributing a performance
improvement to AP47.4. Raw control stdout, checksums and paired order are retained
in `.temp-data/ap47.4-isolated-controls.json`.

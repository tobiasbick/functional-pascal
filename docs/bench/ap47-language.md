# AP47.3 language performance

The control is the unchanged AP47.2 checkout at `7f0fd9f`, rebuilt from its
source archive. The comparison is the AP47.3 working tree. Both use the same
release harness, suite and arguments; power settings were left unchanged. Three runs per group
and tree ran serially, with no unrelated builds or tests during timing.

These unchanged workloads guard existing VM paths, compiler/editor work,
project builds and artifact admission. They do not measure the new error
syntax itself. The dedicated error workloads remain scheduled after Std migration.

All times are milliseconds. Group totals are sums of row medians, not
whole-suite wall times. The VM gate applies to repeatable increases over 5%
for rows whose control median is at least 20 ms. Shorter rows retain raw samples.

## vm

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 716/693/698 | 715/714/715 | 698/715 | +2.44% |
| `typed_real` | 154/155/167 | 158/153/154 | 155/154 | -0.65% |
| `typed_boolean` | 241/240/241 | 239/241/241 | 241/241 | +0.00% |
| `typed_string` | 131/130/123 | 129/130/121 | 130/129 | -0.77% |
| `global_access` | 102/99/100 | 98/98/98 | 100/98 | -2.00% |
| `record_field_access` | 332/330/330 | 328/327/330 | 330/328 | -0.61% |
| `closure_call` | 144/142/144 | 145/142/142 | 144/142 | -1.39% |
| `mutable_capture` | 184/179/179 | 181/192/181 | 179/181 | +1.12% |
| `branch_dispatch` | 345/343/342 | 341/352/346 | 343/346 | +0.87% |
| `dynamic_numeric` | 282/280/281 | 285/283/286 | 281/285 | +1.42% |
| `array_push` | 53/53/53 | 50/54/51 | 53/51 | -3.77% |
| `array_length` | 23/23/23 | 23/23/23 | 23/23 | +0.00% |
| `string_concat` | 893/886/898 | 917/1165/929 | 893/929 | +4.03% |
| `string_length` | 22/22/22 | 22/22/21 | 22/22 | +0.00% |
| `intrinsic_dispatch` | 655/653/654 | 653/658/667 | 654/658 | +0.61% |
| `function_call` | 262/262/261 | 260/265/258 | 262/260 | -0.76% |
| `array_callbacks` | 828/835/832 | 835/835/829 | 832/835 | +0.36% |
| `record_update` | 102/86/86 | 87/87/86 | 86/87 | +1.16% |
| `unicode_char_at` | 267/267/266 | 264/264/262 | 267/264 | -1.12% |
| `wrapper_payload` | 722/724/716 | 724/716/719 | 722/719 | -0.42% |
| `string_search` | 68/68/69 | 68/68/68 | 68/68 | +0.00% |
| `substring_ascii` | 6/6/6 | 6/6/6 | 6/6 | +0.00% |
| `substring_unicode` | 11/11/10 | 10/10/10 | 11/10 | -9.09% |

Sum of row medians: 6500 → 6551 ms (+0.78%).

## tooling

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `analysis_queries` | 231/232/230 | 223/222/223 | 231/223 | -3.46% |
| `compiler_lowering` | 1897/1888/1915 | 1856/1854/1922 | 1897/1856 | -2.16% |
| `project_queries` | 928/926/1003 | 926/888/889 | 928/889 | -4.20% |
| `project_edits` | 2990/2822/2888 | 2774/2759/2774 | 2888/2774 | -3.95% |
| `project_overlapping_queries` | 689/600/601 | 569/571/574 | 601/571 | -4.99% |

Sum of row medians: 6545 → 6313 ms (-3.54%).

## startup

| Workload | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `project_build_cold` | 1903/1900/1901 | 1871/1876/1904 | 1901/1876 | -1.32% |
| `project_build_warm` | 1261/1274/1259 | 1240/1255/1257 | 1261/1255 | -0.48% |
| `unit_artifact_shared_types` | 0/0/0 | 0/0/0 | 0/0 | n/a |
| `program_artifact_shared_types` | 0/0/0 | 0/0/0 | 0/0 | n/a |

Sum of row medians: 3162 → 3131 ms (-0.98%).

## Additional controls

The configured codec rows finish below the millisecond timer resolution.
Their zero samples do not quantify decoder performance. Matched native
controls therefore use 50,000 decodes at the same depth 16, with fixture
construction and warmup still outside timing. Suite IDs and arguments stay unchanged.

The two small VM increases were checked with alternating fixed old/current
binaries and identical source/arguments. The smaller shifts remain measurable;
these measurements do not isolate their cause. Primitive instruction handlers
gain no new work in this slice. All three-run VM medians and these controls
stay below the selected 5% gate.

| Control | Before samples | After samples | Median before/after | Change |
| --- | --- | --- | --- | --- |
| `integer_loop` | 685/690/698 | 707/712/704 | 690/707 | +2.46% |
| `string_concat` | 886/884/882 | 927/919/926 | 884/926 | +4.75% |
| `unit-artifact` | 90/87/88 | 91/90/90 | 88/90 | +2.27% |
| `program-artifact` | 69/67/67 | 66/66/66 | 67/66 | -1.49% |

The string-concatenation suite keeps its 1,165 ms outlier in the raw samples.
The separate harness comparisons for all three groups complete successfully.
The VM table in [history.md](history.md) is an additional current `record` run,
not a replacement for the three-run comparison above.

## Local evidence

Snapshots use `ap47-3-before-<group>-<1..3>.json` in the archived control
and `ap47-3-after-<group>-<1..3>.json` in the working tree, under
`.temp-data/bench/`. Each snapshot retains the workload stdout.

Alternating VM and codec samples are retained in `.temp-data/ap47.3-controls.json`
and `.temp-data/ap47.3-codec-controls.json`.

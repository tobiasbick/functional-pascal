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

## 2026-08-08 — after portable register VM

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 6899 | throughput: 7247427 iters/s |
| global_access | 641 | throughput: 7800312 global updates/s |
| record_field_access | 2152 | throughput: 5576208 field accesses/s |
| closure_call | 616 | throughput: 4870129 closure calls/s |
| branch_dispatch | 3445 | throughput: 5805515 branches/s |
| dynamic_numeric | 1057 | throughput: 4730368 dynamic numeric ops/s |
| array_push | 253 | throughput: 7905138 pushes/s |
| array_length | 108 | throughput: 4629629 lengths/s |
| string_concat | 2949 | throughput: 1695489 concats/s |
| string_length | 82 | throughput: 6097560 lengths/s |
| function_call | 862 | throughput: 6960556 calls/s |
| array_callbacks | 1401 | throughput: 6852248 callbacks/s |
| record_update | 593 | throughput: 1686340 updates/s |
| unicode_char_at | 1770 | throughput: 1694915 chars/s |
| task_spawn_wait | 945 | throughput: 105820 tasks/s |
| tui_headless | 17079 | throughput: 29 frames/s |

## 2026-08-03 — after TUI surface bulk fills

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 5292 | throughput: 9448223 iters/s |
| array_push | 144 | throughput: 13888888 pushes/s |
| array_length | 61 | throughput: 8196721 lengths/s |
| string_concat | 2156 | throughput: 2319109 concats/s |
| string_length | 52 | throughput: 9615384 lengths/s |
| function_call | 798 | throughput: 7518796 calls/s |
| array_callbacks | 897 | throughput: 10702341 callbacks/s |
| record_update | 515 | throughput: 1941747 updates/s |
| unicode_char_at | 953 | throughput: 3147953 chars/s |
| task_spawn_wait | 622 | throughput: 160771 tasks/s |
| tui_headless | 4274 | throughput: 116 frames/s |

## 2026-08-03 — after completed Rust crate review fixes

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 5291 | throughput: 9450009 iters/s |
| array_push | 144 | throughput: 13888888 pushes/s |
| array_length | 60 | throughput: 8333333 lengths/s |
| string_concat | 2142 | throughput: 2334267 concats/s |
| string_length | 53 | throughput: 9433962 lengths/s |
| function_call | 789 | throughput: 7604562 calls/s |
| array_callbacks | 875 | throughput: 10971428 callbacks/s |
| record_update | 521 | throughput: 1919385 updates/s |
| unicode_char_at | 946 | throughput: 3171247 chars/s |
| task_spawn_wait | 595 | throughput: 168067 tasks/s |
| tui_headless | 16660 | throughput: 30 frames/s |

## 2026-07-26 — after in-place binary integer stack reduction

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 4257 | throughput: 11745360 iters/s |
| array_push | 126 | throughput: 15873015 pushes/s |
| array_length | 54 | throughput: 9259259 lengths/s |
| string_concat | 2093 | throughput: 2388915 concats/s |
| string_length | 47 | throughput: 10638297 lengths/s |
| function_call | 733 | throughput: 8185538 calls/s |
| array_callbacks | 683 | throughput: 14055636 callbacks/s |
| record_update | 493 | throughput: 2028397 updates/s |
| unicode_char_at | 934 | throughput: 3211991 chars/s |
| task_spawn_wait | 582 | throughput: 171821 tasks/s |
| tui_headless | 3371 | throughput: 148 frames/s |

## 2026-07-25 — after compact shared Value storage

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 9830 | throughput: 5086469 iters/s |
| array_push | 189 | throughput: 10582010 pushes/s |
| array_length | 77 | throughput: 6493506 lengths/s |
| string_concat | 3399 | throughput: 1471020 concats/s |
| string_length | 79 | throughput: 6329113 lengths/s |
| function_call | 1128 | throughput: 5319148 calls/s |
| array_callbacks | 1032 | throughput: 9302325 callbacks/s |
| record_update | 698 | throughput: 1432664 updates/s |
| unicode_char_at | 1481 | throughput: 2025658 chars/s |
| task_spawn_wait | 983 | throughput: 101729 tasks/s |
| tui_headless | 7826 | throughput: 63 frames/s |

## 2026-07-25 — after borrowing direct call names

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 6131 | throughput: 8155276 iters/s |
| array_push | 181 | throughput: 11049723 pushes/s |
| array_length | 62 | throughput: 8064516 lengths/s |
| string_concat | 2298 | throughput: 2175805 concats/s |
| string_length | 56 | throughput: 8928571 lengths/s |
| function_call | 1008 | throughput: 5952380 calls/s |
| array_callbacks | 915 | throughput: 10491803 callbacks/s |
| record_update | 1239 | throughput: 807102 updates/s |
| unicode_char_at | 1001 | throughput: 2997002 chars/s |
| task_spawn_wait | 645 | throughput: 155038 tasks/s |
| tui_headless | 8939 | throughput: 55 frames/s |

## 2026-07-25 — baseline for function calls, callbacks, records, Unicode, and tasks

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 7116 | throughput: 7026419 iters/s |
| array_push | 197 | throughput: 10152284 pushes/s |
| array_length | 64 | throughput: 7812500 lengths/s |
| string_concat | 2284 | throughput: 2189141 concats/s |
| string_length | 57 | throughput: 8771929 lengths/s |
| function_call | 1098 | throughput: 5464480 calls/s |
| array_callbacks | 927 | throughput: 10355987 callbacks/s |
| record_update | 1232 | throughput: 811688 updates/s |
| unicode_char_at | 987 | throughput: 3039513 chars/s |
| task_spawn_wait | 574 | throughput: 174216 tasks/s |
| tui_headless | 10228 | throughput: 48 frames/s |

## 2026-07-25 — after fused counting for (IncLocal/JumpIfLocal*) + for-in IncLocal/SetLocalPop

- Group: `all`
- Suite: [`suite.toml`](suite.toml)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 6016 | throughput: 8311170 iters/s |
| array_push | 181 | throughput: 11049723 pushes/s |
| array_length | 64 | throughput: 7812500 lengths/s |
| string_concat | 2336 | throughput: 2140410 concats/s |
| string_length | 56 | throughput: 8928571 lengths/s |
| tui_headless | 9368 | throughput: 53 frames/s |

## 2026-07-25 — after SharedStr char_len cache

- Group: `all`
- Suite: [`suite.toml`](suite.toml)
- Notes: includes prior wins (VM flat dispatch, array Length COW, `SharedStr` Arc sharing, ASCII/`char_len` Length)

| bench | elapsed_ms | throughput |
|-------|------------|------------|
| integer_loop | 9410 | throughput: 5313496 iters/s |
| array_push | 323 | throughput: 6191950 pushes/s |
| array_length | 97 | throughput: 5154639 lengths/s |
| string_concat | 2501 | throughput: 1999200 concats/s |
| string_length | 92 | throughput: 5434782 lengths/s |
| tui_headless | 9171 | throughput: 54 frames/s |

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

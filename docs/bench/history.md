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

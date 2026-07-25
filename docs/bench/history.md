# Benchmark history

Committed snapshots from `cargo bench-fpas record`. Absolute times are machine-specific; use them
to track relative progress on the same machine and to see which changes moved which benches.

Update after a meaningful performance change:

```sh
cargo bench-fpas record "short note about the change"
cargo bench-fpas record "vm-only note" --group vm
```

Newest entries are prepended below this header.

## 2026-07-25 — after SharedStr char_len cache

- Host: local (seeded from full suite run)
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

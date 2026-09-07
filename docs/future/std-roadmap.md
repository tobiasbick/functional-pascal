# Future: Standard Library Roadmap

> Deferred. Planned for future versions.

Functional Pascal already has a useful `Std.*` surface for console programs, TUI experiments,
strings, math, arrays, dictionaries, options, results, tasks, filesystem access, processes, TCP,
HTTP, and OpenAI-compatible chat. The next standard-library work should keep that unit-based shape
and add focused units instead of merging unrelated APIs into larger buckets.

## Direction

- Keep `Std.*` split by domain.
- Prefer small, documented units with predictable names over broad catch-all modules.
- Keep hosted runtime capabilities explicit when an API touches the process, filesystem, clock, or environment.
- Keep pure helpers separate from effectful APIs.
- Avoid adding memory-management APIs as a standard unit for now.

## Implemented concurrency

Bounded channels, cooperative cancellation, mixed-source selection, task groups, supervised
retries, and timed group close are implemented in [Std.Task](../pascal/std/concurrency/task.md).
Their [concurrency worklist](networked-applications/concurrency.md) records completion evidence.
Hard process-level shutdown escalation remains part of [server lifecycle](networked-applications/server-lifecycle.md).

## Later candidates

These should wait until the runtime and capability model need them:

- [`Std.Crypto`](networked-applications/cryptography.md) for secure randomness, password hashing,
  message authentication, and signatures.
- [`Std.Sqlite`](networked-applications/sqlite.md) for embedded transactional storage.
- [WebSocket client and server support](networked-applications/websocket.md).
- [UDP sockets](networked-applications/udp.md) when a concrete datagram use case exists.
- binary buffers and codecs, once byte-array conventions are stable.

The cross-cutting order and acceptance gates for these capabilities are tracked in the
[networked application platform plan](networked-applications/README.md).

## Implementation Checklist

For every new or moved `Std.*` API:

- add or update the unit page under `docs/pascal/std/`.
- update the standard-library index in `docs/pascal/std/README.md`.
- add sema registration in `fpas-sema` standard-unit wiring.
- add runtime implementation in `fpas-std` where needed.
- add bytecode intrinsic wiring when the API requires VM support.
- add focused examples under `examples/pascal/std/` when useful.
- add tests for successful calls and important edge cases.

## Open Decisions

- How should hosted capabilities be represented for filesystem, environment, time, and process APIs?
- What is the canonical representation for byte data?
- Should filesystem text APIs assume UTF-8 only?
- Should `Std.Fs` remain blocking-but-`go`-friendly, or should a later runtime add true non-blocking filesystem operations?
- How much platform-specific behavior should be exposed versus normalized?

# Future: Standard Library Roadmap

> Deferred. Planned for future versions.

Functional Pascal already has a useful `Std.*` surface for console programs, TUI experiments, graphics, strings, math, arrays, dictionaries, options, results, and tasks. The next standard-library work should keep that unit-based shape and add focused units instead of merging unrelated APIs into larger buckets.

## Direction

- Keep `Std.*` split by domain.
- Prefer small, documented units with predictable names over broad catch-all modules.
- Keep hosted runtime capabilities explicit when an API touches the process, filesystem, clock, or environment.
- Keep pure helpers separate from effectful APIs.
- Avoid adding memory-management APIs as a standard unit for now.

## Mid-Term Units

### `Std.Parse`

Add parsing helpers if `Std.Conv` grows beyond simple conversions.

Possible scope:

- integer and real parsing with explicit error behavior.
- boolean parsing.
- token-oriented helpers for small CLI tools.

Implementation notes:

- Keep `Std.Conv` for straightforward type-to-type conversions.
- Use `Std.Parse` when callers need structured parse errors or nontrivial input rules.

## Later Candidates

These should wait until the runtime and capability model need them:

- `Std.Proc` for spawning processes and inspecting exit status.
- `Std.Net` for sockets or low-level networking.
- `Std.Http` for request/response workflows.
- `Std.Crypto` for hashing and cryptographic primitives.
- binary buffers and codecs, once byte-array conventions are stable.

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
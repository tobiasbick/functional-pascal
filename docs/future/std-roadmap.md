# Future: Standard Library Roadmap

> Deferred. Planned for future versions.

Functional Pascal already has a useful `Std.*` surface for console programs, TUI experiments, graphics, strings, math, arrays, dictionaries, options, results, and tasks. The next standard-library work should keep that unit-based shape and add focused units instead of merging unrelated APIs into larger buckets.

## Direction

- Keep `Std.*` split by domain.
- Prefer small, documented units with predictable names over broad catch-all modules.
- Keep hosted runtime capabilities explicit when an API touches the process, filesystem, clock, or environment.
- Keep pure helpers separate from effectful APIs.
- Avoid adding memory-management APIs as a standard unit for now.

## Near-Term Units

### `Std.Env`

Add process environment access.

`Std.Env` is also a process-wide hosted capability, not a console-only API. Console, TUI, and Graph programs should all be able to import it and read the environment visible to the host process.

Initial scope:

- get variable by name.
- check whether a variable exists.
- list variables later, if dictionary or array ergonomics are suitable.

Implementation notes:

- Treat environment access as hosted and effectful.
- Keep the API process-wide and UI-independent: `Std.Console`, `Std.Tui`, and `Std.Graph` programs should all be able to use it when imported.
- Return `Option of String` for missing values when possible.
- Keep mutation APIs deferred until the process model needs them.

### `Std.Path`

Add path manipulation without filesystem access.

Initial scope:

- join path segments.
- basename and dirname.
- extension extraction.
- normalize separators where the target platform permits it.

Implementation notes:

- Keep `Std.Path` pure: it should not check whether paths exist.
- Document platform-sensitive behavior.
- Avoid guessing current working directory inside path helpers.

### `Std.Fs`

Add basic filesystem operations.

Filesystem operations should be regular blocking standard-library calls at first, but they must be safe and documented for use with `go`. This gives programs task-based asynchronous file workflows without introducing a separate `async` / `await` language model.

Initial scope:

- read text file.
- write text file.
- exists checks.
- file and directory distinction.
- create directory.

Implementation notes:

- Keep this separate from `Std.Path`.
- Use `Result` for fallible operations once error typing is ready enough.
- Define encoding behavior for text reads and writes.
- Document that `Std.Fs` calls may block the worker thread that runs them.
- Ensure filesystem runtime code is thread-safe when called from multiple `go` tasks.
- Add examples that spawn reads or writes with `go` and collect results with `Std.Task.Wait`.
- Add binary APIs later only after byte-array conventions are stable.

Example shape:

```pascal
uses Std.Fs, Std.Task;

begin
	var ReadJob: task := go ReadText('input.txt');
	var Text: string := Wait(ReadJob)
end.
```

Later, if the language grows a dedicated async model, `Std.Fs` can gain non-blocking variants or a separate async unit. That should be a runtime design decision, not a requirement for the first filesystem API.

### `Std.Time`

Add time and duration helpers.

Initial scope:

- current timestamp.
- monotonic elapsed time.
- sleep by milliseconds.
- duration helpers if records or numeric conventions make them pleasant to use.

Implementation notes:

- Move or mirror console delay behavior only after deciding the compatibility story.
- Keep wall-clock time and monotonic time distinct.
- Document precision and platform limits.

## Mid-Term Units

### `Std.Json`

Useful for tools, configuration, and simple data interchange.

Initial scope:

- parse JSON text into a documented FPAS value representation.
- stringify supported values.
- typed lookup helpers if dynamic values are introduced.

Implementation notes:

- Do not invent a large dynamic object model accidentally.
- Decide how JSON null maps to existing `Option` or future dynamic values.
- Start with clear errors and small examples.

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
# Examples

Use these examples to calibrate behavior when this skill triggers.

## Example 1: Add the dependency

User request:

```text
continue with turbo-vision-4-rust, start with the dependency
```

Expected agent behavior:

- Read the required future-plan files.
- Check the current phase in `04-implementation-phases.md`.
- Verify upstream `turbo-vision` version from crates.io or GitHub.
- Add the dependency in the narrowest crate that needs it, or workspace dependencies if several crates need it.
- Run `cargo build` and `cargo tree -i crossterm`.
- Update `04-implementation-phases.md` with completed checks and any version notes.
- Do not modify old `Application.Host*` APIs yet.

## Example 2: Design the new API

User request:

```text
design the new Std.Tui API over Turbo Vision
```

Expected agent behavior:

- Do not mirror Rust traits or builders into FPAS.
- Work from `03-target-api.md`.
- Propose handles for live objects and records for values.
- Keep planned behavior in `docs/future/turbo-vision-4-rust/` until implementation exists.
- Mention open decisions explicitly.

## Example 3: Implement command callback spike

User request:

```text
implement the minimal callback spike
```

Expected agent behavior:

- Read `fpas-change-checklist`.
- Verify upstream constructors and event/command routing from source.
- Implement only enough API for application, one window/dialog, one button, command callback, run or pump, and quit.
- Add Rust VM coverage for callback and cleanup.
- Add an FPAS test only if headless/test execution is available.
- Stop before deleting old TUI internals unless the spike passes.

## Example 4: Remove old Host APIs

User request:

```text
delete the old TUI host API
```

Expected agent behavior:

- Confirm Phase 2 callback and headless gate is complete.
- Remove sema registrations, compiler lowering, bytecode intrinsics, VM implementations, docs, examples, and tests tied only to old `Application.Host*`.
- Do not keep compatibility aliases.
- Update current `docs/pascal/std/tui/` to describe only implemented replacement APIs.
- Run broad verification.

## Example 5: User asks about upstream API details

User request:

```text
what Button constructor does turbo-vision currently expose?
```

Expected agent behavior:

- Browse or fetch upstream source instead of relying on memory.
- Cite the upstream file or summarize the exact checked signature.
- If network access is unavailable, say the answer is unverified and may be stale.

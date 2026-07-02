# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Planned Work

| Area | Plan | Scope |
|------|------|-------|
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| Libraries | [Library export model](libraries.md) | Finer per-symbol exports and re-export rules beyond current unit exports |
| Task runtime | [Task memory benchmark](task-memory-benchmark.md) | Reproduce async memory benchmark and validate future task-runtime behavior |
| TUI | [`Std.Tui` Turbo Vision rewrite](turbo-vision-4-rust/README.md) | **Migration complete** on `turbo-vision-4-rust`; remaining polish in [07-post-migration-improvements.md](turbo-vision-4-rust/07-post-migration-improvements.md) |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Remove completed planning notes once the implemented docs and tests make the plan obsolete.

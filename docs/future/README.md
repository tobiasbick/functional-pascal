# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Open planning items

| Area | Plan | Scope |
|------|------|-------|
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Networking | [Networking follow-up](networking.md) | HTTP/HTTPS server, WebDAV, FTP/FTPS |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| Runtime | [Deferred Cranelift backend](cranelift-backend.md) | Parked second-backend idea with explicit re-entry gates |

## Architecture records and development intake

| Area | Document | Scope |
|------|----------|-------|
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Intake for newly discovered compiler panics and language limitations |
| Workspace crates | [Crate review follow-ups (2026-08)](crate-review-2026-08/README.md) | Defect list and one-task-at-a-time implementation plan |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once implemented docs and tests cover their scope.

# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Open planning items

| Area | Plan | Scope |
|------|------|-------|
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| IDE | [IDE next steps](ide-next-steps.md) | Decide project/workspace startup before multiple documents |
| Language | [Opaque records](opaque-records.md) | Representation hiding and unforgeable transient capabilities |

## Architecture records and development intake

| Area | Document | Scope |
|------|----------|-------|
| Libraries | [Compiled-unit architecture](libraries.md) | Implemented `.fpascu` architecture and separately scoped extensions |
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Intake for newly discovered compiler panics and language limitations |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once the implemented docs and tests make the plan obsolete.

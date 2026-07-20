# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Planned Work

| Area | Plan | Scope |
|------|------|-------|
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| Libraries | [Library export model](libraries.md) | Finer per-symbol exports and re-export rules beyond current unit exports |
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Compiler panics and source-level language constraints found during implementation |
| Std.Tui | [Turbo Vision bridge](tui-bridged-readback.md) | Functionally complete except for three upstream read-back adapters — good [contributor entry point](../../AI_CONTRIBUTING.md#good-entry-points) |
| Std.Tui2 | [FPAS-native terminal UI](tui2/README.md) | New source-library architecture, API contracts, layout, events, lifecycle, and implementation phases |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once the implemented docs and tests make the plan obsolete.

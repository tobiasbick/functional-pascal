# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Planned Work

| Area | Plan | Scope |
|------|------|-------|
| Language | — | Bound methods, record properties, and event properties are in `docs/pascal/`; see Tui2 step 4 for applying events to controls |
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| Libraries | [Library export model](libraries.md) | Finer per-symbol exports and re-export rules beyond current unit exports |
| Std.Tui | [Turbo Vision bridge](tui-bridged-readback.md) | Functionally complete except for three upstream read-back adapters — good [contributor entry point](../../AI_CONTRIBUTING.md#good-entry-points) |
| Std.Tui2 | [FPAS-native terminal UI](tui2/README.md) | New source-library architecture, API contracts, layout, events, lifecycle, and implementation phases |
| IDE | [Project tree](ide-project-tree.md) | `Std.Toml` and `Std.Fs.Glob` complete; IDE tree window next |

## Recommended Tui2 Language Sequence

The four plans form one dependency chain. Use this order:

| Step | Plan or milestone | Why it comes here |
| --- | --- | --- |
| 1 | ~~Bound-method milestone~~ **done** | Spec: [record-methods.md](../pascal/language/types/record-methods.md#bound-methods-as-values). |
| 2 | ~~Record properties~~ **done** | Spec: [record-properties.md](../pascal/language/types/record-properties.md). |
| 3 | ~~Event-property milestone~~ **done** | Spec: [record-events.md](../pascal/language/types/record-events.md). |
| 4 | [Tui2 events and actions](tui2/events-and-actions.md) | Applies the completed language features to controls, lifecycle callbacks, actions, and posting. |

Capturing closures, bound record methods, record properties, and event properties are
implemented; see
[`closures.md`](../pascal/language/functions/closures.md),
[`record-methods.md`](../pascal/language/types/record-methods.md#bound-methods-as-values),
[`record-properties.md`](../pascal/language/types/record-properties.md), and
[`record-events.md`](../pascal/language/types/record-events.md).

Each remaining step must finish its own docs, tests, formatter support, linker/source-map handling,
and full verification before the next dependent step starts.

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Remove completed planning notes once the implemented docs and tests make the plan obsolete.

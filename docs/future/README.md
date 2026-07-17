# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Planned Work

| Area | Plan | Scope |
|------|------|-------|
| Language | [Capturing closures](capturing-closures.md) | Escaping anonymous and nested callables with managed lexical environments |
| Language | [Record properties](record-properties.md) | Pascal-style computed record state backed by instance accessors |
| Language | [Events and bound methods](events-and-bound-methods.md) | Bound method values and deterministic single-handler event properties |
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Dictionaries | [Dictionary decision](09-remove-dict.md) | Decide whether `Std.Dict` stays, changes, or is removed |
| Libraries | [Library export model](libraries.md) | Finer per-symbol exports and re-export rules beyond current unit exports |
| Std.Tui | [Turbo Vision bridge](tui-bridged-readback.md) | Functionally complete except for three upstream read-back adapters — good [contributor entry point](../../AI_CONTRIBUTING.md#good-entry-points) |
| Std.Tui2 | [FPAS-native terminal UI](tui2/README.md) | New source-library architecture, API contracts, layout, events, lifecycle, and implementation phases |
| IDE | [Project tree](ide-project-tree.md) | `Std.Toml` and `Std.Fs.Glob` complete; IDE tree window next |

## Recommended Tui2 Language Sequence

The four plans form one dependency chain, but the combined events plan has two independent
milestones. Use this order:

| Step | Plan or milestone | Why it comes here |
| --- | --- | --- |
| 0 | Finish the in-progress expression postfix-chaining implementation | Properties and Tui2 APIs should build on the final member-access behavior rather than modify it concurrently. |
| 1 | [Capturing closures](capturing-closures.md) | Establishes managed callable environments, mutable captures, lifetime, and task-transfer rules. |
| 2 | [Bound-method milestone](events-and-bound-methods.md#milestone-1-bound-record-methods) | Reuses closure environments and completes the three handler forms: named routine, closure, and bound method. |
| 3 | [Record properties](record-properties.md) | Establishes computed getter/setter syntax and registry-backed state access without special Tui2 compiler behavior. |
| 4 | [Event-property milestone](events-and-bound-methods.md#milestone-2-event-properties) | Specializes properties for assignment, `nil`, `Assigned`, and owner-only invocation. |
| 5 | [Tui2 events and actions](tui2/events-and-actions.md) | Applies the completed language features to controls, lifecycle callbacks, actions, and posting. |

Each step must finish its own docs, tests, formatter support, linker/source-map handling, and full
verification before the next dependent step starts. Bound methods and record properties are
technically independent after closures and may be implemented in parallel, but event properties
must wait for both.

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Remove completed planning notes once the implemented docs and tests make the plan obsolete.

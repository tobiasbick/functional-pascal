# AP53: Standard-library namespaces

Status: Planned. Effort: medium, with a broad consumer migration.
Completion is tracked in the [central plan](../README.md#package-status).
The selected integration branch is `codex/std-namespaces`.
Only planning and documentation are authorized. Implementation, source
migration, builds and tests have not started.

## Goal

Give existing and future standard-library capabilities predictable homes
before introducing the byte buffer. Use one binding
[namespace specification](../../../specs/std-namespaces.md) and retain the
[existing roadmap](../../std-roadmap.md) for capability priorities.

## Current behavior and selected contract

- Current exports mix `Std.Ai.OpenAi` with `Std.Acp` and `Std.Mcp`, and
  `Std.Net.Uri` with `Std.Http` and the domain-misplaced `Std.Net.Utf8`.
  Source manifests already export nested names; intrinsic ACP/MCP names are
  nested too. Namespace syntax is not the missing feature.
- The specification's existing-unit table owns every canonical target.
  `Std.Conv` is split between text formatting and numeric conversion.
  Other moves preserve public operations and behavior.
- Preserve explicit import aliases, case-insensitive resolution, visibility,
  evaluation, resource ownership, limits, errors and cancellation semantics.
  Canonical type and operation names change with their owning units.
- Remove old import paths when the migration is delivered. Rebuild dependent
  artifacts and migrate consumers instead of shipping compatibility units.
- Exclude new buffer APIs, streams, UDP, protocols, shared UI, graphics, audio,
  ownership rules, syntax and native backend changes. `Std.Ui.Terminal` is the
  existing TUI under its new name, not the shared UI rewrite.

## Decisions

| ID | Question | User decision | Affected work packages |
| --- | --- | --- | --- |
| Q68 | How should public standard units be grouped? | Adopt the proposed domain structure and existing-unit moves. Complete a binding catalogue and rules for future capabilities. Detailed allocations are recorded in the specification. | AP53.1, AP53.2, AP53.3 |
| Q69 | How much existing behavior should change? | Migrate names consistently, split `Std.Conv` by responsibility and update all consumers, including the dungeon. Use one canonical path without compatibility aliases. Preserve other behavior. | AP53.2, AP53.3 |
| Q70 | Which document owns each concern and when does this run? | Keep the existing Std roadmap, add a permanent namespace specification and a migration AP, and place the migration before byte buffers. | AP53.1, AP53.2, AP53.3 |
| Q71 | What work is authorized now? | Create `codex/std-namespaces` and update planning/documentation only. Implementation needs a later explicit instruction. | AP53.1, AP53.2, AP53.3 |

## Open decisions

None for the naming contract. A later implementation instruction is still
required before AP53.2 or AP53.3 starts. Runtime contracts for future modules
remain with their feature plans and do not block this migration.

## Dependencies and work packages

Prerequisites: the decisions above and the existing public-unit inventory.
The selected integration branch may carry completed prerequisites for later
work packages before merge, following the
[branch rules](../development-process.md#dependencies-and-branches).

- [x] [AP53.1: Record the namespace contract](01-decision.md)
- [ ] [AP53.2: Migrate FPAS and its repository consumers](02-implementation.md)
- [ ] [AP53.3: Migrate dungeon consumers](03-dungeon-migration.md)

This is the only work-package checklist for AP53. AP53.2 depends on AP53.1;
AP53.3 depends on the delivered compiler/library from AP53.2. Completion does
not imply commit, push or merge authorization.

## Acceptance

Every public unit in the intrinsic list and source export manifest has the
specified home. FPAS tooling, current documentation and both repositories'
consumers use those names. Old paths are rejected, private units remain
private, and behavior is preserved except for canonical name changes and the
specified `Std.Conv` relocation. Applicable regression checks pass.

Future allocations are documented without creating empty units or claiming
new capabilities. The byte-buffer work begins only after the migration is
complete and receives its own design and implementation scope.

## Documentation and regression owners

- Binding policy: [standard-library namespaces](../../../specs/std-namespaces.md).
- Scheduling: [Std roadmap](../../std-roadmap.md) and
  [FPAS 2](../../fpas2-language-directions.md).
- Current documentation: [standard library](../../../pascal/std/README.md),
  [unit imports](../../../pascal/program-structure/units.md) and
  [agent reference](../../../pascal/agents/README.md).
- Implementation and consumers: the affected-path tables in AP53.2 and AP53.3.
- Regression boundaries: [shared constraints](../shared-constraints.md).

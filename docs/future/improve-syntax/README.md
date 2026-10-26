# AP planning and work packages

The FPAS 1 syntax campaign is closed. This directory keeps active work packages,
their planning workflow and a copyable package template for later work, including
[FPAS 2 language directions](../fpas2-language-directions.md).
Completed plans are removed after delivery; implemented behavior remains in the
current handbook and grammar.

Implemented behavior belongs in the [current handbook](../../pascal/README.md)
and [grammar](../../specs/grammar.ebnf). A proposal here does not authorize a
language change. Syntax, semantics and the user-facing language specification
require explicit user agreement before implementation.

## Files

| File | Purpose |
| --- | --- |
| [development-process.md](development-process.md) | Dependencies, branches, decision gates, status tracking, migration, review and completion checks |
| [shared-constraints.md](shared-constraints.md) | Cross-package boundaries and regression checks |
| [example-apNN-topic/README.md](example-apNN-topic/README.md) | Copyable package overview, decisions, dependencies, work-package checklist and acceptance |
| [example-apNN-topic/01-decision.md](example-apNN-topic/01-decision.md) | A bounded decision work package |
| [example-apNN-topic/02-implementation.md](example-apNN-topic/02-implementation.md) | A bounded implementation work package |

The example is a template, not an active AP or an approved proposal.

## Create a package

1. Select a concrete problem and inspect its current implementation, handbook,
   consumers and tests. For FPAS 2, select a topic from the retained directions
   rather than treating that whole document as approved work.
2. Copy `example-apNN-topic/` to `ap52-<topic>/` for the next package. Continue
   with the next unused number for later packages. AP01 through AP50 are
   closed; AP51 is allocated below. Do not reuse or renumber those IDs.
3. Replace `APNN`, `QNN`, the topic and every `<placeholder>`. Keep the README
   and one numbered file per work package, such as `01-decision.md` and
   `02-implementation.md`. Delete template instructions from the copied files.
4. Record the goal, proposed contract, actual prerequisites, open decisions,
   explicit exclusions and a verifiable acceptance criterion. Split work into
   small parts that each deliver a consistent, reviewable result.
5. Add the package to the table below and its work packages to the copied
   package README. Link dependencies to their owning package or work-package
   file. Numbering groups work; prerequisites determine execution order.
6. Start with `Decision pending` where approval or a required design choice is
   missing. Record the user's decision before implementing affected behavior.
   Follow the [development process](development-process.md) for delivery.

Use `AP52` for the next package and `AP52.1`, `AP52.2`, and so on for its work
packages. Decision IDs continue at `Q65`; Q01 through Q57 belong to closed
packages, and Q58 through Q64 belong to AP51. Record each decision
and its affected work packages in the owning package README. Do not create a
second decision or completion index.

## Package status

AP51 adds independent `Std.Mcp` tool registration and serving through the
official Rust SDK (`rmcp`). `Std.Acp` offers `RegisterTool` through the same
implementation and can attach an existing MCP server. Both interfaces expose
registered FPAS functions to external clients with explicit handler ownership,
bounded execution and defined server/session lifetimes. Examples and acceptance
use neutral application state. Target checks and live Luna acceptance run locally.
Keep the example out of this table.

| Done | Package | Effort | Depends on | Status |
| --- | --- | --- | --- | --- |
| [x] | [AP51: MCP tool registration and ACP integration](ap51-mcp-wrapper/README.md) | Medium; shared tool server and ACP convenience | Delivered ACP support; verified local HTTP target | Complete |

The central table is the only package-completion index. Each package README
owns the only checklist of its work packages. Use `[x]` only for completed
delivery, following [status tracking](development-process.md#status-tracking).

| Status | Meaning |
| --- | --- |
| Decision pending | A proposal or required design choice is unresolved; affected implementation is not approved. |
| Agreed direction | The direction is accepted; named open details may still block implementation. |
| Planned | The delegated design and work packages are recorded; implementation still requires authorization. |
| In progress | Implementation is authorized and delivery is underway. |
| Complete | Every work package is delivered and verified within the agreed scope. |
| Deferred | The user has parked the proposal; no work is scheduled and reopening requires a new decision. |
| Rejected; closed | The proposal will not be implemented under this plan. |
| Transferred | Another linked plan owns the work. |

Keep agreement, specification, implementation and verification distinct.
A documented proposal is not implemented behavior; a successful build alone
does not complete a work package. Completed delivery and Git integration are
tracked separately.

## Previous campaign

The removed AP files, recorded decisions, audits and reviews remain in Git
history. Find their revisions with:

```text
git log -- docs/future/improve-syntax
```

Historical AP references do not schedule new work. Reconsidering a rejected or
deferred idea requires a new explicit decision and a new package.

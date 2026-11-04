# Source analysis acceptance

Status: **Implementation and Linux acceptance complete; Windows acceptance open**.
FPAS syntax and runtime behavior are unchanged.

| Step | Status | Current owner |
| --- | --- | --- |
| SA-01 Shared analysis without emission | Implemented | [Compiler API](../../architecture/compiler.md#compiler-api-and-output) |
| SA-02 Evaluated project/solution contexts | Implemented, Linux tested | [Workspaces](../../architecture/workspaces.md) |
| SA-03 Project-aware check | Implemented, Linux tested | [CLI](../../pascal/program-structure/cli.md#check-a-project-or-solution) |
| SA-04 Detached semantic snapshot/index | Implemented, Linux tested | [Snapshots](../../architecture/compiler.md#semantic-snapshots) |
| SA-05 Definition/type queries | Implemented, Linux tested | [Source queries](../../pascal/program-structure/cli.md#query-checked-sources) |
| SA-06 Scoped references | Implemented, Linux tested | [Source queries](../../pascal/program-structure/cli.md#query-checked-sources) |
| SA-07 Packaging, regression and platform acceptance | Linux complete; Windows open | [Checklist](implementation.md) |

Completed design detail belongs in the linked current documentation. This folder
tracks remaining acceptance only. LSP, unsaved buffers, formatter, source edits,
find/inspect/call hierarchies and query MCP tools are outside this delivery;
[CLI gaps](../cli-gap.md) and [editor gaps](../editor-gap.md) own them.

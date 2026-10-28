# AP53.3: Migrate dungeon consumers

Package: [AP53](README.md). Status: complete.
The consumer branch is `codex/16-llm-integration`; commits and pushes are
authorized without merge.

## Prerequisites

- [AP53.2](02-implementation.md) delivers the compiler and standard library
  together, on the selected integration branch or after integration.
- The user authorizes implementation, including the external consumer work.
- Inspect the `fpas-into-the-dungeon` repository instructions, checkout state,
  compiler selection and project manifests before changing its files.

## Contract and scope

All standard-unit imports and affected qualifications in
`fpas-into-the-dungeon` follow the binding namespace specification. The consumer
uses the matching compiler/library from AP53.2. Game rules, generation, network
messages, ACP/MCP behavior and terminal presentation are preserved.

This is a namespace migration. It does not introduce shared UI elements, a
graphical client, byte buffers or other future standard-library features.
The two repositories retain separate commits and pushes on the branches named
in Q71. Unrelated work and merges are outside this migration.

## Affected paths and owners

| Action | Repository/path scope | Purpose |
| --- | --- | --- |
| Inspect | `fpas-into-the-dungeon/AGENTS.md` and its project manifests | Follow consumer rules and identify all compiler/library entry points |
| Modify | All affected FPAS sources in that repository | Migrate game, client, server console, tools and tests, including split `Std.Conv` callers |
| Modify where affected | Its manifests, configuration, fixtures and generated-source inputs | Use the matching library and canonical identities without changing protocol payloads |
| Modify | Its documentation and agent guidance | Keep examples and development commands consistent with the selected FPAS build |
| Reconcile | This AP and the shared UI plan | Record completion of names only, leaving the separate UI rewrite pending |

The inspected consumer paths are `apps/{client,server}/src`,
`libs/{game,world,protocol,persistence}/src`,
`tests/{client,client-session,fixtures,game,levels,protocol,runtime,server,world}`,
and `tools/{world-preview,persistence-bench,pi-launcher,dap-acceptance}/src`.
Current names also occur in `README.md`, `AGENTS.md`, `docs/agent-reference.md`,
`docs/architecture/overview.md`, `docs/plans/16a-gamemaster-connection.md`, and
`DUNGEON_RPG_IDEAS.md`. Project manifests contain no old standard-unit names.
Keep both repositories' unrelated user changes outside this migration.

## Acceptance and verification

Search all tracked consumer content for old public paths and inspect each
remaining match. Check project resolution and affected sources with the
AP53.2 compiler/library pair. Run the consumer's prescribed formatter and
relevant suites. Include generation
fingerprint coverage, terminal client/server behavior and affected AI/network
integration coverage; do not infer external-service success from compilation.

The unchanged generation fingerprints, terminal client/server behavior,
AI/network integration and persistence contracts pass the consumer suite.
Spec and Standards review is complete. The
[verification report](../../../reports/ap53-namespace-migration.md) records the
commands, final results and an earlier non-reproduced readiness timeout.
Git integration follows Q71; completion does not authorize a merge.

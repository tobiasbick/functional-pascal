# AP53.3: Migrate dungeon consumers

Package: [AP53](README.md). Status: not started; implementation not authorized.

## Prerequisites

- [AP53.2](02-implementation.md) delivers the compiler and standard library
  together, on the selected integration branch or after integration.
- The user authorizes implementation, including the external consumer work.
- Inspect the `fpas-into-the-dungeon` repository instructions, checkout state,
  compiler selection and project manifests before changing its files.

## Contract and scope

Migrate all standard-unit imports and affected qualifications in
`fpas-into-the-dungeon` to the binding namespace specification. Use the matching
compiler/library from AP53.2. Preserve game rules, generation, network messages,
ACP/MCP behavior and terminal presentation.

This is a namespace migration. It does not introduce shared UI elements, a
graphical client, byte buffers or other future standard-library features.
The two repositories retain their own Git work; this package does not authorize
committing, pushing or changing unrelated work.

## Affected paths and owners

| Action | Repository/path scope | Purpose |
| --- | --- | --- |
| Inspect | `fpas-into-the-dungeon/AGENTS.md` and its project manifests | Follow consumer rules and identify all compiler/library entry points |
| Modify | All affected FPAS sources in that repository | Migrate game, client, server console, tools and tests, including split `Std.Conv` callers |
| Modify where affected | Its manifests, configuration, fixtures and generated-source inputs | Use the matching library and canonical identities without changing protocol payloads |
| Modify | Its documentation and agent guidance | Keep examples and development commands consistent with the selected FPAS build |
| Reconcile | This AP and the shared UI plan | Record completion of names only, leaving the separate UI rewrite pending |

Before implementation, replace broad consumer scopes with the actual affected
paths found in that checkout. Keep both repositories' uncommitted user changes
outside this migration.

## Acceptance and verification

Search all tracked consumer content for old public paths and inspect each
remaining match. Check project resolution and affected sources with the
AP53.2 compiler/library pair. Run the consumer's prescribed formatter and
relevant suites when implementation is authorized. Include generation
fingerprint coverage, terminal client/server behavior and affected AI/network
integration coverage; do not infer external-service success from compilation.

Compare observable results with the unchanged game contracts. Report any
environment-dependent verification limits without marking unmet acceptance
criteria complete. Review Spec and Standards, then mark AP53.3 complete in the
package README. Mark AP53 complete in the central index only after every work
package is delivered and verified. Git integration remains separately authorized.

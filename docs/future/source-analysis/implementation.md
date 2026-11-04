# Source analysis remaining acceptance

## SA-07

- [x] Package Workspaces, official SolutionPersistence and shared process ownership
  with the CLI; export evaluated inputs through the matching SDK task. Include
  each compiler DLL once and exclude private MSBuild runtime assemblies.
- [x] Verify fresh isolated Linux SDK consumers, including mixed C#/FPAS/F# graphs,
  configuration mappings, generated/linked sources and no-restore failures.
- [x] Verify check/query CLI results, source/metadata definitions, generic properties,
  aliases, named parameters, linked-file ambiguity, scoped refs and truncation.
- [x] Verify stale dependency detection with build-input/output proof; edited source
  is rebuilt or rejected before a source mapping can be published.
- [x] Verify UTF-16 positions, overloads/shadowing, captures/flow, generics, contracts,
  patterns, attributes, events, interfaces and async/cleanup indexing.
- [x] Verify no inspection executes initializers, attribute constructors or getters.
- [x] Verify bounded MSBuild output, controlled running-task cancellation and real
  CLI SIGINT: one cancelled JSON document, exit 130 and terminated owned tasks.
- [x] Update current compiler/workspace architecture, CLI help/guide, agent reference,
  coverage and gap inventories; remove duplicate completed planning detail.
- [x] Run complete compiler/integration regressions, including packaged builds,
  emitted metadata/execution, documentation, incremental outputs and real MCP/DAP.
- [ ] Verify prepared **Windows** managed deployment and fresh SDK check/query
  workflows: restore/no-restore, configuration/linked-file selection, Unicode/space
  paths, changed inputs, limits, cancellation and job/process cleanup. Compilation
  on Linux does not establish native Windows acceptance.

Linux evidence: 687 compiler tests and the complete 513-test integration run pass.
A final 111-test integration regression covers the completed semantic fixes,
all 54 complete handbook/agent source examples, check/query, cancellation and
real MCP/DAP. Fresh SDK packaging and the documented check/definition/type/refs
commands also pass. Build has no warnings or errors. Retained logs/artifacts are
under `.temp-data/sa01/`.

Finish Windows acceptance before closing SA-07. Keep any unavailable platform
explicitly open.

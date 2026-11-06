# DAP and batch debugger frontends

Implement two frontends over the [shared CLR debugger core](future-debugging.md).
Default `fpas debug` to one-shot batch execution. `--protocol dap` hosts one IDE
session over stdio. Remove MCP and expose no JSONL mode, command-file runner,
REPL, HTTP listener or alternate backend.

All names and reports below are planned contracts. The
[acceptance plan](debugger-acceptance-plan.md) defines proof on both platforms.
Follow the [core implementation order](future-debugging.md#implementation-order).
The [batch command](../pascal/tools/batch-debugging.md) and DAP use the same core behavior; source mapping, stepping and
passive inspection must not acquire separate frontend implementations.

## F3: remove MCP and integrate the replacement

- [ ] Replace the CLI command/help/examples and `Fpas.Debugging` entry point.
  Remove MCP tools/catalogs/schemas, server hosting and the NetCoreDbg DAP client
  and launcher. Remove their test-only fixtures after moving useful behavior
  assertions to the new production interfaces.
- [ ] Remove `ModelContextProtocol` and debugger-only hosting dependencies from
  projects and central package pins after checking other consumers. Remove
  NetCoreDbg-only downloads/configuration from product and acceptance setup.
  Do not retain `--netcoredbg`, `--protocol mcp|jsonl` or compatibility aliases.
- [ ] Update `fpas env` to report the own debugger's packaged shim/runtime
  requirements and capabilities. Remove its NetCoreDbg option, environment
  selection, probe and schema entry; document the deliberate report change.
  Environment inspection must remain passive and must not start a debug target.
- [ ] Remove the obsolete `fpas-debug` registration from the known global Codex
  MCP configuration and repository examples during migration. Preserve unrelated
  registrations. Do not terminate another agent's active debugging session;
  report a loaded-binary deployment conflict separately.
- [ ] Update dist/SDK publication for RID-specific native assets and licenses,
  worker launch and offline runtime lookup. Verify ordinary managed deployment.
  Keep the future single-executable goal in [distribution gaps](dist-gap.md):
  select embedding/materialization of native assets rather than silently adding
  a required separately delivered helper executable.
- [ ] Replace the MCP architecture page with an own-debugger reference. Update
  debugging/CLI/environment guides, agent tools, architecture/layout/closures,
  docs navigation, development/test instructions, coverage and packaging assertions.
  Remove obsolete MCP acceptance history and registration walkthroughs.

Finish core behavior before product cutover. Keep current references
accurate during development. F3 completes only after every required
[acceptance row](debugger-acceptance-plan.md) passes on Windows and WSL. Move
implemented contracts to their references and remove completed plan entries.

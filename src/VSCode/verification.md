# Verify the VS Code extension

## Packaging and grammar

Run `npm ci`, `npm run build`, `npm run test:grammar` and `npm run package`
in a clean extension directory. The VSIX contains the manifest, bundled
entry point, language configuration, grammar, snippets, README, license and
dependency notices. Test and development files stay outside the package.

The grammar test runs the packaged TextMate rules with Oniguruma and checks
multiline string state, escaped identifiers and CLR-oriented FPAS tokens. It also
checks indentation rules for protocol and variant declarations.

## Fixture preparation and host runs

Prepare fixtures with the selected CLI and SDK feeds. Use a new ignored output
directory and the same isolated NuGet cache for fixture preparation and the
editor tests. The alternate feed supplies the SDK mismatch for recovery.
Use the release single-file launcher and its exported SDK in the Release run.

```powershell
node scripts/fixtures.mjs NEW_DIRECTORY FPAS Debug DEBUG_FEED RELEASE_FEED
npm test -- NEW_DIRECTORY trusted CODE_EXE EXTENSION.vsix
npm test -- NEW_DIRECTORY restricted CODE_EXE EXTENSION.vsix
```

For Remote-WSL, prepare fixtures inside WSL with the Linux toolchain. From
Windows, give the runner a VS Code executable and the extensions directory
containing the Microsoft WSL extension. Use native WSL paths for the extension
sources, fixtures and matching `code-server` executable. Use a test distro
without an existing installation of this extension.

```powershell
node scripts/test-remote.mjs CODE_EXE EXTENSIONS_DIR Ubuntu WSL_EXTENSION_DIR WSL_FIXTURE_DIR trusted EXTENSION.vsix WSL_CODE_SERVER
node scripts/test-remote.mjs CODE_EXE EXTENSIONS_DIR Ubuntu WSL_EXTENSION_DIR WSL_FIXTURE_DIR restricted EXTENSION.vsix WSL_CODE_SERVER
```

Repeat on each host in Debug and Release with the corresponding SDK feeds.
Cover the minimum VS Code version and current Stable patch. Without the VSIX
arguments, these runners load the development extension. With them, the runners
install the package, confirm the extension loads from the installed directory,
verify LSP shutdown and verify the extension is gone after uninstall and reload.

## Editor and project scenarios

The trusted run previews and applies the checked refactorings, checks stale-plan
rejection, undo and redo, and builds and executes the edited fixture.
It also checks diagnostics and formatting while editing a multiline string.
The project workflow fixtures combine a C# startup project and FPAS library in
one solution. Debugging checks source stops in both an FPAS executable and the
FPAS library launched through the C# host. The Testing view runs both a VSTest
library and a self-hosted Microsoft.Testing.Platform executable.
The receipts include the loaded bundle hash, host, Node and VS Code versions,
configuration, scenario results and the CLI/SDK identities in trusted runs.

Run the relevant .NET language server, debugger, project, test-runner and
distribution suites on each platform. See the
[development workflow](../../docs/build/development.md).

## Source files

<details>
<summary>Browse 24 source files</summary>

- `src/VSCode/scripts/`
  - [build.mjs](scripts/build.mjs)
  - [fixtures.mjs](scripts/fixtures.mjs)
  - [grammar.mjs](scripts/grammar.mjs)
  - [installed.mjs](scripts/installed.mjs)
  - [test-remote.mjs](scripts/test-remote.mjs)
  - [test.mjs](scripts/test.mjs)
  - [workflow-fixtures.mjs](scripts/workflow-fixtures.mjs)
- `src/VSCode/test/`
  - [editor-services.ts](test/editor-services.ts)
  - [host.ts](test/host.ts)
  - [refactorings.ts](test/refactorings.ts)
  - [scenarios.ts](test/scenarios.ts)
  - [support.ts](test/support.ts)
  - [workflow-debug.ts](test/workflow-debug.ts)
  - [workflow-projects.ts](test/workflow-projects.ts)
  - [workflow-testing.ts](test/workflow-testing.ts)
- `src/VSCode/test/fixtures/`
  - [helpers.fpas](test/fixtures/helpers.fpas)
  - [main.fpas](test/fixtures/main.fpas)
  - [probe.fpas](test/fixtures/probe.fpas)
  - [refactoring-dependency.fpas](test/fixtures/refactoring-dependency.fpas)
  - [refactoring.fpas](test/fixtures/refactoring.fpas)
  - [workflow-engine.fpas](test/fixtures/workflow-engine.fpas)
  - [workflow-host.cs](test/fixtures/workflow-host.cs)
  - [workflow-mtp.fpas](test/fixtures/workflow-mtp.fpas)
  - [workflow-vstest.fpas](test/fixtures/workflow-vstest.fpas)

</details>

# Verify the VS Code extension

Run `npm ci`, `npm run build`, `npm run test:grammar` and `npm run package`
in a clean extension directory. The VSIX contains the manifest, bundled
entry point, language configuration, grammar, snippets, README, license and
dependency notices. Test and development files stay outside the package.

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
The trusted run previews and applies all three refactorings, checks stale-plan
rejection, undo and redo, and builds and executes the edited fixture.
It also checks diagnostics and formatting while editing a multiline string.
The receipts include the loaded bundle hash, host, Node and VS Code versions,
configuration, scenario results and the CLI/SDK identities in trusted runs.

Run the relevant .NET language server, debugger, project, test-runner and
distribution suites on each platform. See the
[development workflow](../../docs/build/development.md).

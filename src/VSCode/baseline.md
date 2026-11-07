# Baseline and upstream review

Review date is 2026-11-07. The CLR starting point is `f521e127b` on `csharp-test`.
The local `main` is `0a4bc134e`. Remote `main` is one commit ahead at
`20748fbe8`. The comparison shows no changes under `editors/vscode` in that
commit, so the local extension sources match that part of the remote tree.
[Main comparison](https://github.com/tobiasbick/functional-pascal/compare/0a4bc134ee71b5f922ebd0b1157ab55244bef657...20748fbe867b16415b356e91a1b66308a9db1fc6).

## What to take from the old extension

The `0.3.0` extension is a useful reference for workflows and test cases.
`src/languageClient.ts` serializes startup, restart and disposal.
`src/toolchainSelection.ts` stores the selected executable in user settings.
`src/workflow/` and its tests cover project selection, Testing view requests and
terminal use. The `verify-grammar.mjs` and `verify-package.mjs` scripts check
actual TextMate tokenization and the contents of the VSIX. The older
refactoring tests include unsaved buffers and preview/apply flows.
[Main extension source](https://github.com/tobiasbick/functional-pascal/tree/20748fbe867b16415b356e91a1b66308a9db1fc6/editors/vscode).

| Area | Main extension | CLR implementation |
| --- | --- | --- |
| Host | `extensionKind` is `ui`. | Use the workspace host so the CLI and its projects share an OS, environment and filesystem. |
| Toolchain | Resolves `fpas`, `fpas-runner`, `fpas-lsp` and the VM standard library. | Probe the selected installed `fpas env --json`, then use `fpas lsp`, CLI project commands and `fpas debug --protocol dap`. |
| Project model | Reads `.fpasprj`, `.fpasworkspace` and TOML. | Select `.fpasproj` or `.slnx` and delegate membership, configurations and executable selection to the CLR project driver. |
| Syntax | Examples include `end.`, `integer` and `Std.Console`. | Derive grammar fixtures from the CLR handbook. Use `end program;`, `int` and `System.Console` where appropriate. |
| Source tools | Uses custom Rust-server requests and a refactor apply command. | Use standard LSP where available. Define new contracts for checked CLR source views and edit previews. |
| Debugger | Includes VM mutation, task control, input and reload commands. | Advertise the capabilities of the current CLR DAP session. Keep ordinary terminal runs a separate command. |
| Build and packaging | Already uses strict TypeScript, esbuild, real extension-host tests and verified VSIX packaging. | Keep those checks and use a lockfile, stable APIs and a platform-independent bundle. |

The [current language server](../../docs/pascal/tools/language-server.md)
requires an explicit context and uses LSP 3.17 over stdio with UTF-16 positions.
It already provides versioned diagnostics, definition, references, workspace
symbols, formatting and checked rename. The
[CLR debugger](../../docs/pascal/tools/debugging.md) accepts a built DLL.
Test discovery exposes a fully qualified name and optional MTP UID/display name.
Add source locations and result correlation at the .NET boundary where the
runner can establish them. See [test discovery](../../docs/pascal/tools/testing.md).

## Online changes that affect the plan

| Component | Verified publication | Plan |
| --- | --- | --- |
| VS Code | 1.145.2 is the stable download shown for the 1.145 release. [Release notes](https://code.visualstudio.com/updates/v1_145) | Set the first support floor to 1.145.0. Test that floor and the current stable patch. Use `@types/vscode` 1.145.0. [Type package metadata](https://registry.npmjs.org/@types/vscode/1.145.0) |
| `vscode-languageclient` | 11.0.0 depends on protocol 3.18.5. The old extension uses 10.1.2. The upstream changes include startup and shutdown cleanup, watcher disposal and document notification ordering. [Upstream history](https://github.com/microsoft/vscode-languageserver-node/blob/main/README.md#history) | Pin 11.0.0 and import `vscode-languageclient/node` through its published exports. Exercise it against the existing 3.17 server before adding features. Capabilities determine which requests the client sends. |
| Build | The registry reports TypeScript 7.0.2 and esbuild 0.28.2, which the old extension already uses. [TypeScript metadata](https://registry.npmjs.org/typescript/7.0.2), [esbuild metadata](https://registry.npmjs.org/esbuild/0.28.2) | Retain these pins for the first build. Use strict type checking and a separate Node bundle. Check the published client exports in the installed VSIX. |
| Extension testing | `@vscode/test-cli` 0.0.15 and `@vscode/test-electron` 3.1.0 require Node 22 or newer. [Test CLI metadata](https://registry.npmjs.org/@vscode/test-cli/0.0.15), [Test Electron metadata](https://registry.npmjs.org/@vscode/test-electron/3.1.0) | Use test-electron for the editor download and trusted launch. Launch Restricted Mode and Remote-WSL directly with separate profiles. |
| VSIX packaging | `@vscode/vsce` 4.1.0 requires Node 22 or newer. The old extension uses 4.0.0. [VSCE metadata](https://registry.npmjs.org/@vscode/vsce/4.1.0) | Pin 4.1.0. Use Node 22 or newer for development and packaging. Record the Node version in the acceptance run. |

The manifest's workspace host choice follows the VS Code extension-host model.
The desktop release will be tested with a Windows local workspace and an actual
Remote-WSL workspace. In Restricted Mode keep static language assets available
and gate every path that launches FPAS or evaluates a project.
[Extension host](https://code.visualstudio.com/api/advanced-topics/extension-host),
[Workspace Trust](https://code.visualstudio.com/api/extension-guides/workspace-trust),
[extension testing](https://code.visualstudio.com/api/working-with-extensions/testing-extension).

The extension uses the selected client, build and packaging versions in its
lockfile. The foundation checks used VS Code 1.145.1 on Windows and 1.145.2 in
Remote-WSL. The [verification guide](verification.md) covers the version floor
and installed package for the full editor workflow.

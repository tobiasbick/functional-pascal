# Functional Pascal CLR for VS Code

The `functional-pascal.functional-pascal-clr` extension registers `.fpas`
with CLR syntax highlighting, comments, indentation and snippets. Its language
client runs `fpas lsp` in the VS Code workspace host and supplies live diagnostics,
navigation, IntelliSense, semantic highlighting, formatting and checked edits.

## Install and select a project

Use VS Code 1.145.0 or newer and install the matching FPAS CLI, project SDK and
.NET 10 SDK on the workspace host. For Remote-WSL, open the folder in a WSL
window and install the VSIX on the WSL host. Use the Linux FPAS installation
and .NET environment there. [Toolchain distribution](../../docs/build/distribution.md).

To build a local VSIX, run these commands here with Node 22 or newer.

```sh
npm ci
npm run package
```

Use **Extensions: Install from VSIX** to install the resulting package. Use one
active FPAS extension per workspace. The CLR extension uses language ID `fpas`.

1. Restore the `.fpasproj` or `.slnx` explicitly with the matching SDK. Build
   changed foreign references before analyzing them.
2. Open an FPAS file. The extension starts automatically when one context is
   available. Use **Functional Pascal: Select Analysis Context** or the project
   file's Explorer action when there are several. Select the owning `.fpasproj`
   if the same source belongs to multiple compilations. The status item shows
   the context and build configuration. **Select Build Configuration** changes
   the configuration for the workspace.
3. The extension finds `fpas` on the workspace host's PATH. Use **Select FPAS
   Executable** or `functionalPascalClr.executablePath` in that host's machine
   settings for an absolute launcher path. Use **Show Language Server Output**
   for setup errors and **Restart Language Server** after changing the toolchain.
   If the compiler and SDK differ, export the SDK from the chosen launcher and
   restore the project with that package. Run `fpas env --json` on the workspace
   host to inspect the installed toolchain.
4. Enable VS Code's `editor.formatOnSave` for `[fpas]` and select this extension
   as `editor.defaultFormatter` if several formatters are available. The server
   uses the fixed FPAS style and the current buffer. [Language server contract](../../docs/pascal/tools/language-server.md).

Restricted Mode provides the static language features. Trust the workspace
to enable toolchain commands and project analysis. For protocol tracing, set
the Functional Pascal log level to Trace. `functionalPascalClr.trace.server`
chooses messages or verbose output. Verbose traces include document text.

## Explore code and apply corrections

The outline and folding commands use the current syntax tree, including independent
declarations in a partially typed file. Type definitions and highlights use
checked bindings. Hover, completion and signature help show source and CLR
signatures, including selected generic substitutions and available documentation.
Semantic tokens supplement TextMate in checked regions. Call hierarchy shows
proven direct source calls and reports partial coverage in the language-server log.

Use **Functional Pascal: Show Public Signatures** or **Inspect Declaration** to
open a read-only view of the current buffer. **Search Referenced CLR Metadata**
opens a picker for metadata declarations and available XML documentation.
The views become stale after a change to their source or watched project inputs.
Quick fixes appear in VS Code's lightbulb and recheck the selected correction
against the live project before applying it to the current buffer.

## Project, test and debug workflows

Use the Functional Pascal project commands or tasks to check, restore, build,
clean, run, test and format the selected context. Choose the startup project and
test project separately. The status items show the context and configuration.
The extension asks before using disk files when an open source has unsaved changes.

Run opens an integrated terminal with program input. Set run arguments and the
working directory in the extension settings or in a task with type `fpas`.
Use `captured: true` for a run task that collects bounded output and supplies EOF
on program stdin. Explicit check and build diagnostics appear in Problems
with their project, source range and related locations when available.

Use **Refresh Tests** to discover existing VSTest and MTP outputs. **Build Test
Outputs** builds the context and refreshes the Testing view. VSTest data rows
share a method item. MTP uses the runner's test UIDs. The Testing view keeps
native TRX artifacts and shows an incomplete result if identities cannot be
matched. Build and refresh after test sources change.

Use a launch configuration of type `fpas-clr` for the selected executable
project, or set `program` to an absolute built DLL. Set `build: true` on a project
launch to build first. Portable symbols and source checksums must match.
The debugger uses closed stdin. Use Run for programs that need terminal input.
The adapter supports source breakpoints, stepping, threads, scopes, passive
evaluation and exception filters. See the [CLR debugger reference](../../docs/pascal/tools/debugging.md).

## Preview and apply a refactoring

Place the cursor on a private routine or one of its calls for **Preview Add
Parameter**. Enter a parameter name, FPAS type and expression for each call.
For **Preview Extract Variable**, select the complete value expression and enter
the new constant's name. **Preview Organize Imports** removes unused CLR
imports where the compiler can prove the remaining bindings are preserved.

Review the proposed changes, then choose **Apply Refactoring** or **Discard
Refactoring** in the preview toolbar. Apply changes the open buffers and
supports editor undo. Save when ready. Open all affected files before preparing
an applicable preview. Reprepare after an input or toolchain change. See the
[checked refactoring reference](../../docs/pascal/tools/refactoring.md) for limits.

See the [verification guide](verification.md) for testing a development build
or the installed VSIX.

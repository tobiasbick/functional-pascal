# Functional Pascal

Language support, project tools, and source debugging for Functional Pascal in
VS Code-compatible desktop editors. Diagnostics, completion, formatting, and
debugging use your installed FPAS toolchain.

## Get started

1. Install the extension through **Extensions: Install from VSIX**.
2. Install the [FPAS toolchain](https://github.com/tobiasbick/functional-pascal#build-from-source).
   Keep `fpas`, `fpas-lsp`, and the source standard library (`lib/`) together.
   The distribution also includes `fpas-runner` for building native executables.
3. Add the toolchain directory to `PATH`, or run **Functional Pascal: Select
   FPAS Executable** from the Command Palette and select `fpas` (`fpas.exe` on
   Windows).
4. Open your local project folder. Open a program's `.fpas` source, `.fpasprj`,
   or `.fpasworkspace`, then press **F5** to debug it.

The extension requires VS Code 1.91 or newer and a trusted local workspace.
The VSIX contains the editor integration; install the compiler, language server,
runner, and standard library separately. The selected toolchain supplies all
language and runtime services.

## Settings

Open Settings and search for `@ext:functional-pascal.functional-pascal`.

| Setting | Default | Purpose |
| --- | --- | --- |
| `functionalPascal.executablePath` | Empty | Absolute path to `fpas`. When empty, searches `PATH`. This setting applies to the local machine. |
| `functionalPascal.programTerminal` | `integratedTerminal` | Choose the editor terminal or a separate OS window (`externalTerminal`) for program runs and F5. |
| `functionalPascal.testTimeoutSeconds` | `10` | Per-test timeout in seconds for the Testing view. |

For example, use a separate terminal window and format FPAS sources on save:

```json
{
  "functionalPascal.programTerminal": "externalTerminal",
  "[fpas]": {
    "editor.formatOnSave": true
  }
}
```

Selecting another executable restarts the language server automatically. Its
standard-library location comes from `fpas env --json`.

## Editing

| Feature | What you get |
| --- | --- |
| Diagnostics and quick fixes | Parser and type errors for unsaved source, compiler-supplied fixes, and unambiguous import fixes. |
| Completion and help | Parameter and type previews, colored signatures, context-aware names, generic record fields, native operations, aliased imports, signature help, and snippets. |
| Navigation | Colored, multiline hover with documentation and source links; definition, type definition, references, Outline, workspace symbols, and read/write highlights. |
| Refactoring | Validated symbol rename, local variable extraction, import organization, and syntax-aware selection expansion. |
| Formatting and highlighting | The canonical `fpas fmt` style, semantic highlighting, and TextMate highlighting during startup. |
| Public signatures | **Show Public Signatures** opens a read-only view of a unit's public declarations and contracts, updated as you edit. |

The language server discovers nested projects and workspaces in the opened
folder and follows their dependencies, visibility, and exports. Unsaved buffers
take precedence over disk contents. Source and manifest changes refresh analysis
without a server restart.

**Format Document** and format on save use the same formatter as the CLI:
two-space structural indentation and a 100-column expression target. A source
with parser errors is left unchanged.

Use **F12** or **Ctrl+Click** to open a resolved declaration (**Cmd+Click** on
macOS; **Alt+Click** when Ctrl/Cmd is configured for multiple cursors).
Select a complete expression and choose **Refactor... > Extract variable** to
introduce a local `const`; use **F2** to give it your preferred name. Run
**Organize Imports** to sort imports and remove safely unused aliases. Both
actions check the result before offering an edit. Expressions whose evaluation
would move across other work, and import clauses containing comments, are kept
unchanged. See the [refactoring rules](https://github.com/tobiasbick/functional-pascal/blob/main/docs/pascal/tools/editor-integration.md#refactoring).

## Projects and tests

Run **Functional Pascal: Select Project or Workspace** when the folder contains
several manifests. The active selection appears in the status bar.

All commands below have the **Functional Pascal:** prefix in the Command Palette.

| Command | Action |
| --- | --- |
| **Check Project** / **Build Project** | Check or compile the selected project and show failures in Problems. |
| **Run Project in Terminal** | Run interactively; program arguments are entered as a JSON string array, such as `["--help"]`. |
| **Test Project** / **Refresh Tests** | Run or rediscover project tests in the Testing view. |
| **Format Project** / **Check Project Formatting** | Format project sources or check their formatting. |
| **Cancel Active Operation** | Cancel the current check, build, formatting operation, or test run. |

The Testing view discovers `*_test.fpas` files. Run all tests, selected files,
or a filtered set, and rerun failures. Results distinguish assertion failures,
skips, compile errors, runtime errors, and timeouts.

## Debugging

Press **F9** or click the editor gutter to set a breakpoint, then **F5** to
start. When the active source is a project's program main, F5 launches its
owning `.fpasprj` so project dependencies are available.

Use the normal Debug toolbar to continue, pause, step, and stop. The Run and
Debug view shows task threads, call stacks, scopes, variables, and watches.
Conditional breakpoints, positive-integer hit counts, and logpoints are
supported. Logpoints interpolate `{expression}`; `{{` and `}}` insert braces.

### Launch configuration

F5 works without a `launch.json`. For a fixed target or program arguments,
create a configuration like this:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "fpas",
      "request": "launch",
      "name": "Debug FPAS project",
      "program": "${workspaceFolder}/app.fpasprj",
      "args": [],
      "stopOnEntry": false
    }
  ]
}
```

Replace `app.fpasprj` with your program project, `.fpas` source, or
`.fpasworkspace`. Compiled `.fpascp` images also require `sourceRoot`.
The optional `cwd` sets the working directory. An explicit `console` value
(`integratedTerminal`, `externalTerminal`, or `debugConsole`) overrides the
terminal setting for this configuration.

### Terminal and program input

Integrated and external debug terminals support interactive `Std.Console` and
`Std.Tui` programs, including keyboard, mouse, paste, focus, and resize events.
`debugConsole` provides output-only operation. Expressions entered in the
Debug Console inspect stopped state; they are not program input.

Use **Debug: Send Program Input** to queue a line for `ReadText`/`ReadLn`.
**Debug: Signal Program Input EOF** and **Debug: Cancel Queued Program Input**
control the input queue.

### Inspect and change stopped state

Watch, hover, and Debug Console evaluation support expressions and controlled
deterministic calls. Calls use a detached copy of stopped state; their writes
are discarded, and calls with external effects are rejected.

Use **Set Value** in Variables or Watch to edit supported mutable values.
Command Palette actions cover dictionary entries, array insertion/removal,
string characters, variant construction, empty storage initialization, and
replacement of an unconsumed completed task result.

Tasks appear separately in the Threads view. Select a task to inspect its stack
or step it. Stops pause all tasks; **Debug: Pause Task**, **Debug: Resume Task**,
and **Debug: Cancel Task** control an individual task's execution state.

**Restart Frame** restarts the selected live frame with its current parameters
and captures. **Debug: Force Return** completes a selected frame, including a
program or task entry; completing the program entry ends the session.

**Debug: Reload Compatible Changes** rebuilds the launch target and applies
changes to inactive function bodies. **Debug: Roll Back Last Reload** restores
the preceding image. Changes to active bodies, layouts, captures, or the
function set require a new session.

See the [editor handbook](https://github.com/tobiasbick/functional-pascal/blob/main/docs/pascal/tools/editor-integration.md#source-debugging-and-variable-editing)
for supported edits and task operations, and the
[debugger reference](https://github.com/tobiasbick/functional-pascal/blob/main/docs/pascal/tools/debugger.md)
for evaluation and reload rules.

## Troubleshooting and limits

- Run **Functional Pascal: Show Output** to inspect startup and toolchain errors.
  Check the selected installation with `fpas env --json`.
- Run **Functional Pascal: Restart Language Server** to restart editor analysis.
- Remote SSH, WSL, and container workspaces are unsupported. Open the project
  in a local desktop extension host.
- Debugging supports launch sessions. Attach, reverse execution, native memory
  inspection, and Set Next Statement are unsupported.

The extension collects no telemetry and submits no reports automatically.
Use the [bug-report template](BUG_REPORT.md) for a reproducible problem.

## Build the extension

Contributors need Node.js 22 or newer. From the FPAS repository root:

```sh
npm ci --prefix editors/vscode
npm run package --prefix editors/vscode
```

The result is `editors/vscode/dist/functional-pascal-<version>.vsix`.
Packaging compiles the extension and verifies the archive contents. It does
not build FPAS or run integration tests.

To build FPAS and run the full extension suite before packaging, use the
repository's pinned Rust toolchain and run:

```sh
npm run package:verified --prefix editors/vscode
```

A failed test stops packaging and preserves an existing VSIX.
Run `npm test --prefix editors/vscode` for tests alone, or
`npm run test:restricted --prefix editors/vscode` for Restricted Mode.
The test editor is pinned in `scripts/run-tests.mjs`;
`FPAS_VSCODE_TEST_VERSION=1.91.0` selects the minimum supported version.

See the [editor handbook](https://github.com/tobiasbick/functional-pascal/blob/main/docs/pascal/tools/editor-integration.md)
for the full integration contract and implementation references.

# Editor integration

Functional Pascal has a repository-owned extension for VS Code-compatible
desktop editors. The implemented editor features are:

- `.fpas` language detection and TextMate syntax highlighting
- comment, bracket, indentation, and folding configuration
- parser and semantic diagnostics for the current unsaved buffer
- canonical whole-document formatting
- hierarchical document symbols
- declaration hover and go to definition
- basic visibility-aware completion
- language-server restart and output-channel commands

The extension and native language server live under
[`editors/vscode/`](../../../editors/vscode/) and
[`crates/fpas-lsp/`](../../../crates/fpas-lsp/). They use standard VS Code
extension and Language Server Protocol APIs; no compiler behavior is
reimplemented in TypeScript.

## Current packaging boundary

Repository development mode starts the locally built `fpas-lsp` executable and
has verified diagnostics and formatting in a real VS Code Extension Host. The
current bootstrap VSIX still omits the native executable, so it provides the
language registration and syntax layer only. A host-native self-contained VSIX
is a later packaging phase.

Build and test the current development extension from the repository root:

```text
npm ci --prefix editors/vscode
npm test --prefix editors/vscode
```

The package command and current bootstrap artifact are documented in the
[extension README](../../../editors/vscode/README.md).

## Diagnostics

Opening or changing a local `.fpas` document analyzes the in-memory editor
version. The server publishes lexer/parser diagnostics and, when parsing is
sufficiently valid, semantic diagnostics.

Each diagnostic preserves its stable `Fxxxx` code, error or warning severity,
UTF-16 editor range, and compiler help text. Analysis is briefly debounced
during typing. Results carry the exact document version, and superseded work
is discarded. Correcting the source or closing the document clears stale
diagnostics.

Project discovery uses the same `.fpasprj` and `.fpasworkspace` model as the
compiler. Open dependency units are analyzed with their own URI and source
ranges.

## Formatting

**Format Document** formats the current unsaved buffer through the same
`fpas-fmt` implementation as `fpas fmt`. Formatting is canonical: editor tab
size or indentation preferences do not create a second style.

The language server returns a whole-document edit when the text changes and no
edit when it is already canonical. Parser errors return no edit, preventing a
recovered or partial syntax tree from destructively replacing the source.
Comments are preserved according to the [formatter style](fmt-style.md).

The editor's standard format-on-save setting works without an FPAS-specific
watcher or extension setting. For example:

```json
{
  "[fpas]": {
    "editor.formatOnSave": true
  }
}
```

## Navigation

The language server provides hierarchical document symbols for programs,
units, types, routines, parameters, variables, enum members, and record
members. Symbol ranges cover the declaration, while selection ranges identify
the declared name exactly.

Hover shows the resolved source declaration. **Go to Definition** works for
declarations and references in the same file and across units in the loaded
project. Project navigation follows FPAS rules for lexical shadowing, direct
`uses` imports, public declarations and record members, qualified unit names,
and library `[exports].units`.

Basic completion lists declarations visible at the cursor. After `.` it lists
visible unit or record members. Equal candidates imported from different units
remain distinct so the editor can present their qualified owners. Queries use
the current unsaved buffers of every open project source.

Comments, string contents, unknown names, inaccessible declarations, and
sources outside the loaded project produce no navigation result. Recovered or
incomplete syntax may produce a partial symbol/completion result, but does not
fail the language server.

## Current limits

Completion is intentionally declaration-oriented; signature help, references,
rename, semantic tokens, and code actions are not implemented. Remote SSH,
WSL, and container extension hosts are outside the local hobby-project
packaging contract.

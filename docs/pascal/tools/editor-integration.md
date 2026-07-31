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

## Build and installation

Node.js 22 or newer and a stable Rust toolchain are required. From the
repository root, install the pinned Node dependencies once and build the
extension:

```text
npm ci --prefix editors/vscode
npm run package --prefix editors/vscode
```

The package command is non-interactive. It runs the extension tests, builds
`fpas-lsp` in Cargo release mode, stages the current host binary together with
the authoritative source-standard-library manifest and `.fpas` files, verifies
the archive contents, and exercises the server extracted from the resulting
VSIX. Derived `.fpascu` files are not packaged. The output is:

```text
editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix
```

Install that file through **Extensions: Install from VSIX** in a compatible
desktop editor. The VSIX is self-contained for the operating system and
architecture where it was built. Users on another host build the same source
there; the hobby project does not cross-compile, publish, or maintain a release
matrix.

For development, `npm test --prefix editors/vscode` builds the debug language
server and runs the real VS Code Extension Host checks. More detail is in the
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

The opened editor folder does not have to be an FPAS project. For each opened
`.fpas` source, the server searches its parent directories up to that folder
and loads the nearest `.fpasprj` or `.fpasworkspace` that directly owns the
source. Multiple nested FPAS projects can therefore coexist inside a larger
Rust repository. A direct source owner takes precedence over a project that
only consumes the source through a dependency.

Discovery uses the same manifests, source ownership, dependencies, workspace
members, exports, and standard-library rules as the compiler. It does not scan
the whole repository recursively. A file without a matching manifest remains
a loose file, while overlapping direct owners produce an actionable
ambiguity error. Open dependency units are analyzed with their own URI and
source ranges.

The VSIX supplies its bundled source standard library to every loaded project
and loose document. Source-defined units such as `Std.Tui` therefore work even
when the opened project is outside the Functional Pascal repository. Projects
do not need to declare that implementation-owned library as a dependency, and
editor analysis does not write compiled-unit sidecars.

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

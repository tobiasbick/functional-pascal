# Editor integration

Functional Pascal has a repository-owned extension for VS Code-compatible
desktop editors. The implemented editor features are:

- `.fpas` language detection and TextMate syntax highlighting
- comment, bracket, indentation, and folding configuration
- parser and semantic diagnostics for the current unsaved buffer
- canonical whole-document formatting
- hierarchical document symbols
- declaration hover, go to definition, go to type definition, and find all references
- workspace symbol search and same-document read/write highlights
- syntax-aware selection-range expansion
- validated project-wide symbol rename
- rich visibility-aware completion with lazy declaration documentation
- signature help, repository snippets, and unambiguous auto-import completion
- compiler-backed semantic highlighting with TextMate fallback
- deterministic import quick fixes for eligible `F2001` and `F2003` diagnostics
- project check, build, run, test, format, and format-check workflows
- Problems, Testing view, cancellation, active-project status, and terminal runs
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

## Project workflows

The extension bundles the host-native `fpas` CLI beside `fpas-lsp`; it never
depends on a compiler from `PATH`. Before the first workflow operation it checks
the CLI's stable `fpas --version` output. A missing or incompatible executable
produces one actionable recovery message without stopping language-server
features.

Use **Functional Pascal: Select Project or Workspace** to choose a
`.fpasprj` or `.fpasworkspace`. A folder with one manifest selects it
automatically. When multiple manifests exist, the extension never chooses the
first match: selection is explicit and remembered for the editor workspace.
The status bar shows that manifest and changes to the active operation while a
command runs.

The Command Palette provides **Check Project**, **Build Project**, **Test
Project**, **Format Project**, **Check Project Formatting**, **Run Project in
Terminal**, and **Cancel Active Operation**. Non-interactive commands invoke the
bundled CLI without a shell, so paths and arguments containing spaces remain
separate. Cancellation terminates the owned CLI process. Run uses an editor
terminal because interactive console, TUI, and graphical programs retain their
normal host interaction; program arguments are entered as a JSON string array
and forwarded after `--`.

Compiler output becomes a dedicated `fpas workflow` Problems collection with
the real source URI, source position, severity, stable `Fxxxx` code, message,
and help. The Testing view discovers cases through `fpas test --list` and runs
them through the versioned `fpas test --report json` contract. Run, rerun, all,
and selected/filtered runs preserve pass, assertion failure, skip, compile
error, runtime error, and timeout as distinct results. The per-test timeout is
configured with `functionalPascal.testTimeoutSeconds` and defaults to 10
seconds.

## Diagnostics

Opening or changing a local `.fpas` document analyzes the in-memory editor
version. The server publishes lexer/parser diagnostics and, when parsing is
sufficiently valid, semantic diagnostics.

Each diagnostic preserves its stable `Fxxxx` code, error or warning severity,
UTF-16 editor range, and compiler help text. Analysis is briefly debounced
during typing. Results carry the exact document version, and superseded work
is discarded. Correcting the source or closing the document clears stale
diagnostics.

The opened editor folder does not have to be an FPAS project. At startup, the
server builds a bounded catalog of `.fpasprj` and `.fpasworkspace` manifests
inside that folder. It asks the normal project loader for the authoritative
source sets, dependencies, workspace members, and exports; it does not treat
every recursively found `.fpas` file as a loose project source. Multiple nested
FPAS projects can therefore coexist inside a larger Rust repository. A direct
source owner takes precedence over a project that only consumes the source
through a dependency.

Discovery uses the same manifests, source ownership, dependencies, workspace
members, exports, and standard-library rules as the compiler. Generated and
dependency directories such as `.git`, `target`, `node_modules`, and
`.vscode-test` are excluded from catalog traversal. A file without a matching
manifest remains a loose file, while overlapping direct owners produce an
actionable ambiguity error. Open dependency units are analyzed with their own
URI and source ranges.

The extension watches `.fpas`, `.fpasprj`, and `.fpasworkspace` files inside
the opened folder. Create, change, rename, and delete notifications rebuild the
manifest catalog and invalidate affected analysis results without restarting
the server. Unsaved open buffers remain authoritative over disk changes.

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

**Go to Symbol in Workspace** (`Ctrl+T`) searches declarations from every
cataloged project plus current unsaved buffers. Matching is case-insensitive
and ranks exact short names before prefixes, substrings, and qualified-name
matches. Results have stable ordering, retain equal short names from distinct
owners, and are limited to 100 entries per query.

Hover shows the resolved source declaration. **Go to Definition** works for
declarations and references in the same file and across units in the loaded
project. Project navigation follows FPAS rules for lexical shadowing, direct
`uses` imports, public declarations and record members, qualified unit names,
and library `[exports].units`. For hierarchical unit names, navigation checks
every matching imported owner and returns a target only when the complete
qualified identity resolves unambiguously.

**Go to Type Definition** follows the named source type of variables,
parameters, record fields and properties, function results, and aliases. It
uses the same import, qualification, visibility, and record-member resolution
as definition navigation. Built-in, unknown, or inaccessible types have no
source target and therefore return no result.

Selecting a resolved identifier highlights its declaration, reads, and direct
assignment writes in the current document. Lexically shadowed declarations,
comments, and strings are excluded. Editor selection expansion grows from the
token through enclosing declarations and statements to the routine, type, and
compilation-unit boundaries. Malformed source is restricted to its recovered
token boundary so expansion does not jump across unreliable syntax.

**Find All References** (`Shift+F12`) returns declaration and usage locations
for the resolved symbol across every indexed project whose dependency and
library-export graph makes that declaration visible. This includes a program
that consumes a directly owned sibling library project. The search preserves
lexical shadowing and ignores matching text in comments and strings.

**Rename Symbol** (`F2`) validates the replacement as a non-keyword ASCII FPAS
identifier, rejects declaration conflicts and lexical capture or shadowing,
and returns one workspace edit for the declaration and every resolved usage in
those indexed projects. It edits current unsaved snapshots of open files.
Program and unit names are excluded because a correct rename would also have to
rename source files or manifests. A declaration outside the opened editor
folder, including a standard library bundled with an installed VSIX, is never
modified.

Completion lists parameters, locals, visible unit declarations, record and enum
members, and keywords appropriate to the recovered source context. Each item
reports its declaration kind, qualified owner, type or callable signature,
stable sorting, and the exact identifier range it replaces. This replacement
keeps punctuation and surrounding Unicode strings or comments unchanged.
Declaration documentation is loaded only after the editor resolves a selected
item. Equal candidates imported from different units remain distinct so the
editor can present their qualified owners. Private, shadowed, and non-exported
declarations are excluded.

When one unresolved identifier maps to exactly one public declaration in one
accessible unit, completion can add that unit to the compilation unit's `uses`
clause. The edit is produced through the canonical formatter and is withheld
when the declaration or unit is ambiguous, inaccessible, already visible, or
the existing clause cannot be edited conservatively.

Signature help covers functions, procedures, record methods, nested routines,
function values, enum constructors with associated values, and generic calls.
It tracks the active argument through nested and multiline expressions. The
extension also contributes parser- and formatter-checked snippets for programs,
units, routine and record declarations, variables, branches, and common loops.
Type a snippet prefix such as `function`, `record`, `if`, or `for` and select
the snippet item from completion.

Queries use the current unsaved buffers of every open project source. Recovered
or incomplete syntax can still provide candidates when its lexical and symbol
context is reliable.

## Semantic highlighting and quick fixes

The server emits full-document semantic tokens for resolved units, types,
enums, type parameters, functions, procedures, methods, parameters, variables,
fields, properties, events, enum members, and constants. Declaration,
read-only, and public modifiers are emitted only when the resolved declaration
proves them. Token ranges use UTF-16 positions, preserve lexical ordering, and
remain non-overlapping. Recovered malformed source can return a safe partial
result. The TextMate grammar remains active before server startup and whenever
semantic analysis has no token for a source region.

**Quick Fix** (`Ctrl+.`) can add a unit to `uses` for an unknown type (`F2001`)
or unknown callable (`F2003`) when the project contains exactly one accessible
public declaration that supplies it. The action is associated with the current
diagnostic and is offered only if the edit still matches that diagnostic and
produces parseable, canonically formatted source. Applying it re-analyzes the
unsaved document and clears the resolved diagnostic without a server restart.
Ambiguous, inaccessible, changed, malformed, and stale inputs receive no edit.
Compiler help that only explains an error is not presented as an automatic fix.

Comments, string contents, unknown names, inaccessible declarations, and
sources outside the loaded project produce no navigation result. Recovered or
incomplete syntax may produce a partial symbol/completion result, but does not
fail the language server.

## Current limits

Semantic-token delta responses are not implemented; the server sends complete
document token sets. The only current code-action family is an unambiguous
`uses` import for eligible unknown-type and unknown-callable diagnostics.
Remote SSH, WSL, and container extension hosts are outside the local
hobby-project packaging contract.

## Reporting editor problems

The extension has no telemetry and does not submit reports automatically. For
every reproducible problem, copy the local
[editor bug-report template](../../../editors/vscode/BUG_REPORT.md), replace
its placeholders, and save it as a local note or paste it into a repository
issue. Include the smallest source that reproduces the behavior and the
sanitized **Functional Pascal** output-channel excerpt.

## Implementation references

The pinned SDK, packaging, highlighting, testing, language-client, LSP, and
Rust transport sources are collected in
[editor implementation references](editor-references.md). These links are
implementation inputs; repository tests remain authoritative for current
behavior.

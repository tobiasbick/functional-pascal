# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding. Its bundled native `fpas-lsp` server
provides parser and semantic diagnostics, canonical whole-document formatting,
document symbols, hover, same- and cross-unit go to definition, find all
references, workspace symbol search, document highlights, go to type
definition, syntax-aware selection expansion, validated project-wide rename,
rich visibility-aware completion, lazy completion documentation, signature
help, checked FPAS snippets, and safe unambiguous auto-imports. The repository
builds and tests the extension without a Marketplace. Compiler-backed semantic
tokens distinguish resolved declarations and references, and an `F2001` or
`F2003` diagnostic can offer a safe import quick fix when exactly one public,
accessible unit provides the missing type or callable.
The same VSIX bundles the host-native `fpas` CLI for project check, build, run,
test, format, and format-check commands, Problems integration, and the Testing
view.
The extension also contributes the `fpas` debugger. Use **Run and Debug** with
the generated **Debug Functional Pascal** configuration, or set `program` to a
`.fpas`, program `.fpasprj`, `.fpasworkspace`, or `.fpascp` target. Compiled
images additionally require `sourceRoot`. Set source breakpoints in an `.fpas`
editor gutter or with **F9**. Breakpoints, stepping, stack frames, scopes,
variables, read-only watches/hover/Debug Console evaluation, conditional
breakpoints, exact positive-integer hit conditions, non-stopping logpoints,
and program output use the bundled CLI's DAP adapter; the
adapter supplies the bundled source standard library automatically. The
language server remains responsible only for static editor features.
Use the Debug toolbar for Continue, Pause, Step Into, Step Over, Step Out, and
Stop. The Run and Debug sidebar exposes the call stack, lexical scopes, locals,
parameters, globals, and expandable aggregate values. Evaluated aggregates are
also expandable until execution resumes. Program output, logpoint text, and
structured runtime failures appear in the Debug Console. Log messages use
`{expression}` interpolation and `{{`/`}}` for literal braces. Debugger-side
calls may invoke deterministic functions, procedures, record methods,
constructors, readable properties, visible closures, and pure `Std.*`
intrinsics. They run against a detached copy of stopped state; writes are
discarded, and calls involving output, files, processes, environment, time,
randomness, blocking, tasks, or unknown dynamic effects are rejected.
While stopped, the Variables view can edit mutable locals, parameters, globals,
closure captures, record fields, array elements, existing dictionary
values, active enum payload fields, and `Result`/`Option` `.value` children.
Rejected edits leave the session stopped and unchanged. A successful
edit refreshes the Variables view; continuing execution observes the committed
value. Dictionary keys are not edited through the standard Variables request;
immutable or uninitialized bindings, evaluation-only
results, inactive enum or wrapper variants, task values, and opaque
host values are not editable. Task debugging is deterministic and all-stop;
attach remains unsupported.

Use **Functional Pascal: Debug: Insert Dictionary Entry**, **Debug: Remove
Dictionary Entry**, or **Debug: Replace Dictionary Key** while stopped to
change dictionary structure. The commands prompt for a complete mutable
dictionary target and FPAS key/value expressions. Insert appends a missing
pair, remove deletes an existing pair, and key replacement preserves the
associated value and iteration position. Failures and cancelled prompts leave
the stopped program unchanged.

Use **Functional Pascal: Debug: Insert Array Element**, **Debug: Remove Array
Element**, or **Debug: Replace String Character** for bounded sequence changes.
Array insertion accepts indexes from zero through the current length; removal
uses an existing zero-based index. String indexes count Unicode characters and
the replacement must be a one-character FPAS string. Successful commands
refresh debugger variables; failures and cancelled prompts send no mutation.

## Build

Node.js 22 or newer and a stable Rust toolchain are required. From the
repository root, install the pinned Node dependencies once:

```text
npm ci --prefix editors/vscode
```

Then build the VSIX:

```text
npm run package --prefix editors/vscode
```

The command runs the extension tests, builds `fpas-lsp` and `fpas` in Cargo
release mode, stages both current-host binaries plus the authoritative source-standard-library
manifest and `.fpas` files, creates the target-labelled archive, and tests an
external FPAS project through the server extracted from that archive. Derived
`.fpascu` files are excluded. It produces:

```text
editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix
```

For example, a Windows x64 host produces a `win32-x64` package and a Linux x64
host produces a `linux-x64` package. There is no cross-compilation or release
matrix; build the VSIX on each operating system and architecture where it will
be used.

Install the resulting file through **Extensions: Install from VSIX** in a
VS Code-compatible desktop editor. No registry login or publication is
required.

## Verify

Open a `.fpas` file and confirm the status bar identifies the language as
**Functional Pascal**. Syntax highlighting works before the extension's
TypeScript entry point is activated.

Introduce a syntax or type error and confirm the editor reports an `Fxxxx`
diagnostic for the unsaved buffer. Run **Format Document** and confirm the
result matches `fpas fmt`. The editor's standard `editor.formatOnSave` setting
uses the same formatter without an FPAS-specific setting.

Open the Outline view to inspect FPAS declarations. A contiguous standalone `//` block immediately
before a declaration is Markdown documentation; hover and resolved completion items display it.
The packaged standard library also contains editor-only declarations for Rust-backed intrinsic
`Std.*` units. They provide the same hover, completion, signature, and definition experience and
open as ordinary read-only `.fpas` files without becoming part of program compilation.
Hover a declaration or
reference, use **Go to Definition** or **Go to Type Definition**, search all
project declarations with **Go to Symbol in Workspace** (`Ctrl+T`), and invoke
completion in a routine body or after a unit/record `.`. Resolved identifiers
highlight their declaration, reads, and writes in the current document.
Selection expansion follows enclosing FPAS syntax. Project-aware results use
the same `.fpasprj`,
`.fpasworkspace`, visibility, and library-export boundaries as the compiler.
Use **Find All References** (`Shift+F12`) to list resolved declarations and
usages, including uses in indexed programs that consume a directly owned
library, and **Rename Symbol** (`F2`) to validate and edit a normal declaration
across those loaded projects. Program/unit renames and declarations outside
the opened folder are intentionally rejected.

Completion includes parameters, locals, imported declarations, record and enum
members, and context-appropriate keywords with accurate kinds, owners, types,
signatures, and replacement ranges. Signature help tracks nested and multiline
calls. Type a prefix such as `function`, `record`, `if`, or `for` to select a
repository-owned snippet. A completion may add a `uses` entry only when one
accessible public declaration has one unambiguous unit import; ambiguous or
inaccessible names are never guessed.

Semantic highlighting refines the TextMate colors for resolved units, types,
enums, type parameters, routines, parameters, variables, members, enum values,
and constants. TextMate highlighting remains the startup and recovery fallback.
Use **Quick Fix** (`Ctrl+.`) on an unknown type or callable diagnostic to add a
`uses` import when that edit is uniquely determined and the resulting source is
parseable and canonically formatted. Stale, ambiguous, or inaccessible
diagnostics produce no edit.

The folder opened in the editor may be the complete Functional Pascal Rust
repository or another parent folder without an FPAS manifest. The server
catalogs the `.fpasprj` and `.fpasworkspace` manifests in that folder and uses
the normal project loader to determine their sources and relationships.
Multiple nested FPAS projects can be used in one editor session; files without
a matching manifest remain available as loose files. External source and
manifest changes refresh affected analysis, references, and rename results
without a language-server restart, while unsaved open buffers remain
authoritative.

The installed VSIX supplies its own source standard library to the server, so
`Std.Tui` and the other source-defined `Std.*` units do not depend on a global
compiler installation or a `lib/` directory in the opened project.

Select **Functional Pascal: Select Project or Workspace** before using project
commands in a folder containing multiple manifests. The remembered selection
appears in the status bar. **Check Project**, **Build Project**, **Test
Project**, **Format Project**, and **Check Project Formatting** run the bundled
CLI without a shell and publish compiler failures in Problems. **Cancel Active
Operation** stops a running non-interactive command. **Run Project in Terminal**
starts the normal interactive CLI in an editor terminal and accepts program
arguments as a JSON string array.

The Testing view discovers `*_test.fpas` files for the selected manifest and
supports all, selected, filtered, and rerun requests. Outcomes distinguish
pass, assertion failure, skip, compile error, runtime error, and timeout. Set
`functionalPascal.testTimeoutSeconds` to change the default 10-second per-test
limit.

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated.
```

The test command builds `target/debug/fpas-lsp[.exe]`, starts it from a real
VS Code Extension Host, verifies diagnostics, formatting, document symbols,
hover, cross-unit definition and type definition, workspace symbols, document
highlights, references, rename, rich completion, signature help, snippets, and
a safe auto-import, semantic tokens, an applied diagnostic quick fix, project
commands, Problems, cancellation, Testing API outcomes, read-only evaluation
and controlled calls in every supported DAP context, detached-state recovery,
mutable scalar and aggregate Variables-view edits with invalidation and
continued-execution checks, conditional and exact-hit stops, and non-stopping
logpoints,
restarts it once, and shuts it down with the extension:

```text
npm test --prefix editors/vscode
```

For daily use, record reproducible problems with the local
[bug-report template](BUG_REPORT.md). The extension has no telemetry and sends
nothing automatically.

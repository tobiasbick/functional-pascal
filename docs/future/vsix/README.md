# Local VSIX and language server

**Status:** planned  
**Change class:** tooling and editor integration; no FPAS language change

## Goal

Build Functional Pascal editor support inside this repository and produce one
local host-native VSIX that can be installed in VS Code-compatible desktop
editors such as:

- Visual Studio Code
- Cursor
- VSCodium
- compatible desktop editors that implement the VS Code extension API

The extension provides:

- `.fpas` language detection
- TextMate syntax highlighting
- bracket, comment, indentation, and folding configuration
- diagnostics from the FPAS parser and semantic analyzer
- whole-document formatting through the canonical `fpas-fmt` implementation
- hover information
- go to definition
- document symbols
- basic completion from the current document and loaded project
- server lifecycle commands and an output channel for actionable failures

The finished VSIX is built locally. Publishing to Open VSX, the Visual Studio
Marketplace, or any other registry is explicitly out of scope.

## Product boundary

The Hello World bootstrap VSIX contains no native code and is platform
independent. The completed extension bundles a native `fpas-lsp` executable,
so its final VSIX is host-specific.

Whoever wants to use the extension builds it on the target system. A Windows
build contains the Windows executable, a Linux build contains the Linux
executable, and a macOS build contains the macOS executable. The build does
not cross-compile and does not bundle binaries for other systems.

This hobby project does not produce or maintain a release matrix. The shared
source and build script remain portable, but only the VSIX produced on the
current host is an output of a build.

Double-click installation may work when the operating system associates
`.vsix` files with an editor. The supported installation path is the editor
command **Extensions: Install from VSIX** or its command-line equivalent.

## Planned repository layout

```text
crates/
  fpas-language-service/
    Cargo.toml
    src/
      lib.rs                 — public editor-oriented analysis API
      document.rs            — immutable source snapshot and line index
      workspace.rs           — open documents and project context
      diagnostics.rs         — parser/sema diagnostic collection
      symbols/               — symbol index, definitions, hover, completion
      formatting.rs          — canonical whole-document formatting
  fpas-lsp/
    Cargo.toml
    src/
      main.rs                — stdio process entry point
      server.rs              — LSP connection and request loop
      capabilities.rs        — advertised server capabilities
      documents.rs           — LSP document synchronization
      convert/               — URI, UTF-16 position, range, and diagnostic conversion
      requests/              — hover, definition, symbols, completion, formatting
      notifications/         — open/change/save/close handling
      tests/                 — protocol-level fixtures and regression tests

editors/
  vscode/
    package.json             — extension manifest and local scripts
    package-lock.json        — reproducible Node dependency graph
    tsconfig.json
    language-configuration.json
    syntaxes/
      fpas.tmLanguage.json   — TextMate grammar
    src/
      extension.ts           — activation and lifecycle only
      languageClient.ts      — bundled server launch and client options
      serverPath.ts          — validated packaged executable lookup
      commands.ts            — restart and output commands
    test/
      grammar/               — highlighting fixtures
      extension/             — activation and manifest tests
    server/
      <host-target>/
        fpas-lsp[.exe]       — host-native build input; not committed
    scripts/
      stage-server.mjs       — copy and validate the release server
      package.mjs            — create the deterministic local VSIX
    dist/
      functional-pascal-<version>-<host-target>.vsix
```

The existing parser, semantic analyzer, diagnostics model, formatter, and
project loader remain in their owning crates. Protocol types must not leak into
those crates, and editor-specific code must not move into `fpas-cli`.

See [Architecture](architecture.md) for ownership rules and
[Implementation plan](implementation.md) for the delivery sequence. The
[SDK and documentation references](references.md) record the primary sources
used by the plan.

## Immediate first milestone

The first implementation phase does not wait for syntax highlighting, the
Rust language service, or the language server. It creates the smallest real
TypeScript extension, packages it locally, and produces:

```text
editors/vscode/dist/functional-pascal-0.0.1-bootstrap.vsix
```

When installed, the extension exposes **Functional Pascal: Show Output** and
writes `Functional Pascal extension activated (Hello World).` to its dedicated
output channel. The same bootstrap VSIX is installed and checked in VS Code,
Cursor, and VSCodium before architectural work begins.

## Non-goals

- changing FPAS syntax, semantics, or language documentation
- implementing a debugger
- executing untrusted FPAS programs from the extension
- adding a marketplace publisher account or release automation
- adding GitHub Actions or another CI service
- cross-compiling or distributing a platform matrix
- bundling several platform binaries into one VSIX
- maintaining a second formatter or parser in TypeScript
- depending on a globally installed `fpas` or `fpas-lsp`

## Definition of success

The plan is complete when a clean local checkout can run one documented npm
command on the target system and produce:

```text
editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix
```

That file must install without modification in a local VS Code-compatible
desktop editor running on the same host target; open an `.fpas` file with
highlighting; start the bundled language server; publish diagnostics; format
the document; and provide the planned navigation features.

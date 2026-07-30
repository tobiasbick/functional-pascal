# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding. Its bundled native `fpas-lsp` server
provides parser and semantic diagnostics, canonical whole-document formatting,
document symbols, hover, same- and cross-unit go to definition, and basic
visibility-aware completion. The repository builds and tests the extension
without a Marketplace.

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

The command runs the extension tests, builds `fpas-lsp` in Cargo release mode,
stages only the current host binary, creates the target-labelled archive, and
tests the server extracted from that archive. It produces:

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

Open the Outline view to inspect FPAS declarations. Hover a declaration or
reference, use **Go to Definition**, and invoke completion in a routine body or
after a unit/record `.`. Project-aware results use the same `.fpasprj`,
`.fpasworkspace`, visibility, and library-export boundaries as the compiler.

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated (Hello World).
```

The test command builds `target/debug/fpas-lsp[.exe]`, starts it from a real
VS Code Extension Host, verifies diagnostics, formatting, document symbols,
hover, cross-unit definition, and completion, restarts it once, and shuts it
down with the extension:

```text
npm test --prefix editors/vscode
```

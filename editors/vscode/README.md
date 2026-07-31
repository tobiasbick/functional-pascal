# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding. Its bundled native `fpas-lsp` server
provides parser and semantic diagnostics, canonical whole-document formatting,
document symbols, hover, same- and cross-unit go to definition, find all
references, validated project-wide rename, and basic visibility-aware
completion. The repository builds and tests the extension without a
Marketplace.

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
stages the current host binary plus the authoritative source-standard-library
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

Open the Outline view to inspect FPAS declarations. Hover a declaration or
reference, use **Go to Definition**, and invoke completion in a routine body or
after a unit/record `.`. Project-aware results use the same `.fpasprj`,
`.fpasworkspace`, visibility, and library-export boundaries as the compiler.
Use **Find All References** (`Shift+F12`) to list resolved declarations and
usages, including uses in loaded programs that consume a directly owned
library, and **Rename Symbol** (`F2`) to validate and edit a normal declaration
across those loaded projects. Program/unit renames and declarations outside
the opened folder are intentionally rejected.

The folder opened in the editor may be the complete Functional Pascal Rust
repository or another parent folder without an FPAS manifest. When a `.fpas`
file is opened, the server searches upward from that file and lazily loads its
nearest directly owning project or workspace. Multiple nested FPAS projects
can be used in one editor session; files without a matching manifest remain
available as loose files.

The installed VSIX supplies its own source standard library to the server, so
`Std.Tui` and the other source-defined `Std.*` units do not depend on a global
compiler installation or a `lib/` directory in the opened project.

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated (Hello World).
```

The test command builds `target/debug/fpas-lsp[.exe]`, starts it from a real
VS Code Extension Host, verifies diagnostics, formatting, document symbols,
hover, cross-unit definition, references, rename, and completion, restarts it
once, and shuts it down with the extension:

```text
npm test --prefix editors/vscode
```

For daily use, record reproducible problems with the local
[bug-report template](BUG_REPORT.md). The extension has no telemetry and sends
nothing automatically.

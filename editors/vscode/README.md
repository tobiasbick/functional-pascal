# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding. In repository development mode it
also starts the native `fpas-lsp` server over stdio for lifecycle and
full-document synchronization. The repository builds and tests it without a
Marketplace.

## Build

Node.js 22 or newer is required.

```text
npm ci
npm test
npm run package
```

The package command creates:

```text
dist/functional-pascal-0.0.1-bootstrap.vsix
```

Install it through **Extensions: Install from VSIX** in a VS Code-compatible
desktop editor.

## Verify

Open a `.fpas` file and confirm the status bar identifies the language as
**Functional Pascal**. Syntax highlighting works before the extension's
TypeScript entry point is activated.

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated (Hello World).
```

The test command builds `target/debug/fpas-lsp[.exe]`, starts it from a real
VS Code Extension Host, restarts it once, and shuts it down with the extension.
The bootstrap VSIX still does not bundle the native executable; host-native
staging and final VSIX naming belong to the packaging phase. Diagnostics,
formatting, and navigation are not advertised yet.

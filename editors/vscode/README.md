# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding without requiring a language server.
The repository builds, tests, and packages it as a VS Code-compatible VSIX
without a Marketplace.

## Build

Node.js 22 or newer is required.

```text
npm ci
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

This phase does not yet provide formatting or a language server.

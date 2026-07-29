# Functional Pascal

This is the local bootstrap extension for Functional Pascal. It proves that the
repository can build, test, package, and install a VS Code-compatible VSIX
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

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated (Hello World).
```

This bootstrap does not yet provide syntax highlighting, formatting, or a
language server.

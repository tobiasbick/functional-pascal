# VSIX SDK and documentation references

**Last verified:** 2026-07-29

These are implementation inputs, not substitutes for repository tests. Prefer
the Microsoft documentation and official project repositories below over
third-party tutorials. Recheck versions and compatibility before adding each
dependency because the plan intentionally does not pin future versions.

## Bootstrap extension and VSIX packaging

- [Your First Extension — Microsoft](https://code.visualstudio.com/api/get-started/your-first-extension)  
  Official TypeScript Hello World flow, Extension Development Host, command
  registration, and `engines.vscode` compatibility.
- [Extension Anatomy — Microsoft](https://code.visualstudio.com/api/get-started/extension-anatomy)  
  Manifest, activation, contribution points, `activate`/`deactivate`, and the
  relationship between `engines.vscode` and `@types/vscode`.
- [Hello World sample — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples/tree/main/helloworld-sample)  
  Maintained reference implementation for the minimal extension used as the
  Phase 0 baseline.
- [Publishing Extensions / local VSIX packaging — Microsoft](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)  
  Documents `vsce package` producing an installable VSIX. This project uses
  packaging only; the publishing and Marketplace sections are out of scope.
- [`@vscode/vsce` — Microsoft on GitHub](https://github.com/microsoft/vscode-vsce)  
  Source, requirements, and CLI documentation for the package builder.
- [Installing from VSIX — Microsoft](https://code.visualstudio.com/docs/configure/extensions/extension-marketplace#_install-from-a-vsix)  
  Supported editor UI and `code --install-extension <path.vsix>` installation
  paths for local artifacts.

## Host-native packaging decision

- [Platform-specific extensions — Microsoft](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platformspecific-extensions)  
  Documents target names and `vsce package --target <target>` for extensions
  containing native components.
- [Remote extensions and native code — Microsoft](https://code.visualstudio.com/api/advanced-topics/remote-extensions#using-native-nodejs-modules)  
  Explains why native executables must match the host where the extension runs.

This hobby project does not implement Microsoft's multi-platform publication
or CI model. It uses the platform target only to label a locally built VSIX:
build `fpas-lsp` natively on the current desktop host, package that one binary,
and let users on other systems run the same build themselves.

## Extension API and tests

- [VS Code Extension API — Microsoft](https://code.visualstudio.com/api/)  
  Entry point for stable extension API documentation and official samples.
- [Testing Extensions — Microsoft](https://code.visualstudio.com/api/working-with-extensions/testing-extension)  
  Current `@vscode/test-cli` and `@vscode/test-electron` integration-test flow.
- [VS Code extension samples — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples)  
  Maintained examples for contribution points, language features, tests, and
  extension structure.
- [VS Code source — Microsoft on GitHub](https://github.com/microsoft/vscode)  
  Authoritative source for the extension host and stable API implementation.

## Language registration and highlighting

- [Language Extensions Overview — Microsoft](https://code.visualstudio.com/api/language-extensions/overview)  
  Separation between declarative language support and programmatic features.
- [Syntax Highlight Guide — Microsoft](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide)  
  TextMate JSON grammar format, scopes, and the `grammars` contribution point.
- [Language configuration sample — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples/tree/main/language-configuration-sample)  
  Reference for comments, brackets, auto-closing, indentation, and folding.
- [`vscode-textmate` — Microsoft on GitHub](https://github.com/microsoft/vscode-textmate)  
  VS Code's TextMate tokenization library; useful for grammar fixture tests.

## Language client and LSP

- [Language Server Extension Guide — Microsoft](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)  
  Client/server architecture, process separation, logging, and LSP integration.
- [Programmatic Language Features — Microsoft](https://code.visualstudio.com/api/language-extensions/programmatic-language-features)  
  Mapping between editor features and LSP methods.
- [Official LSP 3.17 specification — Microsoft](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)  
  Protocol lifecycle, messages, capabilities, positions, diagnostics, and
  requests. Proposed protocol features are not required by this plan.
- [`vscode-languageserver-node` — Microsoft on GitHub](https://github.com/microsoft/vscode-languageserver-node)  
  Source for `vscode-languageclient`, the Node client used by the extension.
- [LSP sample — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples/tree/main/lsp-sample)  
  End-to-end language client/server extension example.

## Rust server implementation candidates

Choose and pin one Rust protocol implementation during Phase 1 after checking
maintenance, LSP 3.17 coverage, dependency impact, cancellation support, stdio
correctness, and testability. Do not write a custom JSON-RPC/LSP stack.

- [`tower-lsp-server` — community project on GitHub](https://github.com/tower-lsp-community/tower-lsp-server)  
  Maintained Tower-based async LSP server with stdio support and bundled
  `ls-types`.
- [`lsp-server` — rust-analyzer project on GitHub](https://github.com/rust-lang/rust-analyzer/tree/master/lib/lsp-server)  
  Transport-oriented synchronous LSP infrastructure used by rust-analyzer.
- [`lsp_server` API documentation — rust-analyzer](https://rust-lang.github.io/rust-analyzer/lsp_server/index.html)  
  Public API and message-loop documentation for the `lsp-server` candidate.

The selection belongs in the Phase 1 verification notes together with the
rejected candidate and a short reason. This avoids silently depending on stale
SDK assumptions.

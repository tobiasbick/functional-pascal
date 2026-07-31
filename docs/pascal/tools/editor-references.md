# Editor implementation references

These primary sources document the stable APIs used by the local Functional
Pascal extension. Recheck compatibility before changing pinned dependencies.

## Extension and packaging

- [VS Code Extension API — Microsoft](https://code.visualstudio.com/api/)
- [Your First Extension — Microsoft](https://code.visualstudio.com/api/get-started/your-first-extension)
- [Extension Anatomy — Microsoft](https://code.visualstudio.com/api/get-started/extension-anatomy)
- [VS Code extension samples — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples)
- [Local VSIX packaging — Microsoft](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- [`@vscode/vsce` — Microsoft on GitHub](https://github.com/microsoft/vscode-vsce)
- [Install from VSIX — Microsoft](https://code.visualstudio.com/docs/configure/extensions/extension-marketplace#_install-from-a-vsix)
- [Platform-specific extensions — Microsoft](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platformspecific-extensions)
- [Native code in remote extensions — Microsoft](https://code.visualstudio.com/api/advanced-topics/remote-extensions#using-native-nodejs-modules)

## Language support and tests

- [Language Extensions Overview — Microsoft](https://code.visualstudio.com/api/language-extensions/overview)
- [Syntax Highlight Guide — Microsoft](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide)
- [Language Configuration Guide — Microsoft](https://code.visualstudio.com/api/language-extensions/language-configuration-guide)
- [`vscode-textmate` — Microsoft on GitHub](https://github.com/microsoft/vscode-textmate)
- [`vscode-oniguruma` — Microsoft on GitHub](https://github.com/microsoft/vscode-oniguruma)
- [Testing Extensions — Microsoft](https://code.visualstudio.com/api/working-with-extensions/testing-extension)
- [VS Code source — Microsoft on GitHub](https://github.com/microsoft/vscode)

## Language client and server

- [Language Server Extension Guide — Microsoft](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [Programmatic Language Features — Microsoft](https://code.visualstudio.com/api/language-extensions/programmatic-language-features)
- [Language Server Protocol 3.17 — Microsoft](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [`vscode-languageserver-node` — Microsoft on GitHub](https://github.com/microsoft/vscode-languageserver-node)
- [LSP extension sample — Microsoft on GitHub](https://github.com/microsoft/vscode-extension-samples/tree/main/lsp-sample)
- [`tower-lsp-server` — community project on GitHub](https://github.com/tower-lsp-community/tower-lsp-server)
- [`tower-lsp-server` feature coverage](https://github.com/tower-lsp-community/tower-lsp-server/blob/main/FEATURES.md)

The extension pins `vscode-languageclient` in its package lock, and the Rust
server pins `tower-lsp-server` through the Cargo lock. The project packages
locally and does not publish to a registry.

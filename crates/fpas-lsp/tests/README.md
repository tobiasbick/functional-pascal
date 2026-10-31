# LSP tests

Integration tests share their transcript helpers within each executable. Each
transcript still starts its own server process and owns its temporary files.

| Target | Modules |
| --- | --- |
| `analysis` | `diagnostics`, `intellisense`, `semantic_tools`, `snippets` |
| `editing` | `formatting`, `refactoring`, `references_rename`, `public_signatures` |
| `workspace` | `navigation`, `workspace_navigation`, `document_lifetime` |
| `process` | Raw transport and process fixtures; remains standalone |
| `protocol` | Protocol and shutdown lifecycle fixtures; remains standalone |

Example: `cargo test -p fpas-lsp --test analysis diagnostics`.
Add new test modules to a suite; automatic test discovery is disabled.

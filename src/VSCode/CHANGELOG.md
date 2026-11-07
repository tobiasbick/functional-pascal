# Functional Pascal CLR 0.1.0

The extension runs in VS Code 1.145.0 and newer on Windows and Remote-WSL.
It uses the FPAS CLI, project SDK and .NET 10 SDK installed on the workspace
host.

- Syntax highlighting, snippets, navigation, IntelliSense, live diagnostics,
  formatting, checked corrections and rename for `.fpas` sources.
- Project commands and tasks, with build diagnostics in Problems and
  interactive program input in the integrated terminal.
- Native VSTest and Microsoft.Testing.Platform discovery and execution in the
  Testing view, with TRX artifacts and cancellation.
- CLR debugging for FPAS and mixed solutions with the `fpas-clr` launch type.
  Supports breakpoints, stepping, scopes, passive evaluation and exception
  filters.
- Checked previews for Add Parameter, Extract Variable and Organize Imports.
  Apply revalidates the edit, preserves open buffers and supports undo.

Restricted Mode provides syntax highlighting and snippets. Toolchain
workflows require workspace trust. See the [installation and recovery
instructions](README.md#install-and-select-a-project).

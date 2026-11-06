# Language server

`fpas lsp` serves LSP 3.17 over standard input/output. Start it as a child process
and exchange UTF-8, byte-counted `Content-Length` frames. Standard output contains
protocol messages only; standard error contains connection failures.

The local distribution and project SDK include the server and its dependencies.
Keep the published files together; `fpas env` reports the server's managed
identity without starting it. The SDK copy can also run as
`dotnet tools/compiler/Fpas.Cli.dll lsp` from an extracted package. See the
[deployment workflow](../../build/development.md#deploy-the-local-release-compiler).

## Initialization and documents

Send `initialize` with `capabilities` and these initialization options:

```json
{
  "context": "file:///project/Application.fpasproj",
  "configuration": "Debug"
}
```

`context` selects an existing `.fpasproj` or `.slnx`, using an absolute filesystem
path or local file URI. `configuration` defaults to `Debug`. Document URIs must
be local file URIs. The server uses zero-based UTF-16 positions; clients offering
only another encoding are rejected. Send `initialized` after initialization.

Open documents with language ID `fpas`. Full and incremental `didChange` batches
apply in order, with strictly increasing integer versions. `didSave` retains the
open text; `didClose` drops it and clears its diagnostics. Invalid changes clear
diagnostics and pause analysis until a newer full-text change or close/reopen
resynchronizes the document. Sources are never saved by the server.

The server publishes diagnostics for open documents with their exact version,
range, severity, code and related locations. Unowned open files receive syntax
diagnostics. Project/setup failures use `window/logMessage`, without invented
source locations.

Missing terminators and broken bodies can retain diagnostics from independent
checked bodies. A body with syntax errors is unavailable for semantic facts;
invalid declarations can make its source unit and source consumers unavailable.
Repairing the open text replaces those results at the new document version.

## Navigation and edits

The server advertises definition, references, workspace symbols and document
formatting. Definition returns an exact checked source location or null; metadata
and implicit declarations have no invented file URI. References respect
includeDeclaration. Workspace symbols use the common case-sensitive Find search,
including an empty query. Results cover the selected FPAS context, with at most
1,000 locations/symbols. Incomplete or oversized searches fail explicitly rather
than returning an apparently exhaustive shortened array. Standard LSP requests
cannot select a linked file's owning project; choose a narrower initialization
context when ownership is ambiguous.

Rename requires workspace.workspaceEdit.documentChanges support. Clients with
textDocument.rename.prepareSupport also receive prepare-rename. Both use the
common checked rename service; incomplete analysis, capture, overload changes
and external contracts are refused. Supply an unescaped ASCII newName; keywords
receive their required @ spelling. WorkspaceEdit.documentChanges includes each
open document's exact version and an explicit null version for closed documents.
The client applies the edits; the server never writes sources or calls applyEdit.

Formatting uses the current open text, including unowned files, without project
evaluation. It returns one full-document edit or an empty array. Invalid syntax
fails without replacement text. The fixed FPAS style governs indentation and
newlines; client tabSize/insertSpaces do not select another style.

Requests capture the current revision on receipt, reverify semantic inputs before
responding and reject superseded results. Cancellation returns -32800, changed
content -32801, invalid parameters -32602 and unavailable/limited work -32803.

## Project inputs and limits

Use a matching installed FPAS SDK and explicitly restore projects before opening
them. Analysis performs no restore, dependency build or source generation. FPAS
project references are analyzed from current sources. C#/F# references require
existing assemblies and matching portable symbols; explicitly rebuild changed
foreign projects. See [project analysis](../../architecture/workspaces.md).

Clients should send `workspace/didChangeWatchedFiles` for changed sources,
projects, imports, restore assets and DLL/PDB references. Clients supporting
dynamic registration receive the server's watcher registration. Publication
rechecks inputs and discards obsolete work.

Messages are limited to 8 MiB incoming and 4 MiB outgoing, with 8 KiB headers.
Document text is limited to 4 MiB UTF-8, total open text to 64 MiB, outstanding
requests to 64 and accounted retained analysis/cache data to 128 MiB. Analysis
has a 30-second work deadline. Superseding edits cancel obsolete analysis;
shutdown and EOF cancel and join owned evaluation processes.

Send `shutdown`, await its null result, then send `exit`. Clean EOF or this
sequence exits with 0; an unexpected `exit` or broken connection exits with 1.
Invalid command arguments exit with 2 and process cancellation with 130.

## See also

- [Compiler CLI](../program-structure/cli.md)
- [Diagnostics](diagnostics.md)
- [Source formatting](fmt-style.md)

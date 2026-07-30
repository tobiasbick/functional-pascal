# VSIX architecture

## Design principles

Editor support is split into three layers:

```text
VS Code-compatible extension
          |
          | LSP over stdio
          v
      fpas-lsp
          |
          | editor-oriented Rust API
          v
 fpas-language-service
          |
          +-- fpas-project
          +-- fpas-parser
          +-- fpas-sema
          +-- fpas-diagnostics
          `-- fpas-fmt
```

Each layer has one concern:

- The extension integrates with the VS Code API and owns process lifecycle.
- `fpas-lsp` translates between LSP messages and FPAS service requests.
- `fpas-language-service` owns source snapshots, project state, analysis, and
  symbol queries without knowing about LSP.
- Existing FPAS crates remain authoritative for language behavior.

The language server communicates only through standard LSP over stdin/stdout.
Logs go to stderr so they cannot corrupt the protocol stream.
Phase 1 selected `tower-lsp-server` as the Rust transport implementation and
fixed full-document synchronization as the initial document contract. See
[Phase 1 editor contracts](contracts.md) for the capability and host policies.

The delivery order deliberately starts one step earlier than this final
architecture. Phase 0 packages a minimal extension with no Rust server and
writes a Hello World activation line to a Functional Pascal output channel.
This proves the manifest, TypeScript toolchain, VSIX packaging, installation,
activation, and compatible-editor behavior before LSP complexity is introduced.
Phase 2 adds `.fpas` registration, language configuration, and TextMate
highlighting declaratively. Opening a Functional Pascal file therefore does
not activate TypeScript code or depend on a server process.
Phase 3 implements the protocol-independent service layer. `DocumentStore`
owns immutable source snapshots and open-buffer overlays, `WorkspaceContext`
loads loose/project/workspace metadata, and `LanguageService` caches parser
and semantic results across all participating source revisions. LSP lifecycle
and UTF-16 conversion remain isolated to the Phase 4 transport.
Phase 4 implements that transport with `tower-lsp-server`: a native stdio
binary, strict file-URI conversion, full-text synchronization, framed protocol
tests, and a VS Code development client with explicit start/stop/restart
ownership. Only implemented synchronization capabilities are advertised.

## Why a language-service crate

Diagnostics and formatting can be called directly from existing crates, but
hover, definitions, symbols, completion, and multi-file invalidation share
state and indexing rules. Keeping this state in `fpas-lsp` would couple useful
compiler functionality to a transport protocol and encourage a large server
module.

`fpas-language-service` provides a small editor-facing facade while reusing
the current parser, semantic analyzer, formatter, and project loader. It does
not duplicate compiler rules and does not become a second compiler pipeline.

The existing large `fpas-sema/src/interface.rs` should only be split when a
specific language-service query requires changes there. Any split should be by
symbol responsibility and preserve language behavior; a broad prerequisite
refactor is not part of the first phase.

## Document model

Open editor buffers are authoritative, including unsaved text. Each snapshot
contains:

- normalized local path
- monotonically increasing document version
- UTF-8 source text
- line-start index
- parsed compilation unit and parser diagnostics

`DocumentAnalysis` pairs a snapshot with semantic results, merged diagnostics,
and extracted declarations when parsing permits analysis. Keeping semantic
state outside the immutable source snapshot allows one snapshot to participate
in either loose-file or project analysis.

The extension requests full-document synchronization initially. This keeps
change handling deterministic and avoids premature incremental-parser work.
The service caches results by the exact source revisions participating in an
analysis. A changed open buffer therefore invalidates its loose analysis and
any project analysis that includes it.

LSP positions use UTF-16 code units while Rust source spans use UTF-8 byte
offsets. All conversion lives under `fpas-lsp/src/convert/` and must handle:

- ASCII
- non-ASCII BMP characters
- surrogate-pair characters
- CRLF and LF
- a position at end of line or end of file
- invalid, stale, or out-of-range client positions without panicking

## Workspace and project discovery

The server treats the opened workspace folder as the discovery root and reuses
`fpas-project` for `.fpasprj`, `.fpasworkspace`, dependencies, exported units,
and standard-library resolution.

Loose `.fpas` files without a project still receive syntax diagnostics,
formatting, document symbols, and same-document navigation. Project-dependent
features degrade gracefully when no valid manifest can be found.

Unsaved `.fpas` buffers override the corresponding on-disk sources during
analysis. The server must not create `.fpascu` files merely to answer editor
queries.

Phase 3 loads manifests when `WorkspaceContext` is constructed, and Phase 4
constructs that context from the initialized workspace root. Dynamic manifest
reload is still later server work; source-text invalidation already happens
through `DocumentStore`.

## Capabilities

### Syntax highlighting

Highlighting uses a TextMate grammar in the extension and does not wait for the
server. The grammar covers declarations, control flow, visibility, literals,
comments, operators, built-in types, and qualified names. It must avoid
classifying arbitrary identifiers as standard-library symbols.

Semantic tokens are deferred until they provide a measured improvement over
TextMate highlighting.

### Diagnostics

Diagnostics are produced on open and change, with a short debounce for typing.
Parser and semantic diagnostics retain their existing stable codes, severity,
message, source range, and help text where LSP supports it.

The server clears stale diagnostics when a document closes or becomes valid.
Failures in project discovery are reported as actionable diagnostics or server
messages rather than panics.

### Formatting

The language service parses the current in-memory source and delegates output
to `fpas-fmt`. It returns one whole-document text edit only when formatting
succeeds. Parse errors produce no destructive edit.

The extension registers the formatter through LSP, so normal editor settings
such as format-on-save work without introducing a separate watcher.

### Symbols and navigation

The symbol index is derived from parser and semantic data. It begins with:

- programs, units, types, constants, variables, functions, and procedures
- record members and parameters
- declaration ranges and selection ranges
- same-document and project-unit definitions
- type/signature text for hover
- visible declarations for basic completion

Results must respect current FPAS visibility and unit rules. The index must not
invent language semantics to compensate for missing compiler information.

## Extension lifecycle

The extension activates for `.fpas` documents. It resolves the bundled server
relative to the installed extension directory, verifies the executable exists,
and starts it with stdio transport.

The packaged server directory is selected from the current operating system
and architecture. Each final VSIX contains only the executable built on and
for that host. An unsupported host-target mapping produces an actionable build
error rather than a partially working package.

No shell command string is constructed. The executable and arguments are
passed separately to the process API. Startup failures identify the expected
path and suggest reinstalling the VSIX.

The extension contributes:

- Functional Pascal language registration
- language configuration and TextMate grammar
- `Functional Pascal: Restart Language Server`
- `Functional Pascal: Show Language Server Output`
- an optional trace setting disabled by default

It does not modify user settings automatically.

During repository development and tests, the extension resolves
`target/debug/fpas-lsp[.exe]` relative to the repository. Production lookup is
limited to `server/<host-target>/fpas-lsp[.exe]`; the package script stages the
current host's release binary at that exact path. Neither path falls back to a
globally installed executable.

## Packaging

The package script extends the proven bootstrap path on the current host:

1. Build `fpas-lsp` in Cargo release mode.
2. Derive the VS Code target name from the host operating system and
   architecture.
3. Copy `fpas-lsp` or `fpas-lsp.exe` to
   `editors/vscode/server/<host-target>/`.
4. Compile TypeScript.
5. Run TypeScript, grammar, manifest, and package-content checks.
6. Create a target-labelled VSIX under `editors/vscode/dist/`.
7. Inspect the archive and fail if the server, grammar, license, or extension
   entry point is missing.
8. Extract the archive and complete an LSP initialize/shutdown transcript
   against its server.

Generated JavaScript, the staged server executable, Node dependencies, and
VSIX files are local build artifacts and must be ignored by Git.

The package contains the repository license and does not contain source maps,
test fixtures, host paths, usernames, machine names, Cargo target files, or
unrelated FPAS binaries.

The script does not download native binaries, cross-compile, build other
targets, publish, or invoke CI. A user on another platform runs the same build
from that platform.

## Compatibility contract

The extension uses only stable VS Code extension APIs supported by the chosen
minimum `engines.vscode` version. It must not inspect editor product names or
special-case Cursor/VSCodium.

Compatibility is checked in at least one locally available VS Code-compatible
desktop editor. Additional clone checks are useful when those editors are
already installed, but maintaining an editor/platform test matrix is out of
scope. A successful install alone is insufficient; activation, server startup,
diagnostics, formatting, and one navigation request must work.

# Shared diagnostics

The Rust toolchain uses `fpas-diagnostics::Diagnostic` for lexer, parser,
semantic, compiler and runtime errors. Its fields contain a stable code,
severity, message, optional help, optional source span, and optional
producer-supplied expected/found details. A missing position is `None`; it is
not line zero or an invented first line.

## Text output

Known source positions retain this form:

```text
parse.fpas:2:1: error[F1001]: Expected `;`, found `begin`
  help: Insert `;` here.
```

For example, `program Demo` without its terminating semicolon produces that
error before the following `begin`. Correct the heading to `program Demo;`.

When a position is unavailable, the renderer omits the line/column prefix.
An available path can still identify the file. Project source-read failures use
`F5001` with an operating-system error and a hint to check the file. Invalid UTF-8
source bytes use `F5002` with no scalar position and a hint to save as UTF-8.
Other project and build errors still use their existing reporting paths.

## Rust JSON rendering API

`render_json(path, source_text, diagnostic)` serializes one record without a
trailing newline. This is a Rust library API; the CLI currently emits text.
The caller supplies the name and text matching the diagnostic's source ID.
The renderer does not read files or guess another source when text is unavailable.

```json
{"kind":"diagnostic","code":"F1001","severity":"error","phase":"parse","source":"parse.fpas","location":{"source_id":0,"start":{"line":2,"column":1},"end":{"line":2,"column":6}},"message":"Expected `;`, found `begin`","expected":";","found":"begin","hint":"Insert `;` here."}
```

| Field | Meaning |
|---|---|
| kind | `diagnostic` |
| code | Stable `Fxxxx` identifier |
| severity | `error` or `warning` |
| phase | `lex`, `parse`, `sema`, `compile`, `runtime`, `project`, or `internal` |
| source | Caller-supplied source name, or null |
| location | Source ID and positions, or null |
| message | Primary explanation |
| expected / found | Structured details when supplied by the producer, otherwise null |
| hint | Optional actionable help, or null |

Lines and columns are one-based; columns count Unicode scalar values, including
tabs as one scalar. A supplementary character counts once, not as two UTF-16
units. CRLF is one line ending; LF and bare CR also advance the line. End positions
are exclusive. Real byte ranges are resolved against UTF-8 text; invalid bounds
or non-character boundaries produce null location rather than a panic.

Runtime source maps can provide only a start location. Such point locations have
`end: null`; their synthetic byte offsets are never interpreted as real ranges.
A real span without source text likewise retains its known start with a null end.
LSP adapters convert scalar point locations to the protocol's UTF-16 coordinates;
diagnostics without positions cannot be attached to a document range.

The serializer uses a fixed field order and escapes newlines, quotes and control
characters. A caller can append one newline per record without message text
injecting another record. Text and JSON preserve the same code, severity, message
and hint; expected/found details are not reconstructed by parsing message prose.

## Project error transport

`fpas_project::ProjectError` preserves source failures through project loading,
transitive library dependencies, standard-library loading, unit-graph creation
and `UnitNode::parse_source_snapshot`. `diagnostics()` returns all records from
the failing source in producer order; `source_path()` identifies that source,
including read failures without a position. Lexer diagnostics precede parser
diagnostics. Expected/found details are retained without parsing message text.

Source IDs are local to their producer. Project loading and graph construction
parse each source with ID zero before a graph exists; use the error's source
path to identify that file. Snapshot parsing uses the existing graph node's
source ID. A dependent source is not relabeled as its consuming project.

Manifest and graph validation failures retain text, with an empty diagnostic
slice and no source path. Successful-source warnings still use existing string
collections. `Display` renders source records only when requested by a caller;
current CLI, editor and distribution adapters explicitly convert to their text
interfaces. This API does not enable CLI JSON output.

## Build error transport

`fpas_build::BuildError::diagnostics()` exposes the original compiler or parser
records as `BuildDiagnostic` entries in producer order. Each entry contains the
shared `Diagnostic` and an optional source path. Unit compilation retains its
unit path when the diagnostic's source ID matches that unit. Unknown or foreign
source IDs are not assigned the current file.

Program-artifact parsing retains all lexer/parser diagnostics when parsing fails,
including expected/found details. Program-artifact compilation retains its
supplied main-source path. The AST-based `build_program` and `check_program` APIs
do not receive an authoritative path for the supplied AST; their root compiler
errors retain their source IDs and positions with no path.

`BuildError::link_error()` exposes the original `fpas_linker::LinkError`, which is
also available through `std::error::Error::source()`. Linker errors have not been
assigned shared diagnostic codes. Conversion from `ProjectError` preserves all
source records and their path, including positionless read/UTF-8 failures.
Other filesystem, project-validation and build failures still use their existing
text reporting. An empty `diagnostics()` slice
therefore does not mean success; inspect the returned `Result` first.

`Display` renders preserved records at the text-output boundary. It does not
recover codes, coordinates or expected/found fields from message text. For JSON,
pass each entry's diagnostic and optional path to `render_json`, supplying the
matching source text when available.

Root compiler diagnostics omit the path in `Display`, preserving the text
contract in which the caller supplies the main-file context. Their structured
entries still retain the artifact's known main path.

## Code ranges

| Phase | Range |
|---|---|
| Lex | F0001–F0013 |
| Parse | F1001–F1999 |
| Sema | F2001–F2999 |
| Compile | F3001–F3999 |
| Runtime | F4001–F4999; F4017 remains reserved |
| Project/build | F5001–F5999; F5001 is source-read failure, F5002 is invalid UTF-8 source |
| Internal | F9001–F9999 and otherwise unassigned values |

The allocated code inventory is maintained in
[`codes.rs`](../../../crates/fpas-diagnostics/src/codes.rs). Existing allocated
identifiers and their phase assignments are unchanged.

## Implementation

The shared model and renderers live in
[`fpas-diagnostics`](../../../crates/fpas-diagnostics/src/lib.rs).
Parser token expectations populate structured expected/found fields.
VM instruction source maps preserve source identity, including imported units.
Project source failures are retained by
[`ProjectError`](../../../crates/fpas-project/src/source/error.rs); build snapshot
parsing forwards them into `BuildError` without rendering.

See also [tools](README.md), [CLI](../program-structure/cli.md), and
[editor integration](editor-integration.md).

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
`F5001` with an operating-system error and a hint to check the file. Other project
and build errors still use their existing reporting paths.

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

## Code ranges

| Phase | Range |
|---|---|
| Lex | F0001–F0013 |
| Parse | F1001–F1999 |
| Sema | F2001–F2999 |
| Compile | F3001–F3999 |
| Runtime | F4001–F4999; F4017 remains reserved |
| Project/build | F5001–F5999; F5001 is source-read failure |
| Internal | F9001–F9999 and otherwise unassigned values |

The allocated code inventory is maintained in
[`codes.rs`](../../../crates/fpas-diagnostics/src/codes.rs). Existing allocated
identifiers and their phase assignments are unchanged.

## Implementation

The shared model and renderers live in
[`fpas-diagnostics`](../../../crates/fpas-diagnostics/src/lib.rs).
Parser token expectations populate structured expected/found fields.
VM instruction source maps preserve source identity, including imported units.
Project source reads create structured errors; their current project API renders
those errors into its existing string return type.

See also [tools](README.md), [CLI](../program-structure/cli.md), and
[editor integration](editor-integration.md).

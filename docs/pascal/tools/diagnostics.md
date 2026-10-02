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
An available path can still identify the file:

```text
error[F5005]: Invalid `project.kind` value `app` in `demo.fpasprj`.
  help: Use `program`, `library`, or `test`.
```

Correct the manifest to `kind = "program"`, `"library"`, or `"test"`. A `uses`
entry that names a missing unit is located at the import:

```text
src/a.fpas:2:6: error[F5015]: Unknown unit `Demo.Missing` in unit `Demo.A`.
  help: Known units in `Demo`: Demo.A, Demo.B.
```

The hint lists units in the missing unit's namespace, otherwise units with the
same root segment, otherwise the project's non-`Std` units; long lists show ten
names and the number of omitted units.

Multi-line messages, such as TOML parser excerpts, continue on lines prefixed
with `message:`.

## Rust JSON rendering API

`render_json(path, source_text, diagnostic)` serializes one record without a
trailing newline. `fpas check`, `build`, `run` and `test` write these records with
`--diagnostics json`, one per stderr line; see
[machine-readable diagnostics](../program-structure/cli.md#machine-readable-diagnostics).
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

Manifest, workspace, standard-library and graph validation failures also carry
one coded record with an optional hint; `diagnostics()` is never empty. These
records have no position and no source path because they concern manifests or
several files, which their messages name. Unknown or non-exported units in a
unit's `uses` clause are the exception: their record has the importing unit's
path and the span of the imported name. The workspace discovery functions
`load_workspace`, `discover_workspace_file`, `discover_run_project_in_workspace`
and `discover_test_projects_in_workspace` return the same `ProjectError`.

A successfully loaded project reports non-fatal findings in
`LoadedProject::warnings` as `fpas_diagnostics::FileDiagnostic` records: the
shared `Diagnostic` with warning severity and the file it concerns. F5035 marks a
source file listed more than once (the first occurrence is kept); F5036 marks a
`program` source that was skipped because it is not an allowed entry file.
Lexer/parser warnings of a successfully parsed source keep their original code
and span with that source's path. The CLI prints them in text form, for example:

```text
src/util.fpas: warning[F5035]: Duplicate source file was ignored; the first occurrence was retained.
  help: List each source file once in `[sources].include`.
```

`Display` renders the records when requested by a caller; current CLI and editor
adapters explicitly convert to their text interfaces; the CLI writes the same
records as JSON with `--diagnostics json`.

## Build error transport

`fpas_build::BuildError::diagnostics()` exposes the original compiler or parser
records as `FileDiagnostic` entries in producer order. Each entry contains the
shared `Diagnostic` and an optional source path. Unit compilation retains its
unit path when the diagnostic's source ID matches that unit. Unknown or foreign
source IDs are not assigned the current file.

Program-artifact parsing retains all lexer/parser diagnostics when parsing fails,
including expected/found details. Program-artifact compilation retains its
supplied main-source path. The AST-based `build_program` and `check_program` APIs
do not receive an authoritative path for the supplied AST; their root compiler
errors retain their source IDs and positions with no path.

Every `BuildError` carries at least one coded record. Artifact filesystem and
encoding failures, source files that cannot be read or changed during the build,
and build invariant failures have positionless records; failures about one source
file carry its path. `BuildError::link_error()` exposes the original
`fpas_linker::LinkError`, which is also available through
`std::error::Error::source()`; its record uses the code from `LinkError::code()`.
Conversion from `ProjectError` preserves all records and their path, including
positionless read/UTF-8 failures. `stage_standard_library` reports its failures
as `BuildError` too.

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
| Project, build, CLI and test runner | F5001–F5999 |
| Internal | F9001–F9999 and otherwise unassigned values |

Project, build, linker, CLI and test-runner codes:

| Code | Meaning |
|---|---|
| F5001 | A source file cannot be read |
| F5002 | Source bytes are not valid UTF-8 |
| F5003 | A `.fpasprj` or `.fpasworkspace` manifest cannot be read |
| F5004 | A manifest is not valid TOML or does not match the manifest schema |
| F5005 | A manifest field is missing, empty, unknown or not allowed for the project kind |
| F5006 | A manifest path is missing, not a file, has the wrong extension, or cannot be resolved |
| F5007 | A source glob is invalid, cannot be evaluated, or matches no files |
| F5008 | A manifest lists the same entry twice |
| F5009 | A project dependency is not a library |
| F5010 | Library projects depend on each other in a cycle |
| F5011 | One source file belongs to more than one project |
| F5012 | A source declares `program` where `unit` is required, or the reverse |
| F5013 | A unit name violates the `Std.*` namespace rules |
| F5014 | Two sources declare the same unit name |
| F5015 | A `uses` clause or `[exports].units` names an unknown unit |
| F5016 | A unit is imported across a library boundary without being exported |
| F5017 | Units depend on each other in a cycle |
| F5018 | Too many source files for 32-bit source IDs |
| F5019 | A source file changed after its project graph or build snapshot was taken |
| F5020 | A directory needed for workspace discovery cannot be read |
| F5021 | Project or workspace discovery found no candidate or more than one |
| F5022 | `[dependencies].workspace` names no workspace member |
| F5023 | The standard-library directory or manifest is invalid |
| F5024 | A compiled-unit or program artifact cannot be read, locked, written or replaced |
| F5025 | A compiled interface, object or program image cannot be encoded or validated |
| F5026 | A compiled object is malformed or inconsistent with linked objects |
| F5027 | The root program object has no entry function |
| F5028 | Two linked objects define the same symbol |
| F5029 | Linked objects disagree on a record or enum layout |
| F5030 | No linked object defines a required public symbol |
| F5031 | An object imports a private definition |
| F5032 | An import has the wrong kind or an incompatible ABI |
| F5033 | A linked table or address exceeds its fixed-width limit |
| F5034 | The linked executable failed bytecode verification |
| F5035 | Warning: a source file is listed more than once; the first occurrence is kept |
| F5036 | Warning: a `program` source that is not an allowed entry file was skipped |
| F5037 | Command-line arguments are invalid, duplicated or incomplete |
| F5038 | The input kind cannot be used by the command (for example running a library) |
| F5039 | A command could not write its promised output or output files |
| F5040 | A test's standard output differs from its `.expect.stdout` file; expected/found carry both outputs |
| F5041 | A test exceeded its timeout |
| F5042 | The test runner could not prepare, isolate or finish a test or hook |
| F9003 | Project graph or build orchestration reached an inconsistent state |

The allocated code inventory is maintained in
[`codes.rs`](../../../crates/fpas-diagnostics/src/codes.rs). Existing allocated
identifiers and their phase assignments are unchanged.

## Implementation

The shared model and renderers live in
[`fpas-diagnostics`](../../../crates/fpas-diagnostics/src/lib.rs).
Parser token expectations populate structured expected/found fields.
VM instruction source maps preserve source identity, including imported units.
Project failures are retained by
[`ProjectError`](../../../crates/fpas-project/src/source/error.rs); shared manifest
failures live in [`manifest.rs`](../../../crates/fpas-project/src/manifest.rs).
Build failures are retained by
[`BuildError`](../../../crates/fpas-build/src/engine/error.rs), which forwards
project records without rendering. Linker categories are assigned in
[`LinkError::code`](../../../crates/fpas-linker/src/error.rs).

See also [tools](README.md), [CLI](../program-structure/cli.md), and
[editor integration](editor-integration.md).

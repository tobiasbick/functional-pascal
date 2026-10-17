# Shared diagnostics

The Rust toolchain uses `fpas-diagnostics::Diagnostic` for lexer, parser,
semantic, compiler and runtime errors. Its fields contain a stable code,
severity, message, optional help, optional source span, and optional
producer-supplied expected/found details. A missing position is `None`; it is
not line zero or an invented first line.

## Embedded VM failure observation

The CLI exposes this capture through `fpas run --failure-dump PATH`. See
[runtime failure reports](failure-dump.md) for activation, publication and the
versioned JSON schema.

Rust hosts can opt into `fpas_vm::Vm::set_failure_observer` before `Vm::run`.
The observer receives one immutable `FailureObservation` for an escaping runtime
error. It contains the selected diagnostic, an explicit original-failure identity,
the responsible task's available pre-cleanup frames and source bindings, and
structured secondary failures in the existing runtime attachment order. Source
positions use the verified image's source IDs. No filesystem lookup is performed.

The top-level observation also contains a `FailureTaskCatalog`: main, cause
tasks, live tasks and terminal tasks whose outcomes remain retained at the cut.
Released outcomes are not reconstructed. Entries use runtime task IDs and
`runnable`, `running`, `waiting`, `sleeping`, `completed`, `failed`, or `cancelled`
states. Task output is sorted by ID; selection preserves the primary and main
identities before other causes and peers. At most 1,024 entries are retained;
known totals and omitted counts disclose task or storage limits.

Fault frames keep `observation: "fault"`. Live peer frames and the distinct
main-task propagation stack use `"failure_handling"`; these are later
cooperative observations. Running workers answer at existing batch boundaries.
Saved tasks are inspected under their current owner's synchronization, without
executing queued code or consuming results/channels. Running peers continue
after answering; they are not held for the observer. A peer blocked in native
work retains its identity and known state, with frames unavailable after the
single 500-ms collection deadline. Completed/cancelled peers have no invented
stack. A synchronous callback peer reports partial data if its outer worker's
storage is unavailable. An unexecuted source initializer is explicitly uninitialized.

Globals are captured once from the root's actual local or shared storage,
after primary frame values and before root disposal, including errors first
selected during server finalization or worker joins. A completed main task keeps
its own completed state if a peer fails afterward. `globals_status`, task inspection
states and omission reasons distinguish
available, partial and unavailable data. Synchronization is released before the
observer runs, and ordinary cancellation and cleanup continue afterward.

The observer runs after ownership barriers have selected the escaping error and
before remaining server shutdown and worker joins when the error is already
known. It must return promptly. Cleanup and joins still run; observer panics do
not replace the program diagnostic. Handled task-group failures, successful
supervisor retries, ordinary `Result` data and successful runs do not notify.

Capture is disabled by default. Enabled captures share a VM-wide limit of 16,384
value objects and 8 MiB of charged retained data. Each cause retains at most 256
frames, each aggregate 256 children, each string 4,096 Unicode scalars, and
each value tree 16 child edges. Missing metadata, busy storage and exhausted
bounds have explicit markers. Diagnostic presentation text is bounded as well,
with a separate reason when text cannot be retained. The ordinary error returned
by `Vm::run` is unchanged. Captured data follows the diagnostic's lifetime;
handled or released outcomes do not create a failure history. Diagnostic
attachments preserve identity through clones and do not change text/JSON
diagnostic rendering or diagnostic equality.

Control data and the selected diagnostic have reserved capacity within the same
8 MiB limit, so large values cannot erase the primary error message. Secondary
relationships and terminal model collections are charged before allocation;
at most 1,024 relationships are retained, with explicit omitted counts when a
relationship, model-storage, or nesting bound prevents retaining more.
The control reservation also covers the task catalog. Disabled runs allocate no
D1 catalog; their batch loops use the specialization without capture polling.

## Text output

Known source positions retain this form:

```text
parse.fpas:2:1: error[FP2001]: Expected `;`, found `begin`
  2 | begin
    | ^^^^^
  help: Insert `;` here.
```

For example, `program Demo` without its terminating semicolon produces that
error before the following `begin`. Correct the heading to `program Demo;`.

CLI text diagnostics show one numbered source line and a caret marker before
the help when matching source text and a valid position are available. Byte
ranges mark their visible extent; runtime point locations use one caret.
Multi-line ranges show the first affected line and `...` after its marker.
Empty ranges, empty lines and end-of-file positions still receive one caret.

Source coordinates remain one-based Unicode scalar positions. Display alignment
uses terminal cells: tabs expand to four-cell stops, wide characters take their
display width, and combining characters add no width. Control characters use
the same visible escapes as other diagnostic content. Long source lines are
cropped to at most 100 display cells, including `...` cropping markers and
excluding the gutter. The diagnostic start stays visible; ranges use the same
cropped window. UTF-8 scalars and displayed escapes are never split.

The CLI prefers retained compilation text and caches known runtime sources
before execution. Source IDs select the corresponding file, including imported
units; a project or image fallback name is not a source file. Direct `.fpascp`
execution displays text only when an exact stored source path can be read and
its content matches the image's source hash. It does not search for relocated
sources. Missing, unreadable, mismatched or invalid source data silently keeps
the original header and help. The source-free embedded rendering API also keeps
that compact form and performs no filesystem access.

Text diagnostics automatically color severity/code and carets red for errors
and yellow for warnings when the final stderr destination is a terminal. A
nonempty `NO_COLOR` disables color, including `NO_COLOR=0`; an empty value does
not. Redirected output and ordinary capture buffers stay plain. Parallel and
isolated test workers inherit the outer output policy and indent every excerpt
line. Source text cannot inject ANSI styling. JSON diagnostics, program stderr,
stdout, exit codes and runtime failure-report documents keep their existing
formats; no excerpt or color field is added to JSON.

When a position is unavailable, the renderer omits the line/column prefix.
An available path can still identify the file:

```text
demo.fpasprj: error[FP4105]: Invalid `project.kind` value `app` in `demo.fpasprj`.
  help: Use `program`, `library`, or `test`.
```

Correct the manifest to `kind = "program"`, `"library"`, or `"test"`. A `uses`
entry that names a missing unit is located at the import:

```text
src/a.fpas:2:6: error[FP4115]: Unknown unit `Demo.Missing` in unit `Demo.A`.
  2 | uses Demo.Missing as Missing;
    |      ^^^^^^^^^^^^
  help: Known units in `Demo`: Demo.A, Demo.B.
```

The hint lists units in the missing unit's namespace, otherwise units with the
same root segment, otherwise the project's non-`Std` units; long lists show ten
names and the number of omitted units.

Multi-line messages, such as TOML parser excerpts, continue on lines prefixed
with `message:`.

## Required qualification

FP2022 rejects an import without its explicit alias and shows an aliased `uses`
entry. FP3043 rejects an imported short name or a user enum variant without its
type qualifier. Each diagnostic shows the required source spelling. For example:

```pascal id=example-4 error=FP3043
program ReadFile;

uses Std.Console as Console, Std.Fs as Files;

begin
  const Text: result of (string, string) := ReadText('input.txt');
end.
```

The help lists `Console.ReadText` and `Files.ReadText`. Use
`Files.ReadText('input.txt')` to select the file operation. This rule applies even
with a single import or an unambiguous expected type. User enum variants use
`Type.Variant` locally and `Alias.Type.Variant` through an import; Some/None/Ok/Error
remain reserved language forms. Dot calls
resolve declared methods and native operations; see [dot calls](../language/functions/fluent-calls.md).
Migration hints for removed native free operations, such as `Length(Text)`,
recommend the implemented `Text.Length()` form.

## Diagnostics for other-language spellings

The compiler recognizes common Delphi, Free Pascal and Ada spellings that FPAS
rejects and answers with the FPAS form instead of a generic syntax error. One
habit produces one diagnostic; parsing continues as if the FPAS form had been
written, so later errors in the file are still reported. Valid FPAS is never
reported: recognition starts only after the canonical parse has failed.

The help text names why the foreign form does not apply and ends with
"FPAS writes `…`." In JSON output, `found` holds the foreign text and `expected`
its FPAS replacement. FP2023 marks constructs that FPAS has no form for; other
habits keep the code of the underlying error with a specific message and help.

```pascal id=example-5 error=FP2023
program ResultHabit;

function Answer(): integer;
begin
  Result := 42;
end function;

begin
end.
```

The help reads "FPAS writes `return 42`."

| Habit | Origin | FPAS form | Code |
| --- | --- | --- | --- |
| `{ comment }`, `(* comment *)` | Pascal | `// comment` | FP1013 |
| `(const S: string)` parameter modifier | Delphi | `(S: string)`; parameters are read-only | FP2023 |
| `procedure Q;`, `function F: T;` | Pascal | `procedure Q();`, `function F(): T;` | FP2001 |
| `function F return T is … end F;` | Ada | `function F(): T; begin … end function;` | FP2023 |
| `Result := Value;`, `Result.X := Value;` | Delphi | `return Value;` | FP2023 |
| `F := Value;` inside function `F` | Pascal | `return Value;` | FP3005 |
| `Exit(Value);` | Delphi | `return Value;`, `return;` in a procedure | FP3003 |
| `var` / `const` / `type` section with several declarations | Pascal | Repeat the keyword for each declaration | FP2015 |
| `var I, J: integer` grouped names | Pascal | `var I: integer := 0; var J: integer := 0;` | FP2015 |
| `X : integer := 1;` without keyword | Ada | `var X: integer := 1;` | FP2015 |
| `var Total: integer;` without initializer | Pascal | `var Total: integer := 0;` | FP2001 |
| `const N = 3;` | Pascal | `const N: integer := 3;` | FP2023 |
| `array[0..2] of integer = (3, 7, 12)` | Pascal | `array of integer := [3, 7, 12]`, zero-based | FP2023 |
| `var` / `const` section between a routine header and `begin` | Pascal | Local `var` / `const` statements after `begin` | FP2023 |
| `type TFoo = class … end;` | Delphi | `type TFoo = record … end record;` with methods taking `Self` | FP2023 |
| `interface` / `implementation` sections, unit `end.` | Delphi | One declaration list, exports marked `public`, `end unit;` | FP2023 |
| `TList<integer>`, `Name<T>` | Delphi | `array of integer`, `Name of T` | FP2001 |
| `uses SysUtils;` and other Delphi run-time units | Delphi | `Std` units with an alias, such as `uses Std.Parse as Parse;` | FP2022 |

## Rust JSON rendering API

`render_json(path, source_text, diagnostic)` serializes one record without a
trailing newline. `fpas check`, `build`, `run` and `test` write these records with
`--diagnostics json`, one per stderr line; see
[machine-readable diagnostics](../program-structure/cli.md#machine-readable-diagnostics).
The caller supplies the name and text matching the diagnostic's source ID.
The renderer does not read files or guess another source when text is unavailable.

```json
{"kind":"diagnostic","code":"FP2001","severity":"error","phase":"parse","source":"parse.fpas","location":{"source_id":0,"start":{"line":2,"column":1},"end":{"line":2,"column":6}},"message":"Expected `;`, found `begin`","expected":";","found":"begin","hint":"Insert `;` here."}
```

| Field | Meaning |
|---|---|
| kind | `diagnostic` |
| code | Stable `FPnxxx` identifier |
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

## Program-output records

A JSON diagnostic stream can also carry text the program wrote to standard error
through child processes (see
[machine-readable diagnostics](../program-structure/cli.md#machine-readable-diagnostics)).
`render_program_stderr_json(text)` serializes one such line:

```json
{"kind":"program-output","stream":"stderr","text":"child err"}
```

Consumers distinguish records by `kind`: `diagnostic` records have the fields
above, while `program-output` records have only `stream` (always `stderr`) and
`text` (one line without its line ending).

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
records have no position when a precise range is unavailable. Read, syntax and
validation failures in a manifest retain that manifest's path, including errors
in dependency manifests. Failures that concern several files may have no single
source path. Unknown or non-exported units in a
unit's `uses` clause retain the importing unit's
path and the span of the imported name. The workspace discovery functions
`load_workspace`, `discover_workspace_file`, `discover_run_project_in_workspace`
and `discover_test_projects_in_workspace` return the same `ProjectError`.

`resolve_program_units(graph, root_uses, root_path)` retains the original
unit-name span for missing (FP4115) and nonexported (FP4116) root imports. The
range ends before an optional `as Alias` suffix. CLI commands and editor
analysis supply the main program's authoritative path; test entries supply
their own path even when they share a prepared unit graph. With `root_path = None`,
the original span and source ID remain available without assigning a
unit or manifest path to the program.

A successfully loaded project reports non-fatal findings in
`LoadedProject::warnings` as `fpas_diagnostics::FileDiagnostic` records: the
shared `Diagnostic` with warning severity and the file it concerns. FP4135 marks a
source file listed more than once (the first occurrence is kept); FP4136 marks a
`program` source that was skipped because it is not an allowed entry file.
Lexer/parser warnings of a successfully parsed source keep their original code
and span with that source's path. The CLI prints them in text form, for example:

```text
src/util.fpas: warning[FP4135]: Duplicate source file was ignored; the first occurrence was retained.
  help: List each source file once in `[sources].include`.
```

`Display` renders the records when requested by a caller; current CLI and editor
adapters explicitly convert to their text interfaces; the CLI writes the same
records as JSON with `--diagnostics json`.

## Editor project diagnostics

`fpas_language_service::LanguageServiceError::Project` retains the original
`ProjectError`. Its `diagnostics()` accessor returns `FileDiagnostic` records
in producer order, including source attribution, spans, hints and expected/found
details. Project discovery, dependency manifests, graph analysis and source
standard-library loading use this transport. Editor source-read failures carry
FP4101, while invalid source UTF-8 carries FP4102 with a UTF-8 correction hint.
Both retain their file path without an invented position.

Source-driven project discovery validates open editor buffers before disk text,
including dependency sources and editable standard-library units. Correcting a
source parser error without saving restores project analysis; the old disk
error is not mapped onto the corrected buffer. Manifest loading and source
selection still use the files on disk.

The LSP publishes a located project record at its authoritative source URI,
using that source's snapshot to convert the range to UTF-16. Project graph IDs
remain producer-local; file attribution identifies the snapshot for conversion.
The standard diagnostic fields preserve code, severity and the displayed message
with help. `Diagnostic.data` retains the original `source`, `source_id`, primary
`message`, `hint`, `expected` and `found` fields.

LSP document diagnostics require a range. Records without a usable source range
are rendered through `window/logMessage` with their original code, severity,
available file path and help. They produce no document marker. Correcting or
closing an analysis origin removes its related markers; records still supplied
by another active origin are retained. Editor versions and snapshot revisions
prevent stale records from being applied to a newer version or a reopened buffer.

Root-import errors appear on the main program's imported unit name. Their
positions follow the current editor buffer, and correcting the import clears
the previous marker.

## Build error transport

`fpas_build::BuildError::diagnostics()` exposes the original compiler or parser
records as `FileDiagnostic` entries in producer order. Each entry contains the
shared `Diagnostic` and an optional source path. Unit compilation retains its
unit path when the diagnostic's source ID matches that unit. Unknown or foreign
source IDs are not assigned the current file.

Program-artifact parsing retains all lexer/parser diagnostics when parsing fails,
including expected/found details. Program-artifact compilation retains its
supplied main-source path. Root-import failures likewise retain the first
source path supplied in `ProgramArtifactTarget` and the imported name's original
span. Without a supplied path, the span remains available with no file attribution.
The AST-based `build_program` and `check_program` APIs
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
| Lex | FP1000–FP1999 |
| Parse | FP2000–FP2999 |
| Sema | FP3000–FP3999 |
| Compile | FP4000–FP4099 |
| Project, build, linker, CLI and test runner | FP4100–FP4999 |
| Runtime | FP5000–FP5999; FP5017 remains reserved |
| Internal | FP9000–FP9999; other unassigned ranges also classify as internal |

These identifiers replace the previous `Fxxxx` scheme. Compiler and project
diagnostics share Q01's FP4xxx range, with separate subranges so their `phase`
remains distinguishable. Codes are allocated explicitly; unused values are not
diagnostics. FP1002, FP1003 and FP5017 remain unallocated.

## Code catalog

Each allocated code has a wrong example and a correction. Source examples are
excerpts within an otherwise valid program or unit. Manifest and command examples
name their context. Compiler, linker, runtime invariant and host failure examples
describe invalid input/state; they are not promises that valid FPAS source can
bypass the preceding checks. Correct those artifacts, host conditions or producer
bugs rather than adding a workaround to the program.

### Lexer

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP1001 | Unexpected character | `@` | Remove `@` or use a supported token. |
| FP1004 | Unterminated string | `'hello` | `'hello'` |
| FP1005 | Invalid character code | `#999999999999999999999` | `#65` |
| FP1006 | Invalid hexadecimal literal | `$` | `$FF` |
| FP1007 | Integer literal magnitude overflow | `9223372036854775809` | Use `9223372036854775807` or the direct signed minimum `-9223372036854775808`. |
| FP1008 | Real literal out of range | `1.0e9999` or nonzero `1.0e-9999` | Use a representable finite real, such as `1.0e2`. |
| FP1009 | Invalid exponent | `1e+` | `1e+2` |
| FP1010 | Compiler directive | `{$MODE DELPHI}` | Remove the directive. |
| FP1011 | Invalid digit separator | `1__2` | `1_2` |
| FP1012 | Non-ASCII identifier | `const Größe: integer := 1;` | `const Size: integer := 1;` |
| FP1013 | Invalid comment form | `(* note *)` | `// note` |

### Parser

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP2001 | Expected token | `program Demo begin end.` | `program Demo; begin end.` |
| FP2002 | Expected identifier | `program ; begin end.` | `program Demo; begin end.` |
| FP2003 | Invalid statement start | `begin := 42; end.` | `begin const N: integer := 42; end.` |
| FP2004 | Missing loop direction | `for I: integer := 1 10 do WriteLn(I); end for;` | `for I: integer := 1 to 10 do WriteLn(I); end for;` |
| FP2005 | Expected expression | `const N: integer := ;` | `const N: integer := 1;` |
| FP2006 | Invalid call/assignment | `begin Name; end.` | `begin Name(); end.` when Name is a procedure. |
| FP2007 | Invalid visibility | `program P; public const N: integer := 1; begin end.` | Remove `public` in a program. |
| FP2008 | Invalid static placement | Top-level `static function F(): integer;` | Top-level `function F(): integer;` |
| FP2009 | Nesting limit | Thousands of nested parentheses around `1` | Split the expression into shallow local bindings. |
| FP2010 | Empty record update | `P with end with` | `P with X := 1; end with` |
| FP2012 | Empty enum data list | `type E = enum A(); end enum;` | `type E = enum A; end enum;` |
| FP2013 | Trailing enum field separator | `type E = enum A(X: integer;); end enum;` | `type E = enum A(X: integer); end enum;` |
| FP2014 | Comma/grouped parameters | `function Add(A: integer, B: integer): integer;` or `function Add(A, B: integer): integer;` | `function Add(A: integer; B: integer): integer;` |
| FP2015 | Missing declaration keyword | `const A: integer := 1; B: integer := 2;` | `const A: integer := 1; const B: integer := 2;` |
| FP2016 | Mixed positional and named arguments | `Move(1, Dy := 2)` | `Move(Dx := 1, Dy := 2)` or `Move(1, 2)` |
| FP2017 | Removed record literal | `const P: Point := record X := 1; Y := 2; end;` | `const P: Point := Point(X := 1, Y := 2);` |
| FP2018 | Removed record property | `property Zoom: real read GetZoom;` in a record | Remove the declaration and call `Camera.GetZoom()` |
| FP2019 | `if` expression without `else` | `const N: integer := if Ready then 1 end if;` | `const N: integer := if Ready then 1 else 0 end if;` |
| FP2020 | Statement in an expression branch or arm | `const N: integer := if Ready then 1; else 0 end if;` or `case X of when 1: N := 1; end case` | `const N: integer := if Ready then 1 else 0 end if;` or `case X of when 1: 1; else 0; end case` |
| FP2021 | Positive integer literal outside signed range | `const N: integer := 9223372036854775808;` | Use at most `9223372036854775807`, or write `-9223372036854775808` for the minimum. |
| FP2022 | Missing import alias | `uses Std.Console;` | Write `uses Std.Console as Console;` and use `Console.Name`. |
| FP2023 | Construct from another language | `function F(const S: string): integer;` | `function F(S: string): integer;` |

FP2017 names the declared type when the literal is the initializer of a typed
constant, variable, or field default, for example `Point(X := ..., Y := ...)`;
in other positions the hint uses `TypeName` for the expected record type.

FP2018 names the property's written `read` and `write` methods in its hint, for
example `Value.GetZoom()`; it does not invent accessor names. A record field or
other binding may still be named `property`.

FP2015 reports a declaration group once: a Pascal `var`, `const` or `type` section from its
keyword, grouped names, or declarations without keyword from the first unprefixed name. The help
shows every declaration of the group (up to three) in its complete individual form, including
the actual initializers. For exported groups, repeat `public` in full.

FP2014 points at the offending comma, supplies expected/found details and shows
the complete canonical header in its hint. Recovery stops before the list's
closing parenthesis, retaining the following result type and body. The same check
applies to procedures, record methods, anonymous routines and callable types.
Call arguments still use commas; commas inside types such as
`Result of (integer, string)` remain valid.

### Semantic analysis

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP3001 | Unknown type | `const N: Missing := 1;` | `const N: integer := 1;` |
| FP3002 | Duplicate declaration, repeated import, or conflicting import alias | Two declarations of `N`, `uses Std.Math as Math, std.math as Other;`, or a declaration named like an alias | Keep one import per unit and rename conflicting declarations or aliases. |
| FP3003 | Unknown name or unlisted dot operation | `Missing()` or `Text.Substring(0, 1)` | Declare the routine or use `Text.Slice(0, 1)`. |
| FP3005 | Immutable assignment | `const N: integer := 1;` followed by `N := 2` | Declare `var N: integer := 1;`. |
| FP3006 | Type mismatch | `const N: integer := 'hello';` | `const N: integer := 1;` |
| FP3007 | Argument count | `Add(1)` for a two-parameter Add | `Add(1, 2)` |
| FP3008 | Non-boolean condition | `if 1 then Work()` | `if true then Work()` |
| FP3009 | Invalid panic value | `panic(1)` | `panic('failed')` |
| FP3010 | Break/continue placement | `break` outside a loop | Place `break` inside the loop it exits. |
| FP3011 | Non-exhaustive case | Missing a variant, Boolean value, subrange value, or integer/string fallback | Add explicit missing finite arms or the required scalar `else`; use `if` for selective handling. |
| FP3012 | Enum data count | Construct `A(1)` when A has two data fields | Construct `A(1, 2)`. |
| FP3013 | Generic constraint | Use string for T constrained to an arithmetic type | Use integer for that arithmetic operation. |
| FP3014 | Non-constant or invalid compile-time value | `when ReadValue():` in a scalar `case`, or a computed default in an exported record | Use a guard for a dynamic case condition; use a valid scalar constant expression or optional `None` for an exported default. |
| FP3015 | Missing record field | Construct a record without required X | Supply `X := 1` in typed record construction. |
| FP3016 | Task-bound callable | Pass a closure capturing mutable state into another task | Pass a closure with immutable captures. |
| FP3017 | Private record member | Access another unit's non-public record field | Export the field with `public` or use its public API. |
| FP3018 | Enum backing overflow | Implicit variant after backing value `9223372036854775807` | Assign a smaller unused explicit backing value. |
| FP3019 | Private type in public signature | Public routine returns a private unit type | Export that type or keep the routine non-public. |
| FP3020 | Discard requires a value | `discard Work();` when `Work` is a procedure | Call the procedure directly. |
| FP3022 | Unused function result | `Compute();` or a postfix chain ending in a function | Consume the value or use `discard` when permitted. For `Result`, handle with `case` or propagate with `try` while consuming the success value. |
| FP3023 | Cyclic type alias | `type A = B; type B = A;` | End the alias chain at a concrete type; use a record or enum for finite recursion. |
| FP3024 | No finite recursive value | `type Node = record Next: Node; end record;` | Break the required cycle with `Option`, an empty container, or a finite enum alternative. |
| FP3025 | Invalid named argument | `Sub(Left := 1, Rigth := 2)` | `Sub(Left := 1, Right := 2)` |
| FP3026 | Named arguments not supported | `WriteLn(Text := 'hello')` for a variadic routine | `WriteLn('hello')` |
| FP3027 | `var` marker mismatch | `Increase(Counter)` for `procedure Increase(var Value: integer)` | `Increase(var Counter)` |
| FP3028 | Invalid `var` argument | `Increase(var Fixed)` for a `const` binding `Fixed` | Declare `var Fixed: integer := 1;` and pass `var Fixed`. |
| FP3029 | Aliased `var` arguments | `Swap(var A[I], var A[J])` | `SwapAt(var A, I, J)` |
| FP3030 | `var` parameter escapes the call | A closure that captures `var` parameter `Total` | Capture a local copy `const Current: integer := Total;` |
| FP3031 | Implicit pattern binding | `when Some(Value):` or `when N if N > 0:` | `when Some(const Value):` or `when const N if N > 0:` |
| FP3032 | Misplaced scalar binding | `when const N:` without a guard | Add a condition, for example `when const N if N > 0:` |
| FP3033 | Duplicate or unreachable case label | A repeated constant, an empty range, or a label fully covered by preceding unguarded arms | Remove the redundant label, fix the range, or place a special case before broader coverage. |
| FP3034 | Binding in an `is` test | `X is Some(const Value)` | Use `Some(_)` to test presence or a `case` arm to bind `Value`. |
| FP3035 | Else on a finite case | `else` on enum/Option/Result/Boolean/subrange selectors, including after full coverage | Write explicit missing finite arms or remove a redundant `else`; use `if` for selective handling. |
| FP3036 | Local type annotation required | `const Value := None;`, `const Values := [];`, or `const Present := Some(None);` | Add `: Type`, for example `const Values: array of integer := [];`. Every fully determined statement-local initializer can infer its type. |
| FP3037 | Invalid contract expression | `requires ReadValue() > 0;` or a mutable outer binding in a clause | Use parameters, immutable values, operators, typed construction, and the permitted native receiver queries; see [Preconditions](../language/functions/contracts.md). |
| FP3038 | Go outside its owning scope | `const Job := go Work();` without a same-routine scope, including in named/anonymous helpers | Open `scope ... end scope;` in that routine or pass an explicit TaskGroup; see [Task scopes](../language/concurrency/task-scopes.md). |
| FP3040 | Task handle escape | `return go Fetch(1);` or `Outer := go Fetch(1);` inside a `scope` | Keep scope-owned handles in bindings declared inside the scope and wait for them there; see [Task scopes](../language/concurrency/task-scopes.md#handle-escapes). |
| FP3041 | Uninitialized or unprovable dependency | An initializer calls a helper that reads a later binding, directly or through a callback/default, or captures an unavailable local. | Initialize the dependency before use or pass initialized data explicitly; see [Declaration order](../language/types/declaration-order.md). |
| FP3042 | Foreign mutable worker state | A worker reads or writes a mutable program/unit global through a routine, helper, or callback | Pass an input value from the starting task, mutate a worker-local copy, or communicate through channels; see [Go](../language/concurrency/go.md). |
| FP3043 | Required source qualifier | Imported `WriteLn(...)` or user enum variant `Red` | Use `Console.WriteLn(...)`, `Color.Red`, or the applicable `Alias.Type.Variant`. |
| FP3044 | Duplicate enum backing value | `First = 1; Second = 1;` or `First = 1; Restart = 0; Second;` | Assign an unused value, including for implicitly numbered variants. |
| FP3045 | Backing value in a data enum | `Empty = 7; Number(Value: integer);` in the same enum | Remove the backing annotation: `Empty; Number(Value: integer);`. |
| FP3046 | Write to a for-in source | Assignment, InPlace, var forwarding, a helper/callback write or unknown callable write effects while iterating a writable array/dictionary root | Move the write after the loop or iterate an explicit independent copy. Field/element sources protect their containing root; see [For-in](../language/control-flow/for-in.md). |

FP3025 also reports a parameter or payload field named twice and a named
call or pattern that omits one; its hint lists the visible static names.
Some/Ok/Error use the payload label `Value`. FP3026 applies to variadic
calls such as `WriteLn` and native `Format`, and misplaced named arguments.
Fixed-signature callable values, receiver operations and fixed-arity
polymorphic Std operations accept fully named calls.

FP3027 hints show the marked argument, for example `var Counter`, or ask to
remove `var` for a read-only parameter. FP3028 names the reason a `var`
argument is not writable storage (a `const` binding, read-only parameter, loop
variable, dictionary entry, string character, or computed value).
FP3030 covers closures, routine values, and `go` calls that would let a `var`
parameter outlive its call.

FP3031 reports a plain identifier in a pattern field and a bare scalar label
that names no constant; its hint shows the `const Name` form. FP3032 reports a
scalar `const Name` label without a guard, next to other labels in its arm, or
in a Result, Option, or data-enum `case`. FP3033 reports duplicate labels within
an arm, labels fully covered by earlier unguarded arms, and labels that match no
value. It applies to every supported selector type, including scalars and ranges.
FP3034 rejects every `const Name` inside an `is` pattern, including nested
bindings. Nonbinding `is` tests are ordinary Boolean expressions.

FP3035 rejects `else` on enums, `Option`, `Result`, Boolean, and subranges,
including aliases and imported types. Distinct follows its supported base type.
With valid labels, it names missing finite values or variants, including nested
payload patterns. Guards never count as coverage, even constant-true guards.
When explicit arms already cover every value, its hint asks to remove the
redundant `else`. FP3011 also requires explicit arms; it never recommends an
`else` for finite types. Integer/string cases require `else`, in statements and
expressions alike. Subrange coverage uses intervals without enumerating values.

Ordinary declaration, assignment, return and argument type compatibility checks
supply type names in `expected` and `found` for FP3006. Other uses of that code
can describe a structural restriction and leave those fields null.

### Compiler

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP4001 | Invalid designator base | AST references a base absent from its checked scope | Compile the AST with its matching analysis and declarations. |
| FP4002 | Invalid assignment target | Lowering input tries to assign to a non-addressable target | Supply a checked variable/field target. |
| FP4003 | Intrinsic arity | Lowering input supplies one operand to a two-operand intrinsic | Supply both checked operands. |
| FP4004 | Unsupported intrinsic lowering | Intrinsic metadata has no matching lowering case | Use the registered compiler/runtime intrinsic mapping. |
| FP4005 | Invalid mutable array target | Mutable-array lowering receives a non-addressable expression | Supply its checked mutable array binding. |
| FP4006 | Invalid go expression | Direct compiler input uses `go` with no callable invocation | Supply a checked call such as `go Work()`. |
| FP4007 | Bytecode operand overflow | A constant/function table index exceeds its encoded width | Reduce the table or split the generated program. |
| FP4008 | Function can finish without a return value | `function F(X: integer): integer; begin if X > 0 then return X; end if; end function;` | Add an `else` arm or a final `return Value;` so every normal path returns. |

### Project, build, linker, CLI and test runner

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP4101 | Source read failure | Read missing `src/main.fpas` | Restore the file at the configured source path. |
| FP4102 | Invalid source UTF-8 | Source contains byte `0xFF` | Save the source as UTF-8 text. |
| FP4103 | Manifest read failure | Load missing `app.fpasprj` | Supply an existing readable manifest. |
| FP4104 | Manifest syntax/schema | Manifest contains `[project` | Use `[project]` and schema-valid fields. |
| FP4105 | Invalid manifest value | `[project] kind = "app"` | `kind = "program"` |
| FP4106 | Invalid manifest path | Program main is `src/main.txt` | Point main to an existing `.fpas` file. |
| FP4107 | Source pattern | Include glob matches no source | Include a glob matching the project's actual `.fpas` files. |
| FP4108 | Duplicate manifest entry | Same library path listed twice in dependencies | List the dependency once. |
| FP4109 | Non-library dependency | Depend on a program project | Depend on a `kind = "library"` project. |
| FP4110 | Library cycle | Library A depends on B and B depends on A | Move shared code into C and remove the cycle. |
| FP4111 | Source ownership | Two projects include the same source file | Give each file one owning project. |
| FP4112 | Unit kind mismatch | Library source declares `program Util;` | Declare `unit App.Util;`. |
| FP4113 | Unit namespace | User library declares `unit Std.Custom;` | Use an application namespace such as `App.Custom`. |
| FP4114 | Duplicate unit | Two source files declare `unit App.Util;` | Keep one declaration or rename one unit. |
| FP4115 | Unknown unit | `uses App.Missing as Missing;` with no such unit | Include and declare `App.Missing`, or correct the import. |
| FP4116 | Unexported unit | External project imports a library's internal unit | Add the unit to `[exports].units` or use an exported unit. |
| FP4117 | Unit cycle | App.A uses App.B and App.B uses App.A | Extract shared declarations into a third unit. |
| FP4118 | Source ID overflow | Graph needs more source IDs than u32 can represent | Reduce the graph's source count. |
| FP4119 | Changed source snapshot | Rewrite a source after its graph snapshot was taken | Reload the graph after saving the file. |
| FP4120 | Directory read failure | Discover projects in a missing/unreadable directory | Use an existing readable project directory. |
| FP4121 | Discovery ambiguity | Run a workspace with two candidate program projects | Select one program's `.fpasprj` explicitly. |
| FP4122 | Unknown workspace dependency | `[dependencies].workspace` names no member | Use an existing member's `project.name`. |
| FP4123 | Invalid standard library | Library directory lacks its trusted manifest | Use an intact standard-library distribution. |
| FP4124 | Artifact I/O | Output file cannot be created in an unwritable directory | Select a writable output directory. |
| FP4125 | Artifact encoding | Runner executable has no valid bundled image | Rebuild it with `fpas build --executable`. |
| FP4126 | Invalid linked object | Object references a missing local definition | Rebuild that object from authoritative source. |
| FP4127 | Missing program entry | Root object has no entry function | Link a compiled program object as the root. |
| FP4128 | Duplicate definition | Linked objects define the same canonical symbol twice | Link a single authoritative definition. |
| FP4129 | Incompatible layout | Objects disagree on App.Point's fields | Rebuild consumers against the same source interface. |
| FP4130 | Unresolved import | Object imports App.Util.F with no defining object | Include the defining library/project. |
| FP4131 | Private import | Object imports a private definition from another unit | Import a public definition and rebuild. |
| FP4132 | Incompatible import | Caller and callee disagree on parameter types | Rebuild both with the same signature. |
| FP4133 | Link limit | Linked address/table exceeds its fixed-width capacity | Reduce or split the linked program. |
| FP4134 | Invalid executable | Final bytecode verification rejects a jump target | Rebuild consistent objects with the current toolchain. |
| FP4135 | Duplicate source warning | Include `util.fpas` explicitly and through a glob | Include that file once. |
| FP4136 | Skipped program warning | Include an extra program outside entry-file rules | Remove it from this project's sources or select it as an entry. |
| FP4137 | Invalid CLI arguments | `fpas check --diagnostics xml main.fpas` | `fpas check --diagnostics json main.fpas` |
| FP4138 | Unsupported command input | `fpas run library.fpasprj` | Run a program project that depends on the library. |
| FP4139 | CLI output failure | Write a report into an unwritable directory | Use a writable report destination. |
| FP4140 | Golden stdout mismatch | Test prints `actual` but sidecar expects `expected` | Fix the output or update the sidecar to the approved output. |
| FP4141 | Test timeout | Test exceeds `fpas test --timeout 1 slow_test.fpas` (one second) | Fix the hang or set a suitable timeout in seconds. |
| FP4142 | Runner failure | Test worker cannot be started | Restore the runner executable and its required access. |

### Runtime

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP5001 | Division by zero | Divide by a runtime value equal to zero | Check the divisor or supply a nonzero value. |
| FP5002 | Modulo by zero | Apply modulo with a runtime zero divisor | Check the divisor or supply a nonzero value. |
| FP5003 | Array index or empty Pop/PopInPlace | Index 1 in a one-element array, or pop an empty array | Check array bounds or IsEmpty before popping. |
| FP5004 | Reserved empty-pop code; current array Pop uses FP5003 | — | Check that the array is nonempty before popping. |
| FP5005 | Undefined global | Bytecode addresses an absent global | Rebuild a verified executable with the matching globals. |
| FP5006 | Undefined function | Bytecode calls a missing function ID | Rebuild a verified executable with the matching functions. |
| FP5007 | Wrong call arity | Dynamic call frame has fewer arguments than its signature | Rebuild with the matching callable ABI. |
| FP5008 | Operand type mismatch | VM receives a string where the operation needs integer | Supply a checked integer operand. |
| FP5009 | Intrinsic stack/precondition | Intrinsic receives an incomplete argument stack | Supply the registered intrinsic's checked arguments. |
| FP5010 | Program panic | `panic('boom')` | Remove the intentional panic or handle the failing condition. |
| FP5011 | Console input failure | Read fails after a host input error | Restore the input source or handle input through a result API. |
| FP5012 | Numeric domain | Integer arithmetic overflows, or a real operation produces NaN/infinity | Keep integer results within signed 64-bit bounds and real operands/results finite; ordinary arithmetic does not wrap. |
| FP5013 | Conversion failure | Convert an out-of-range runtime value | Check the conversion's supported range. |
| FP5014 | Console state | Use an operation incompatible with the current console state | Select the supported console mode first. |
| FP5015 | Failed unwrap | Unwrap `None` or an error Result | Match `Some`/`None` or `Ok`/`Error` before accessing the value. |
| FP5016 | Task cancellation | A task cancelled by the debugger or by its task scope | Expected when a scope exits early or fails; start a fresh task rather than resuming a cancelled one. |
| FP5018 | Invalid task | Wait on an absent/invalid task handle | Retain and use a valid handle from the spawned task. |
| FP5019 | Missing dictionary key | Index an absent key | Check membership or insert the key first. |
| FP5020 | VM shutdown | Execute a task after VM shutdown | Keep execution within the live VM lifecycle. |
| FP5021 | String index | Index beyond the string's scalar length | Check `S.Length()`; empty strings have no character index. `Insert` accepts index `S.Length()`. |
| FP5022 | Format mismatch | Format specifiers and arguments have different counts/types | Match each specifier with an argument of its required type. |
| FP5023 | Test assertion | `AssertTrue(false)` | `AssertTrue(true)` after fixing the behavior under test. |
| FP5024 | Unsupported recording effect | Record a host effect that cannot be replayed | Perform the effect outside the recording session. |
| FP5025 | Host random failure | OS random-byte provider fails | Restore the host provider and retry the operation. |

### Internal invariants

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP9001 | Compiler invariant | Compiler metadata is inconsistent with its AST | Reproduce and fix the compiler producer; recheck the source. |
| FP9002 | VM invariant | VM frame/stack metadata is inconsistent | Reproduce and fix the VM producer; rebuild verified bytecode. |
| FP9003 | Project/build invariant | Unit graph refers to a missing graph node | Reproduce and fix graph construction; reload the graph. |

The allocated inventory is maintained in
[`codes.rs`](../../../crates/fpas-diagnostics/src/codes.rs). Its consistency tests
require exactly one catalog row per allocated code, with both example columns
filled; unrelated examples or mere mentions do not satisfy coverage.

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

# Shared diagnostics

The Rust toolchain uses `fpas-diagnostics::Diagnostic` for lexer, parser,
semantic, compiler and runtime errors. Its fields contain a stable code,
severity, message, optional help, optional source span, and optional
producer-supplied expected/found details. A missing position is `None`; it is
not line zero or an invented first line.

## Text output

Known source positions retain this form:

```text
parse.fpas:2:1: error[FP2001]: Expected `;`, found `begin`
  help: Insert `;` here.
```

For example, `program Demo` without its terminating semicolon produces that
error before the following `begin`. Correct the heading to `program Demo;`.

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
| FP1007 | Integer overflow | `9223372036854775808` | `9223372036854775807` |
| FP1008 | Real overflow | `1e9999` | `1e2` |
| FP1009 | Invalid exponent | `1e+` | `1e+2` |
| FP1010 | Compiler directive | `{$MODE DELPHI}` | Remove the directive. |
| FP1011 | Invalid digit separator | `1__2` | `1_2` |
| FP1012 | Non-ASCII identifier | `var Größe: integer := 1;` | `var Size: integer := 1;` |
| FP1013 | Invalid comment form | `(* note *)` | `// note` |

### Parser

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP2001 | Expected token | `program Demo begin end.` | `program Demo; begin end.` |
| FP2002 | Expected identifier | `program ; begin end.` | `program Demo; begin end.` |
| FP2003 | Invalid statement start | `begin 42 end.` | `begin var N: integer := 42 end.` |
| FP2004 | Missing loop direction | `for I := 1 10 do WriteLn(I)` | `for I := 1 to 10 do WriteLn(I)` |
| FP2005 | Expected expression | `var N: integer := ;` | `var N: integer := 1;` |
| FP2006 | Invalid call/assignment | `begin Name end.` | `begin Name() end.` when Name is a procedure. |
| FP2007 | Invalid visibility | `program P; public const N: integer := 1; begin end.` | Remove `public` in a program. |
| FP2008 | Invalid static placement | Top-level `static function F(): integer;` | Top-level `function F(): integer;` |
| FP2009 | Nesting limit | Thousands of nested parentheses around `1` | Split the expression into shallow local bindings. |
| FP2010 | Empty record update | `P with end with` | `P with X := 1; end with` |
| FP2011 | Event accessor order | `event E: procedure() write Add read Get;` | `event E: procedure() read Get write Add;` |
| FP2012 | Empty enum data list | `type E = enum A(); end;` | `type E = enum A; end;` |
| FP2013 | Trailing enum field separator | `type E = enum A(X: integer;); end;` | `type E = enum A(X: integer); end;` |
| FP2014 | Comma/grouped parameters | `function Add(A: integer, B: integer): integer;` or `function Add(A, B: integer): integer;` | `function Add(A: integer; B: integer): integer;` |

FP2014 points at the offending comma, supplies expected/found details and shows
the complete canonical header in its hint. Recovery stops before the list's
closing parenthesis, retaining the following result type and body. The same check
applies to procedures, record methods, anonymous routines and callable types.
Call arguments still use commas; commas inside types such as
`Result of integer, string` remain valid.

### Semantic analysis

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP3001 | Unknown type | `var N: Missing := 1;` | `var N: integer := 1;` |
| FP3002 | Duplicate declaration | Two `const N: integer := 1;` in one scope | Keep one declaration or give them different names. |
| FP3003 | Unknown name | `Missing()` without a declaration | Declare `procedure Missing(); begin end procedure;`. |
| FP3004 | Ambiguous imported name | `Length(Value)` with conflicting imported Length routines | `Std.Str.Length(Value)` for a string. |
| FP3005 | Immutable assignment | `var N: integer := 1;` followed by `N := 2` | Declare `mutable var N: integer := 1;`. |
| FP3006 | Type mismatch | `var N: integer := 'hello';` | `var N: integer := 1;` |
| FP3007 | Argument count | `Add(1)` for a two-parameter Add | `Add(1, 2)` |
| FP3008 | Non-boolean condition | `if 1 then Work()` | `if true then Work()` |
| FP3009 | Invalid panic value | `panic(1)` | `panic('failed')` |
| FP3010 | Break/continue placement | `break` outside a loop | Place `break` inside the loop it exits. |
| FP3011 | Non-exhaustive case | A boolean case covering only `true` | Add the `false` branch or an `else` branch. |
| FP3012 | Enum data count | Construct `A(1)` when A has two data fields | Construct `A(1, 2)`. |
| FP3013 | Generic constraint | Use string for T constrained to an arithmetic type | Use integer for that arithmetic operation. |
| FP3014 | Non-constant value | `const Text: string := ReadLn();` | `var Text: string := ReadLn();` |
| FP3015 | Missing record field | Construct a record without required X | Supply `X := 1` in the record literal. |
| FP3016 | Task-bound callable | Pass a closure capturing mutable state into another task | Pass a closure with immutable captures. |
| FP3017 | Private record member | Access another unit's non-public record field | Export the field with `public` or use its public API. |
| FP3018 | Enum backing overflow | Implicit variant after backing value `9223372036854775807` | Assign a smaller unused explicit backing value. |
| FP3019 | Private type in public signature | Public routine returns a private unit type | Export that type or keep the routine non-public. |
| FP3020 | Discard requires a value | `discard Work();` when `Work` is a procedure | Call the procedure directly. |
| FP3021 | Unsafe discard | A task handle, task-containing aggregate, or callable with unverified captures | Retain and consume handles; use task-free captures or constraints. For a direct `discard go Worker();`, use `go Worker();`. |

Ordinary declaration, assignment, return and argument type compatibility checks
supply type names in `expected` and `found` for FP3006. Other uses of that code
can describe a structural restriction and leave those fields null.

### Compiler

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP4001 | Invalid designator base | AST references a base absent from its checked scope | Compile the AST with its matching analysis and declarations. |
| FP4002 | Invalid assignment target | Lowering input tries to assign to a non-addressable target | Supply a checked mutable variable/field target. |
| FP4003 | Intrinsic arity | Lowering input supplies one operand to a two-operand intrinsic | Supply both checked operands. |
| FP4004 | Unsupported intrinsic lowering | Intrinsic metadata has no matching lowering case | Use the registered compiler/runtime intrinsic mapping. |
| FP4005 | Invalid mutable array target | Mutable-array lowering receives a non-addressable expression | Supply its checked mutable array binding. |
| FP4006 | Invalid go expression | Direct compiler input uses `go` with no callable invocation | Supply a checked call such as `go Work()`. |
| FP4007 | Bytecode operand overflow | A constant/function table index exceeds its encoded width | Reduce the table or split the generated program. |

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
| FP4115 | Unknown unit | `uses App.Missing;` with no such unit | Include and declare `App.Missing`, or correct the import. |
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
| FP4141 | Test timeout | Test exceeds `--timeout-ms 1` | Fix the hang or set a suitable timeout. |
| FP4142 | Runner failure | Test worker cannot be started | Restore the runner executable and its required access. |

### Runtime

| Code | Cause | Wrong example | Corrected example |
|---|---|---|---|
| FP5001 | Division by zero | Divide by a runtime value equal to zero | Check the divisor or supply a nonzero value. |
| FP5002 | Modulo by zero | Apply modulo with a runtime zero divisor | Check the divisor or supply a nonzero value. |
| FP5003 | Array index | Index 1 in a one-element array | Use index 0, or check the array length. |
| FP5004 | Pop empty array | Pop an empty array | Check that the array is nonempty before popping. |
| FP5005 | Undefined global | Bytecode addresses an absent global | Rebuild a verified executable with the matching globals. |
| FP5006 | Undefined function | Bytecode calls a missing function ID | Rebuild a verified executable with the matching functions. |
| FP5007 | Wrong call arity | Dynamic call frame has fewer arguments than its signature | Rebuild with the matching callable ABI. |
| FP5008 | Operand type mismatch | VM receives a string where the operation needs integer | Supply a checked integer operand. |
| FP5009 | Intrinsic stack/precondition | Intrinsic receives an incomplete argument stack | Supply the registered intrinsic's checked arguments. |
| FP5010 | Program panic | `panic('boom')` | Remove the intentional panic or handle the failing condition. |
| FP5011 | Console input failure | Read fails after a host input error | Restore the input source or handle input through a result API. |
| FP5012 | Numeric domain | Integer computation exceeds its checked domain | Check limits before the computation. |
| FP5013 | Conversion failure | Convert an out-of-range runtime value | Check the conversion's supported range. |
| FP5014 | Console state | Use an operation incompatible with the current console state | Select the supported console mode first. |
| FP5015 | Failed unwrap | Unwrap `None` or an error Result | Match `Some`/`None` or `Ok`/`Error` before accessing the value. |
| FP5016 | Debugger task cancellation | Resume a task cancelled by the debugger | Start a fresh task rather than resuming the cancelled one. |
| FP5018 | Invalid task | Wait on an absent/invalid task handle | Retain and use a valid handle from the spawned task. |
| FP5019 | Missing dictionary key | Index an absent key | Check membership or insert the key first. |
| FP5020 | VM shutdown | Execute a task after VM shutdown | Keep execution within the live VM lifecycle. |
| FP5021 | String index | Index beyond the string's scalar length | Check the scalar length before indexing. |
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

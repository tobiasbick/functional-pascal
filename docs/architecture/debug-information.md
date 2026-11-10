# Portable debug information

The emitter produces portable PDBs matching the generated managed assembly.
The compiler API opts in with CompilationRequest.EmitDebugInformation and returns
CompiledProgram.PortablePdb; the CLI and project SDK enable portable symbols by
default. [Debugging](../pascal/tools/debugging.md) describes their use with .NET tools.

## Source and method identity

PDB documents retain source paths and SHA-256 checksums. Sequence points identify
source statements; generated plumbing has hidden points. Lexical scopes record
local names and lifetimes. PE CodeView and checksum entries identify the matching
portable symbols. Identical source/reference/options produce deterministic
assembly and PDB bytes.

The assembly MVID derives from the serialized PE's emitted code, metadata and
symbol linkage. Its reserved GUID slot is zero while hashing and filled only
afterward. Changed inlined reference constants or other changed emission must
therefore change the module identity even when FPAS source and portable PDB
bytes remain equal. The own debugger uses that identity to reject a different
assembly substituted between preparation and runtime module loading.

A local's lexical debug interval starts after its initializer stores the value
and ends at its enclosing block's end. Before a shadowing declaration initializes,
the outer binding remains visible. Async hoisted intervals use the same starts.

## Passive source storage

FPAS portable PDBs identify source storage with custom debug information kind
`4C590C68-DC5F-4A9D-89A5-B8CE5B937137`. Each blob contains a little-endian UInt32
version (`1`), a byte flag (`0` direct value, `1` compiler StrongBox cell), and
a nonempty serialized source-name string. No trailing data is present.

The parent is a LocalVariable for a mutable captured local, or a FieldDefinition
for a closure capture or source-hoisted async local. Direct closure/async fields
retain flag zero. LocalVariable entries use the ordinary lexical interval;
async fields use the standard hoisted scope at their existing slot ordinal.
Field tokens identify storage even when generated names need collision suffixes.

The own debugger uses this mapping to expose capture source names and the stored
cell Value without running code. A user-supplied StrongBox without a cell flag
remains an ordinary object. A raw view retains the actual closure/cell storage.
Readers reject unknown versions, invalid flags, duplicates and malformed blobs;
they do not infer a source cell merely from its CLR type name. Names are bounded
at 8192 UTF-16 code units. This is private debug metadata, with no new CLR public
API, source syntax or runtime dependency.

Visible sequence-point ranges on one source line end no later than the next
point's starting column. This keeps different statements on that line distinct
for column breakpoints. A while-loop backedge targets the condition's visible
point, so repeated conditions remain source stops rather than hidden plumbing.

The CLI canonicalizes file source paths before compilation; relative command-line
arguments therefore produce absolute PDB documents that debuggers can resolve
from a different launch directory. Stdin keeps its virtual name. Compiler API
callers still supply their own document identities, including virtual paths.

Named nested routines retain source method names. Closure locations and capture
storage remain associated with the originating source. Declarations of local
routines do not introduce executing statements or debugger stops. Initializer and
loop-condition points describe actual execution, including repeat conditions.

## Async methods

Portable metadata associates the kickoff method and state machine, with async
yield/resume offsets and hoisted-local scopes. Generated state-machine fields use
CLR debugger conventions so supported debuggers can inspect parameters and locals
after suspension. Source stepping follows the continuation while compiler
plumbing remains hidden.

Source hoisted fields retain readable names, disambiguated with numeric suffixes
when local/parameter names collide. Compiler-only temporaries are synthesized
fields rather than source-hoisted fields. Hoisted-local scope entries retain
the lexical block's emitted interval; internal/non-hoisted storage has an empty
scope. See [async lowering](async-and-cleanup.md#state-machines) for representation
and enclosing-interval limits.

IDE debugging uses the [FPAS DAP adapter](../pascal/tools/debugging.md).
The [coverage map](../../tests/coverage.md#language-and-metadata) owns symbol,
interop and runtime regression coverage. The
[batch frontend](../pascal/tools/batch-debugging.md) consumes the same symbols.

## Output lifetime

Keep the assembly/PDB pair and corresponding sources together. The CLI publishes
symbols with its other outputs and removes the conventional stale PDB when
--debug none is selected. The API returns symbols without writing files.

The SDK accepts DebugType=portable or none. Symbols propagate through normal
output/reference copying. Missing or altered intermediate symbols invalidate the
compilation fingerprint; Clean removes owned symbols. See the
[project contract](../build/msbuild.md) and [publication boundary](compiler.md#publication-boundary).

## Debugger attributes and value views

FPAS emits standard CLR debugger metadata:

| Metadata | FPAS behavior |
| --- | --- |
| DebuggableAttribute | Portable debug output disables JIT optimization and enables tracking unless Optimize is selected. Optimized output can retain its PDB. |
| DebuggerDisplayAttribute | Records and variant cases display source field labels/values; distinct and subrange domains display their decoded Value; closure environments display their captures. |
| DebuggerTypeProxyAttribute | Private, typed record/case views expose source properties and the variant Tag. Generic views retain the original type arguments and constraints. |
| DebuggerBrowsableAttribute(Never) | Backing fields, encoded domain storage, proxy receivers and async bookkeeping are hidden from the ordinary expansion. Proxied objects also offer the debugger's raw view. |
| CompilerGeneratedAttribute | Generated types, fields and helper methods are identified explicitly. Capture fields remain expandable. |
| DebuggerNonUserCodeAttribute / DebuggerStepThroughAttribute | Generated accessors, comparison/copy/factory helpers and domain methods are skipped during normal stepping. Source routines and field-default expressions retain their source debugging. |
| DebuggerHiddenAttribute | Alias markers, closure/state-machine constructors, async entry bridges, SetStateMachine and proxy methods hide their implementation plumbing. |

Async kickoff methods carry AsyncStateMachineAttribute and DebuggerStepThrough;
MoveNext retains its source sequence points and is not hidden/non-user code.
Source-provided display/proxy attributes take precedence. A source variant display
is also attached to its case types; generated proxies preserve property-level
DebuggerBrowsable settings. Display/proxy evaluation follows the debugger's
normal evaluation policy.

The CLI exposes --optimize; the SDK forwards Optimize, defaulting to false in
Debug and true in Release. The option is part of the compilation fingerprint.
Symbols and JIT optimization are independent; the compiler currently has no
separate optimizing CIL rewrite pass.

Reference mechanisms: [F# code generation](https://github.com/dotnet/fsharp/blob/main/src/Compiler/CodeGen/IlxGen.fs),
[generated attributes](https://github.com/dotnet/fsharp/blob/main/src/Compiler/TypedTree/TcGlobals.fs),
and [FSharp.Core debugger views](https://github.com/dotnet/fsharp/blob/main/src/FSharp.Core/prim-types.fs).

## See also

- [Architecture overview](README.md)
- [Closures](closures.md)
- [Async lowering](async-and-cleanup.md)
- [Profiling](../pascal/tools/profiling.md)

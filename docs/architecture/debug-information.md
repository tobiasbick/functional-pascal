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

The [test coverage](../../tests/coverage.md#language-and-metadata) includes symbol
structure, checksums, scopes, closure metadata, async associations, and actual
source stack traces. FPAS IDE/project loaders and debugger front ends remain
separate tooling work; emitting valid portable symbols does not supply them.

## Output lifetime

Keep the assembly/PDB pair and corresponding sources together. The CLI publishes
symbols with its other outputs and removes the conventional stale PDB when
--debug none is selected. The API returns symbols without writing files.

The SDK accepts DebugType=portable or none. Symbols propagate through normal
output/reference copying. Missing or altered intermediate symbols invalidate the
compilation fingerprint; Clean removes owned symbols. See the
[project contract](../build/msbuild.md) and [publication boundary](compiler.md#publication-boundary).

## See also

- [Architecture overview](README.md)
- [Closures](closures.md)
- [Async lowering](async-and-cleanup.md)
- [Profiling](../pascal/tools/profiling.md)

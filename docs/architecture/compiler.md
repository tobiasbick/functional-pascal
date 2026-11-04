# Compiler pipeline and output

Fpas.Compiler reads explicitly supplied source text and assembly metadata and
produces managed output. It does not execute reference assemblies, generate C#,
or emit a VM program image. The binary writer uses System.Reflection.Metadata
and ManagedPEBuilder from .NET.

## Phases and ownership

| Phase | Implementation and responsibility |
| --- | --- |
| Source and syntax | SourceText, Lexer, and Parser retain source locations and declaration documentation |
| Reference metadata | ReferenceCatalog uses PEReader/MetadataReader for identities, signatures, nullable annotations, forwarding, and accessibility |
| Source declarations | SourceCatalog collects containers, types, and routine signatures before binding bodies |
| Binding | Binder resolves names, overloads, constraints, inference, storage, nullable flow, coverage, and contracts; bound nodes and BoundTree provide the shared typed representation and traversal |
| Preparation/lowering | PreparedCompilation owns validated source, reference metadata, bound operations and async plans, including liveness validation before emission |
| Emission | AssemblyEmitter and BodyEmitter create CIL, metadata, optional portable symbols, and CLR entry points |
| Documentation | DocumentationWriter derives optional XML from resolved declarations and emitted identities |

Each source declares one program or unit. An executable contains one program
and optional units; a library contains units and no program. Container identity
comes from the source header. Declarations precede body binding, so source-list
order does not choose dependencies or prohibit mutually recursive unit calls.
Binding/emission order is canonicalized by qualified container name. Diagnostics
retain the source containing the error; reference-input diagnostics use a stable
source location.

Source records declare their interface graph before constructed field and method
types are validated. Transparent aliases retain resolved targets and receive
complete constraint validation before bodies are bound. This permits recursive/
forward interface relationships while preserving checks against every field
when deciding actual parameterless-constructor eligibility.

Overload resolution and [type inference](../pascal/language/functions/type-inference.md)
finish before emission. Unsupported metadata gets a diagnostic identifying the
member and excluded signature, never an erased object call or dynamic fallback.
Generated aggregate metadata is validated before an imported record, union,
domain, or alias is used; its ABI belongs in the [CLR mapping](../specs/clr-mapping.md).

## Compiler API and output

[Compilation.cs](../../src/Fpas.Compiler/Compilation.cs) defines the public API:

- CompilationRequest supplies CompilationSource values, assembly name, reference
  paths, target kind, debug/documentation switches, and diagnostic policy.
- FpasCompiler.Compile returns a CompilationResult with diagnostics and, on
  success, a CompiledProgram. A successful compilation may still have warnings.
- CompiledProgram contains AssemblyImage, RuntimeConfiguration, optional
  PortablePdb, and optional XmlDocumentation. The API returns data; it does not
  publish files or deploy dependencies.
- FpasCompiler.Analyze accepts an AnalysisRequest from Fpas.Compiler.Analysis
  and returns AnalysisResult with IsComplete, Success, diagnostics and a detached
  SemanticSnapshot on success. Its
  success does not require a program image. It validates explicit sources and
  references without producing assembly, PDB, runtime configuration or XML bytes.

Compile and Analyze share PreparedCompilation: parsing, target/reference checks,
declarations, binding, optional documentation validation and async lowering.
The emitter consumes those prepared plans without rebinding or lowering again.
Required support metadata, including generated domain helpers, is checked in
preparation. Analysis preserves fail-fast diagnostics and warning policy. Project
discovery belongs to [Fpas.Workspaces](workspaces.md).

Analyze accepts a CancellationToken, observed during syntax, metadata, binding,
flow, documentation and lowering work. Cancellation throws OperationCanceledException
without publishing a successful result. Prepared metadata is disposed on all
paths; returned diagnostics do not depend on live reference readers. Validation
of documentation is separate from its XML serialization during compilation.

## Semantic snapshots

Parsing retains complete expression/type extents alongside diagnostic positions.
Binding records actual declarations, overloads, types and explicit occurrences
through Binding.Semantics. Analysis never resolves identifiers again from strings
or indexes generated closure/async storage as source references. SemanticPolicy
and its coverage test require an explicit policy for every expression form.

SemanticSnapshot owns copied SourceDocument texts, source-local symbol IDs,
SemanticSymbol/Type descriptors, SymbolOccurrence roles, ExpressionType facts and
exact CLR exports. Collections are read-only and contain no bound symbols,
metadata readers or open files. Metadata identities include assembly version,
culture/token, actual reference content hash, declaring type, member kind, arity
and signature. Extra export
identities map generated public representations back to their source declarations.
External definitions retain original CLR identities/signatures; their checked
generic substitutions belong to expression/type facts, independently of use order.

SourceSpan uses absolute project-input paths, UTF-16 offsets/lengths and one-based
line/column positions. Direct API inputs retain the paths supplied by the caller.
Spans are half-open; escaped identifiers include their source @. Original line
endings and surrogate pairs are preserved. OccurrenceAt selects an identifier
token; TypeAt selects the smallest indexed expression/type extent. Namespace
selections have no fabricated value type.

Types retain generic identity/substitutions, array rank, callable signatures,
domain/enum distinctions, reference/value classification and nullability.
ExpressionType separately identifies declared types when flow changes the checked
type. Captures share the original binding identity. Symbols and type IDs are
snapshot-scoped; a different input snapshot invalidates those handles. The
fingerprint includes source contents, the actual copied reference bytes, compiler
module identity and semantic policy. Failed analysis publishes no complete index.

The [query layer](workspaces.md#source-queries) bridges prepared FPAS dependencies
and supplies explicit project/solution coverage without executing user code.

The API's debug/documentation switches default off. The CLI defaults to portable
symbols and adds its matching Fpas.Core reference unless one is supplied. Direct
API callers supply that reference when their language metadata requires it.
Core contains metadata attributes; ordinary generic methods, closures, and
typeof do not by themselves add a Core runtime dependency.

Identical inputs produce deterministic assembly and matching symbol bytes.
Reference-list order does not determine output. With debug information disabled,
renaming/reordering equivalent input files does not change assembly ownership or
code; portable symbols additionally retain source paths and checksums.
Turning XML generation on or off does not change assembly/PDB bytes.

## Typed emission

Bound operations preserve exact receiver storage, evaluation order, and generic
parameter identity. Writable value-type receivers use their actual address;
read-only receivers use defensive copies. Generic dispatch uses constrained
calls where needed. Rectangular arrays use actual CLR constructor/Get/Set/Address
signatures; no list/dictionary operations are synthesized.

Constructed owners use TypeSpec and MemberRefs retaining their original !n
signatures; generic methods retain !!n slots and use MethodSpec instantiations.
Typeof emits ldtoken and Type.GetTypeFromHandle in that same context. Event calls
resolve Event/MethodSemantics metadata and validate the requested accessor rather
than guessing accessor names. Attributes bind typed constant arguments and emit
real blobs or supported CLR flags without executing referenced constructors.

Return paths, loops, [closures](closures.md), and
[async cleanup](async-and-cleanup.md) share typed control-flow and storage rules.
Record/union helpers, domain encoding, public alias markers, and cross-assembly
validation follow their owning mapping contract rather than a second type model.

## Publication boundary

The CLI's OutputFiles publishes the assembly, portable symbols, runtime
configuration, and requested XML as one staged operation. Failed compilation
leaves existing output intact. Failed replacement restores prior files;
unrecoverable rollback retains backups and reports their location. Parallel
callers must choose distinct output paths.

The CLI rejects source or reference paths that coincide with its output files.
Reference protection includes the assembly, conventional PDB/runtime configuration,
and requested XML. It also applies when symbols are disabled or the target is a
library, because publication removes stale compiler-owned sidecars.

Executable output gets a runtime configuration; library output removes a former
compiler-owned executable configuration. Disabling symbols removes the
conventional stale PDB. XML generation defaults off; a direct CLI call without
XML enabled leaves unrelated XML files intact. Requested XML may be in a separate
directory or volume and participates in rollback.

The SDK invokes this boundary in a separate bounded process, supplies resolved
references, fingerprints its inputs, and owns intermediate/final dependency and
documentation copying. See the [build contract](../build/msbuild.md) and
[development workflow](../build/development.md) for its configuration and use.

## See also

- [Architecture overview](README.md)
- [Compiler CLI](../pascal/program-structure/cli.md)
- [Compiled program files](../pascal/program-structure/compiled-programs.md)
- [Test coverage](../../tests/coverage.md)

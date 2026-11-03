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
| Lowering | Async plans retain typed operations, captured environments, protected regions, and resumable exits |
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

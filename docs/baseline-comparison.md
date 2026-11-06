# CLR baseline comparison

This comparison covers the agreed language, CLR representation, compiler CLI,
and MSBuild contracts for net10.0/AnyCPU on Windows and Ubuntu/WSL. Requirements
come from the [handbook](pascal/README.md), [grammar](specs/syntax.md),
[CLR mapping](specs/clr-mapping.md), and [MSBuild specification](build/msbuild.md).
Deferred proposals and excluded metadata signatures are separate scope.

The comparison maps each rule group to its implementation and primary proof.
The FPAS [documentation checker](pascal/tools/documentation-examples.md) compiles
published programs/units and explicit fragment contexts. LanguageSpecificationTests
executes its assemblies with existing behavior/metadata assertions. Contexts
supply stated prerequisites and preserve the example's source.
The [coverage map](../tests/coverage.md) identifies the owning test files.

## Language and CLR rules

| Required behavior | Implementation owner | Verification |
| --- | --- | --- |
| Strict UTF-8, initial BOM, supported line endings, ASCII/case-sensitive identifiers, escapes, numeric/string/char literals, line comments | Syntax/Lexer, CLI source input | EmissionTests, ScalarDiagnosticsTests, CompilerCliTests, DocumentationTests |
| Program/unit counts, named terminators, entry args, explicit aliased imports, visibility and duplicate identities | Parser, SourceCatalog, Binder | DiagnosticsTests, MultiSourceTests, RoutineTests, MultiSourceCliTests |
| Explicit initialized bindings, immutable/mutable storage, assignment order and definite assignment | Parser.Statements, Binder.Statements, ParameterFlow | DiagnosticsTests, ExpressionDiagnosticsTests, ParameterDiagnosticsTests, ParameterExecutionTests |
| Primitive CLR identities, widths, native integers, decimal, numeric promotion, checked/unchecked behavior, UTF-16 | TypeFacts, NumericFacts/Value, numeric binding/emission | ScalarMetadataTests, NumericExecutionTests, PrimitiveExecutionTests, MsBuildTests.Scalars |
| Transparent aliases, nominal domains, enum constants, full-width ranges, checked construction and lower-bound defaults | SourceCatalog.Scalars, scalar metadata/emission | ScalarDiagnosticsTests, ScalarMetadataTests, ScalarExecutionTests, MsBuildTests.Scalars |
| Nullable storage/metadata, stable flow facts, boxing/unboxing, casts, typeof and initialization effects | NullabilityMetadata, Binder.NullableFlow, conversions | InteropDiagnosticsTests, TypeOfTests, GlobalTests, MsBuildTests.Collections/TypeOf/Globals |
| Arrays, rectangular/jagged shape, contextual literals, default elements, indexers and enumeration cleanup | Binder.Arrays/Enumeration, ReferenceCatalog.Indexers | CollectionsDiagnosticsTests, CollectionsExecutionTests, MsBuildTests.Collections |
| Boolean branches, typed counting/enumeration, repeat/while, extreme endpoints, iteration captures and cleanup exits | Binder.Loops, FlowAnalysis, ParameterFlow | LoopTests, LoopExecutionTests, ControlFlowExecutionTests, MsBuildTests.Loops |
| Overloads, inference, named/optional/params arguments, exact ref/out/in storage and readonly defensive copies | SourceCatalog.Routines, Binder.Arguments/Inference, ReferenceCatalog | GenericRoutineTests, InferenceTests, ParameterDiagnosticsTests, MsBuildTests.Inference/ContractsAndParameters |
| Delegates, lexical recursion, shared mutable cells, escaping closures, generic environments and capture initialization | Binder.Callables/NestedRoutines, closure emission | CallableDiagnosticsTests, NestedRoutineTests, CallableExecutionTests, MsBuildTests.Callables/NestedRoutines |
| Records, immutable properties, defaults, private construction, static methods, shallow updates and field equality | SourceCatalog.Records, Binder.Records, record emission | RecordTests, RecordMetadataTests, RecordExecutionTests, MsBuildTests.Records |
| Closed generic records/unions, constraints and complete interface declarations before field/signature validation | SourceCatalog.TypeGenerics/Interfaces/Scalars, ReferenceCatalog.SourceGenerics | GenericTypeTests, ScalarDiagnosticsTests, MsBuildTests.GenericTypes/Baseline |
| Union constructors and contextual inference, immutable payloads, typed/nested/range patterns and exhaustive coverage | Binder.Unions/Patterns, PatternCoverage, union metadata | UnionTests, CaseCoverageTests, UnionMetadataTests, MsBuildTests.Unions/Inference |
| Imported CLR inheritance/members, real struct storage, interface implementation/defaults and unsupported signatures | ReferenceCatalog, Binder.Members, member emission | InteropDiagnosticsTests, OverloadAndInterfaceDiagnosticsTests, MsBuildTests.Interop/Interfaces/SignatureLimits |
| Nested generic CLR paths and operators/static members through constrained interfaces | SourceCatalog.TypePaths, Binder.Operators, constrained CIL calls | MsBuildTests.NestedTypes/GenericMath |
| Event metadata/accessor selection, delegate identity, ordered operands, mutation and suspension | ReferenceCatalog.Events, Binder.Events | EventTests, EventMetadataTests, MsBuildTests.Events/EventDiagnostics |
| Attribute target/usage/accessibility, typed constants and supported CLR flags without executing reference code | Binder.Attributes/AttributeValues, attribute emission | AttributeTests, MsBuildTests.Attributes/AttributeDiagnostics |
| Ordered exceptions/filters, stack-preserving rethrow, finally and reverse once-only disposal | Binder.Exceptions, CLR EH and cleanup emission | InteropExecutionTests, ContractExecutionTests, MsBuildTests.Interop/Loops |
| Requires/ensures in every configuration, restricted expressions, retained returns and successful cleanup boundary | Binder.Contracts, routine/async lowering | ContractDiagnosticsTests, ContractExecutionTests, MsBuildTests.ContractsAndParameters |
| Real Task state machines, standard Task/ValueTask awaitables, context/cancellation/faults and awaited disposal | Binder.Async, async plans/emission | AsyncDiagnosticsTests, AsyncExecutionTests, MsBuildTests.Async/Events |
| Declaration-order-independent signatures and CLR global initialization/cycles | SourceCatalog, Binder.Globals, container emission | MultiSourceTests, GlobalTests, GlobalExecutionTests, MsBuildTests.Globals/Baseline |
| Actual CIL/metadata, generic slots and deterministic PE/PDB images | AssemblyEmitter, BodyEmitter | EmissionTests, GenericRoutineMetadataTests, GenericTypeMetadataTests, PortableDebugInfoTests |
| Matching portable symbols, source checksums, local/closure scopes and async stepping metadata | PortableDebugInfo, emission sequence points | PortableDebugInfoTests, DebugSymbolsCliTests, MsBuildTests.DebugSymbols |
| Attached Markdown, structured documentation warnings, exact public XML IDs and deterministic escaping | DocumentationWriter, DocumentationIds, diagnostic policy | DocumentationTests, DocumentationCliTests, MsBuildTests.Documentation/DocumentationIds |

These rows include negative controls: unsupported constructs/signatures,
inaccessible or conflicting declarations, invalid generic arguments, unsafe
nullable/ref usage, incomplete patterns, forbidden contract expressions, and
invalid attributes must fail before successful output publication. Successful
execution alone is not the diagnostic contract.

## CLI and build acceptance

| Required behavior | Implementation owner | Verification |
| --- | --- | --- |
| Explicit references, target/output/debug/XML options, strict stdin and structured diagnostics | Fpas.Cli | CompilerCliTests, MultiSourceCliTests, DocumentationCliTests |
| JSON response files, paths with spaces, source/output collision protection, transactional publication and rollback | CLI arguments and OutputFiles | CompilerCliTests, DebugSymbolsCliTests, DocumentationCliTests |
| Versioned local SDK, language-neutral restore/targets and exactly selected FPAS source items | Fpas.NET.Sdk, Fpas.Build.Tasks | SdkPackageFixture, MsBuildTests source-selection and package-reference cases |
| .fpasproj/.csproj/.fsproj graph and .slnx project mapping, library/executable output, net10.0/AnyCPU, Debug/Release | SDK props/targets, standard project references | MsBuildTests.Baseline, mixed solution and example builds |
| Framework/project/package/Core dependencies and runtime assets through normal restore and reference negotiation | Standard .NET/NuGet targets and SDK output items | MsBuildTests package/transitive-reference cases and Baseline |
| CoreCompile, GetTargetFrameworks, GetTargetPath, real assembly metadata and no C# generation | FpasCompile and SDK targets | MsBuildTests.Acceptance and mixed consumers |
| No-change skip; source/reference list/content, compiler/task/options and missing/altered outputs invalidate compilation | CompileFingerprint | MsBuildTests incremental/reference/compiler-assets cases |
| Stable source-located errors/warnings, nonzero failures, cached warning replay and preserved previous outputs | FpasCompile diagnostics and CLI publication | MsBuildTests diagnostics and documentation-policy cases |
| Clean/Rebuild, isolated parallel configurations/projects and equivalence to clean output | SDK cleanup/file tracking and standard project graph | MsBuildTests.Acceptance, DebugSymbols and Documentation |
| Optional/default/custom XML paths, reference sidecar propagation, toggle/repair and ownership-aware cleanup | DocumentationOutputs, FpasCleanDocumentation | MsBuildTests.Documentation/DocumentationIds |

## Findings addressed by the comparison

The source interface graph was populated after constructed record fields and
method signatures were checked. A valid `Box<Item>[]` field was rejected when
Box constrained T to IComparable<T> and Item implemented IComparable<Item>.
The reduced program failed with FP3018 before the correction.

Record interfaces are now declared before field/signature validation. Alias
targets resolve without prematurely checking an incomplete graph; their complete
constructed constraints are validated before body binding. Constructor checks
still use all declared fields, so a forward record with required fields cannot
satisfy new(). MsBuildTests.Baseline executes recursive fields, an alias, a
constrained union, and instance interface dispatch from a real C# host.
ScalarDiagnosticsTests separately rejects an unused invalid constructed alias;
the existing GenericTypeTests retain the forward-constructor negative control.

The original build contract names F# projects as well as C# projects. The new
mixed-solution acceptance uses FSharpShared.fsproj → Game.fpasproj → C# and F#
hosts in Debug/Release. FSharp.Core is restored from the installed SDK's explicit
offline package source into a fresh fixture package directory, and normal output
copying supplies FSharp.Core, Fpas.Core and project assemblies. No runtime DLLs
are copied manually and no public package service is required by this check.

## Validation and boundaries

The toolchain is built with the documented Debug configuration and warnings as
errors, followed by the complete compiler and integration suites on Windows and
Ubuntu/WSL. Mixed-project cases also select Debug/Release consumer configurations.
Handbook/agent-reference checks validate current copied sources, complete example
behavior, local links/anchors and the exact o200k_base token budget. The source
graph regression is additionally checked against the unchanged compiler.

Within these contracts, the comparison leaves no confirmed baseline compiler or
MSBuild omission open. This is bounded acceptance, not a claim that all possible
.NET metadata combinations or external tools are supported. Source events, class hierarchies,
unsafe/byref-like features, custom awaiters, async iterators, extension discovery,
and IDE integration retain their documented separate scope. Application publication,
Native AOT and signing follow the [publishing contract](build/publishing.md).

Source interface declarations are now implemented as an approved extension;
see [interfaces](pascal/interop/interfaces.md) and SourceInterfaceTests/
MsBuildTests.SourceInterfaces for metadata, diagnostics, and cross-language acceptance.

[Lifted nullable operators](pascal/language/types/nullable-types.md#lifted-value-operators)
are specified and implemented, with NullableOperatorDiagnosticsTests and
MsBuildTests.NullableOperators covering safety, C# results/exceptions, generic
dispatch, and operand ordering across suspension in Debug/Release consumers.

The [standalone SDK walkthrough](build/standalone-sdk.md) covers an independent
FPAS/C# consumer with explicit package sources, versions, and deployment files.
The [library guides](pascal/libraries/README.md) cover practical .NET workflows,
including task synchronization and joined channel pipelines.
[Reproducible measurements](pascal/tools/benchmarks.md) and
[diagnostic workflows](pascal/tools/diagnostic-workflows.md) cover fixed FPAS
workloads, explicit timing boundaries, real .NET traces/counters and controlled
failure dumps, accepted in Debug/Release on Windows and WSL/Linux.

## See also

- [Development and test commands](build/development.md)
- [Verification coverage](../tests/coverage.md)
- [Language scope and deferred proposals](future/language-gaps.md)

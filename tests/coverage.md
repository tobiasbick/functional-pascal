# Verification coverage

This map identifies tests to read or select for a behavior. It describes checked
contracts, not support for every .NET signature or deferred language feature.
Commands and configuration selection belong in the
[development workflow](../docs/build/development.md).

## Language and metadata

Compiler tests bind through FpasCompiler and inspect diagnostics/metadata.
Execution tests invoke the actual CLI and .NET host. Topic-specific MsBuildTests
build mixed FPAS/C# consumers and compare exported contracts with .NET behavior.

| Area | Compiler/metadata tests | Execution and mixed-build tests |
| --- | --- | --- |
| Parsing, annotations, locations, return/unused-result rules | DiagnosticsTests, ExpressionDiagnosticsTests, RoutineTests | CompilerCliTests, PrimitiveExecutionTests, ControlFlowExecutionTests |
| Numeric widths, promotions, checked/unchecked, native/decimal | ScalarDiagnosticsTests, ScalarMetadataTests | NumericExecutionTests, ScalarExecutionTests, MsBuildTests.Scalars |
| Units, visibility, source/reference conflicts, deterministic ordering | MultiSourceTests, EmissionTests | MultiSourceCliTests, MsBuildTests.MultiSource |
| CLR members, boxing, nullable flow, struct storage, cleanup | InteropDiagnosticsTests, NullableEffectTests | InteropExecutionTests, MsBuildTests.Interop |
| Arrays, UTF-16, generic owners, indexers, enumeration | CollectionsDiagnosticsTests | CollectionsExecutionTests, MsBuildTests.Collections |
| Delegates, shared captures, callbacks, method groups | CallableDiagnosticsTests, CallableMetadataTests | CallableExecutionTests, MsBuildTests.Callables |
| Records, defaults/copy helpers, equality, imported metadata | RecordTests, RecordMetadataTests | RecordExecutionTests, MsBuildTests.Records |
| Unions, payload patterns, guards, coverage, imported cases | UnionTests, CaseCoverageTests, UnionMetadataTests | UnionExecutionTests, CaseExecutionTests, MsBuildTests.Unions |
| Generic constraints, closures, type/method parameter identity | GenericRoutineTests, GenericRoutineMetadataTests, GenericTypeTests, GenericTypeMetadataTests | GenericRoutineExecutionTests, GenericTypeExecutionTests, MsBuildTests.GenericRoutines/GenericTypes |
| Nested CLR type paths, enclosing arguments, constraints and identities | | MsBuildTests.NestedTypes |
| Constrained generic operators, checked dispatch, static members/defaults | GenericRoutineTests | MsBuildTests.GenericMath, LanguageSpecificationTests |
| Complete record/interface graph before field and alias constraints | GenericTypeTests, ScalarDiagnosticsTests | MsBuildTests.Baseline |
| Inference, variance, contextual arrays/unions, overload evidence | InferenceTests | InferenceExecutionTests, MsBuildTests.Inference |
| Contracts, ref/out/in, optional/params/named calls | ContractDiagnosticsTests, ParameterDiagnosticsTests | ContractExecutionTests, ParameterExecutionTests, MsBuildTests.ContractsAndParameters/ParameterDiagnostics |
| Source overloads and CLR interface implementation/defaults | OverloadAndInterfaceDiagnosticsTests | OverloadExecutionTests, MsBuildTests.Overloads/Interfaces |
| Async faults, cancellation, references, contexts, disposal | AsyncDiagnosticsTests | AsyncExecutionTests, MsBuildTests.Async |
| Portable PDBs, sequence points, scopes, closures, async metadata | PortableDebugInfoTests | DebugSymbolsCliTests, MsBuildTests.DebugSymbols |
| Comments, XML IDs, public boundaries, warning policy | DocumentationTests | DocumentationCliTests, MsBuildTests.Documentation/DocumentationIds |
| Global fields, visibility, initialization, nullable metadata | GlobalTests | GlobalExecutionTests, MsBuildTests.Globals |
| Local recursion, capture initialization, generic/async environments | NestedRoutineTests | NestedRoutineExecutionTests, MsBuildTests.NestedRoutines |
| Typed/repeat loops, extreme endpoints, flow, iteration captures | LoopTests | LoopExecutionTests, MsBuildTests.Loops |
| Event metadata/accessors, storage, delegates, nullable captures | EventTests, EventMetadataTests | MsBuildTests.Events/EventDiagnostics |
| Type tokens, generic identity, initialization effects | TypeOfTests | TypeOfExecutionTests, MsBuildTests.TypeOf |
| Attribute constants, targets, usage, flags | AttributeTests | MsBuildTests.Attributes/AttributeDiagnostics |
| CLR comparison operators and unsupported signatures | ComparisonOperatorTests | ComparisonExecutionTests, MsBuildTests.Comparisons/SignatureLimits |

Files live in [compiler tests](Fpas.Compiler.Tests/) and
[integration tests](Fpas.IntegrationTests/). MsBuildTests uses topic-based partial
files; the shared fixture owns packaging and process setup.

Compiler files are grouped in Types, Routines, ControlFlow, and Metadata;
integration files in Cli, Execution, Documentation, and Build. Shared setup lives
in each project's Support directory.

## Behavioral checks

Arithmetic checks include signed/unsigned extremes, IEEE NaN, checked overflow,
division failures, decimal behavior, and short-circuit/evaluation order. Loop
checks cover empty/reversed ranges, every primitive width, native endpoints,
subranges, per-iteration cells, repeat effects, out flow, and real async suspension.
The SDK builds Mandelbrot and compares every output character with an independent
System.Numerics.Complex calculation.

Resource fixtures record acquisition, filter, body, and cleanup events across
normal completion, return, break/continue, failed acquisition, throwing cleanup,
and faults/cancellation. They check CLR filter ordering and stack-preserving
rethrow. Async fixtures use controlled completion and synchronization contexts,
including ConfigureAwait(false), rather than timing-only assumptions.

Capture checks cover escaping delegates, sibling mutable cells, per-iteration
values, struct method-group snapshots, conservative nullable facts, and premature
lexical environment construction. Generic checks inspect TypeSpec/MethodSpec and
!n/!!n signatures, constraints, nullable annotations, and cross-assembly consumers.

Aggregate checks cover constructor/default order, private state, copy helpers,
equality/hash behavior, recursive generic unions, and corrupted versioned
contracts. Union coverage has an independent truth-table oracle. Scalar-domain
tests check full-width offsets, valid defaults, checked bounds, nullable
conversions, and identity. Global tests cover parallel first access, forward/cyclic
reads, cached initializer failures, readonly storage, and type tokens avoiding
initialization.

Event tests include independent missing/inaccessible accessors and unconventional
accessor names. Execution covers inherited/generic/static/virtual/interface events,
duplicate removal, operand/accessor exceptions, writable/read-only structs, and
async suspension. Attribute tests instantiate actual .NET metadata with generic
classes, unsigned enums, null/boxed/array values, type tokens, async kickoff
placement, and supported pseudo-flags. A throwing attribute constructor does not
execute during compilation.

Comparison tests exercise both operand owners, ambiguity, once-only evaluation,
and safe nullable flow. Signature-limit builds check methods, method groups,
delegates, fields, properties, and indexers against a real C# reference with
unsupported ref returns, pointers, byref-like types, or required modifiers.
Supported overloads remain usable; excluded ones produce identifying diagnostics.

Nullable-effect regressions check ref/out writes, repeated conditions, catch
filters, case guards, deferred arguments, and captured writes. Unsafe stale facts
must produce FP3013; revalidation and saved-value controls remain accepted.
CompilerOutputOwnershipTests invokes the CLI with reference assemblies placed at
assembly/PDB/runtime-configuration output paths and checks rejection without
modifying their bytes, including disabled-sidecar cleanup.

## Documentation acceptance

LanguageSpecificationTests compiles complete program/unit fences directly from
the copied [repository README](../README.md) and [handbook](../docs/pascal/README.md), with portable PDB and XML output,
then executes programs with explicit assertions. Project-labeled programs compile
with their unit companions; units compile as libraries. New complete programs
require execution assertions.

File examples run in isolated directories and cover success/missing-file cleanup.
The HTTP example uses a local rejecting proxy to exercise exception handling
without external network access. Timer examples do not require exact tick counts.
Domain and contract examples also exercise expected failures.
The [process recipe](../docs/pascal/libraries/processes.md) compiles directly from
Markdown. MsBuildTests.ProcessRecipe calls that exact unit from C# in Debug/Release,
checking argument/environment/cwd isolation, stdin EOF, nonzero exit, bounded
concurrent stdout/stderr capture, timeout/cancellation cleanup, and launch failure.
The environment/time and numeric/random recipes also compile and execute directly
from Markdown. Environment cases cover an absent setting, accepted integer bounds,
whitespace/signs, invalid text, and overflow in isolated child environments. Fixed
offsets and invariant formats make time/numeric output deterministic; elapsed time
has no exact timing assertion. Seeded rolls are compared with a C# Random on the
current runtime, while unseeded generators have range/size checks.
The [HTTP recipe](../docs/pascal/libraries/http.md) has a local rejecting-proxy
Markdown check and Debug/Release SDK cases. A real loopback exchange verifies
GET headers and UTF-8 text; controlled handlers/streams check exact/oversized
limits, invalid UTF-8, status errors, cancellation while sending/reading, disposal
of request/content/body, and continued use of the caller's client.
The [task coordination](../docs/pascal/libraries/task-coordination.md) and
[channel](../docs/pascal/libraries/channels.md) recipes execute from Markdown,
including shared updates, WhenAny/WhenAll outcomes, bounded writer backpressure,
and drained completion. MsBuildTests.CoordinationRecipes calls the exact units
in Debug/Release, checking permit ownership, awaited/faulted/canceled work,
producer and consumer failures, overflow, active cancellation, and enumerator
disposal before a pipeline completes. Controlled signals replace timing races.

AgentReferenceTests checks the compact reference's policy, complete/unique
examples, local links, and exact o200k_base budgets using packaged vocabulary.
HandbookTests checks topic hubs, related navigation, repository documentation
links/anchors (including .agents/.claude skills), explicit syntax fragments, and
exact copied Markdown inputs.
Deleted articles cannot remain as stale fixtures. See
[example conventions](../docs/pascal/tools/documentation-examples.md).

## Build acceptance

StandaloneSdkGuideTests materializes the exact project/configuration/source blocks
from the [standalone SDK walkthrough](../docs/build/standalone-sdk.md) outside the
checkout. Debug/Release cases first reject a missing local SDK package, then
restore it into a fresh explicit cache and build C# Shared → FPAS Messages →
FPAS App/C# Host. They execute both consumers, copy complete outputs, rename the
source/feed/cache directory, and execute the deployed DLLs again. Assertions cover
the expected text and required Core/project/runtime/symbol/documentation files.

MsBuildTests exercises the net10.0/AnyCPU contract using the current locally
packaged SDK. Each test uses an isolated feed/cache identity and fixture directory;
it does not depend on a previously installed development package.

The Shared.csproj → Game.fpasproj → Host.csproj fixture and separate FPAS consumers
verify mixed project/solution edges, framework/assembly/NuGet references, transitive
runtime dependencies, public signatures, and nullable warnings as errors.
MsBuildTests.Baseline also builds FSharpShared.fsproj → Game.fpasproj → C# and F#
hosts in the same solution for Debug/Release. It restores FSharp.Core from the
installed SDK's explicit offline package source into an isolated package directory
and verifies execution with transitive assets in each host output.
Examples build through their actual projects, including units, file/collection
tasks, records/unions, generic pipelines/trees, and lexical helpers.

Incremental checks edit/add/remove sources and references, change compiler/task
assets/options, and delete/corrupt requested outputs. They verify no-change skips,
source diagnostics, input/output protection, prior-output preservation, and
publication rollback. Documentation cases cover custom paths, XML IDs compared
with C#, cached warnings, comment-only edits, generation toggles, C# sidecar
propagation, and Clean retaining unrelated XML.

MsBuildTests.Acceptance covers parallel configurations and independent projects,
GetTargetFrameworks/GetTargetPath, reference-list invalidation, and assembly/PDB/XML
equivalence after a clean rebuild. Debug/Release and portable/none symbols are
covered. Other frameworks/platforms, publishing, AOT, signing, native apphosts,
and IDE integration are outside this verified scope. Remaining requirements stay
in [language gaps](../docs/future/language-gaps.md).

## See also

- [Test organization](README.md)
- [Compiler architecture](../docs/architecture/README.md)
- [CLR mapping](../docs/specs/clr-mapping.md)
- [MSBuild contract](../docs/build/msbuild.md)

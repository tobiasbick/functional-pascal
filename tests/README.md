# Tests

The repository uses xUnit for compiler and end-to-end verification. See the
[coverage map](coverage.md) to select a topic's diagnostics, metadata, execution,
and mixed-project checks. Restore/build/test commands and focused filters live
in the [development workflow](../docs/build/development.md).

## Projects and fixtures

| Location | Responsibility |
| --- | --- |
| [Fpas.Compiler.Tests](Fpas.Compiler.Tests/) | Compiler API diagnostics, source locations, flow analysis, emitted metadata, and deterministic images |
| [Fpas.IntegrationTests](Fpas.IntegrationTests/) | Actual CLI/host execution, handbook examples, and packaged SDK/mixed FPAS/C# projects |
| [Fixtures/msbuild](Fixtures/msbuild/) | Shared.csproj → Game.fpasproj → Host.csproj and a mixed .slnx solution |
| [TestSupport](TestSupport/) and [Directory.Build.targets](Directory.Build.targets) | Shared bounded-process and temporary-directory helpers plus resolved framework reference inputs |

Compiler tests use Syntax, Analysis, Types, Routines, ControlFlow, Formatting, and Metadata directories with
matching namespaces. Integration tests use Cli, Execution, Documentation, Build,
Debugging, Processes, Projects, Formatting, Testing, ApiDocumentation, Editing,
Refactoring, Toolchain, LanguageServer and Workspaces.
Each topic's Support directory owns its shared setup; compiler tests
compile through TestCompilation rather than another test class.

Editing tests exercise real check --fix previews/application, parser/type hints,
fresh SDK invocation, rechecking, review-only diagnostics, encoding, no-op writes,
conflicts, stale inputs, linked edits, cancellation, permissions and edit bounds.
Formatting shares the source-publication service and retains its CLI regressions.

Refactoring tests cover overload/callable selection, source interface families,
generics, named arguments, identifier escaping, shadowing/capture, linked owners,
plan consistency, stale inputs and actual cross-DLL execution after renaming.
AnalysisSignalTests covers initial planning and post-publication cancellation
with Linux SIGINT and native Windows console control events.

Testing contains packaged SDK/native VSTest acceptance for FPAS test libraries:
xUnit, MSTest and NUnit templates, data/async/lifecycle behavior, native reports,
mixed solutions, discovery/filtering, failure paths and owned host cleanup.
[Fixtures/testing](Fixtures/testing/) contains actual FPAS test methods, including
a formatter API regression written in FPAS. C# only drives/asserts these consumers.
See [writing tests](../docs/pascal/tools/testing.md) and [coverage](coverage.md#native-test-frameworks).

ApiDocumentation verifies name-based source/metadata lookup, exact XML IDs against
real C# output, unused framework APIs, public boundaries, generic/array/byref
signatures, source preference, scoped/paged results, missing/invalid XML,
documentation budgets, stale inputs and deadlines. The CLI also runs from a fresh
SDK package; AnalysisSignalTests checks console interruption and preparation cleanup.

Build the toolchain and tests before executing them. Do not rebuild toolchain
binaries while tests use them. Markdown and checked-in examples are copied during
the test build; rebuild after changing those inputs. HandbookTests checks that
the copied Markdown tree exactly matches the sources, including deleted pages.

Toolchain.EnvironmentCommandTests exercises actual .NET selection and a packaged
CLI, expected SDK hashes in a large cache, stale/missing files, broken hosts and
global.json, bounded argument errors and passive packaged-debugger inspection. An FPAS
version-probe fixture verifies deadline and console-cancellation cleanup on
Windows and Linux before test-harness cleanup.

SDK tests package the already built tools with a unique version into an isolated
local feed, copy the fixture into a temporary directory, and build actual
consumers. Those fixture builds are part of the test, not a second toolchain build.
They verify dependency copying, exported signatures, incremental inputs/outputs,
configuration isolation, diagnostics, and cleanup. Do not rely on a cached local
development SDK version.

Profiling.DiagnosticsWorkflowTests builds the actual diagnostic FPAS applications
from a fresh SDK package and restores the pinned local diagnostic tools. It
checks independent arithmetic results, unprofiled CSV samples, rejected child
results, timeout cleanup, real EventPipe stack/GC/allocation events, runtime
counters and managed exception recovery from a heap dump after target exit.
These tests require diagnostic IPC and dump permissions on the native platform;
see [acceptance and retained artifacts](../docs/pascal/tools/diagnostic-workflows.md#acceptance).

Projects.PublishingTests owns application delivery through the CLI and actual
packaged SDK: five publish modes with Debug/Release targets, standalone sources,
mixed solutions and C# hosts, no-build reuse, transactional replacement and
deadline cleanup. It independently verifies RSA strong names and CMS delivery
signatures, including key changes, public/delay signing and tampering. Native AOT
and single-file outputs execute from independent directories with .NET host search
disabled. See [publishing](../docs/build/publishing.md).

Workspaces tests exercise evaluated project/solution contexts, check/query JSON,
exact definitions/types/references, name-search paging, bounded declaration
inspection, linked-source ownership, stale metadata,
bounded output and owned cancellation. SemanticDocumentationTests analyzes the
complete handbook/agent examples and checks detached index consistency alongside
their existing compilation/execution assertions. Compiler Analysis tests cover
exact spans, symbol/flow identity and policies for every expression form.

ExistingInputTests checks preparation policy through the production loader,
including missing SDK/metadata and generator/build markers during revalidation.
TextSnapshotTests and DiagnosticFixPlanTests check detached unsaved text, linked
owners, UTF-16 identity, conflicts and explicit byte publication. FileWorkspaceTests
checks shared correction/recheck state, typed rename-plan validation and preserved
root outputs; CLI regression tests exercise the same services through commands.

LiveDocumentTests, LiveWorkspaceTests and LiveGraphTests exercise transactional
UTF-16 changes, versions, save/close authority, unowned/linked files, unsaved
cross-project signatures/rename, current failures, reference invalidation,
independent reuse, limits and cancellation/disposal. PassivePreparationTests
checks imported SDK resolution and rejection of custom preparation before effects.
LiveCancellationTests checks that caller cancellation joins an actual owned
evaluation process before returning on Windows and Linux.
MetadataImageTests exercises actual analysis/emission against private bytes with
no corresponding DLL and detached caller buffers.

PartialAnalysisTests checks detached coverage and independently checked bodies,
incomplete calls/types/declarations, missing closers and rejected emission.
PartialQueryTests checks partial definition/type/refs/Find/call/inspection results,
explicit unavailable regions/dependencies, current repairs and strict rename.
DocumentationCliTests and DiagnosticWorkspaceTests verify that documentation
lookup reports incomplete coverage while preserving compiler diagnostic details.
LspDiagnosticsTests also verifies syntax and independent semantic errors through
the actual stdio server and replacement after unsaved repairs.

LspLifecycleTests and LspDiagnosticsTests use an independent byte-framed stdio
client against the actual CLI, including the installed SDK copy. They check
initialization, UTF-16/versioned edits, save/close/reopen, passive analysis,
watched dependencies, invalid messages, limits and resynchronization.
LspCancellationTests verifies EOF, shutdown and superseding edits against a blocked
owned evaluation process on Windows and Linux. A Windows named pipe or Linux FIFO
blocks the actual evaluator; exit is observed before client disposal.
LspFeatureTests checks standard navigation/search/formatting and capability
negotiation. LspRenameBehaviorTests applies returned edits in the client, rebuilds
and executes programs across overload, capture, keyword and interface renames.
LspScopeTests checks linked ownership and response limits; LspRequestLifecycleTests
checks cancellation, supersession and shutdown of admitted requests on Windows and Linux.
LspCliParityTests compares definition/reference locations, symbol search, rename
edits, formatting and semantic/syntax failures against the CLI using identical
saved UTF-16 snapshots. Partial CLI searches retain coverage; LSP refuses them
because standard result arrays cannot express that coverage.
LocalDeploymentTests copies only the published executable into a fresh directory.
It checks concurrent compilation/export with no compiler checkout or discoverable
.NET host, SDK export conflicts/replacement, content-cache repair, executable SDK
invocation and compiler mismatch rejection. It drives both the delivered launcher
and SDK CLI from a fresh consumer and checks that analysis/rename changes no files.
The same delivered file runs offline batch/DAP debugging. See
[distribution](../docs/build/distribution.md) for the Windows/Linux release contract.

SourceOperationTests verifies body ownership, exact operations, callable
references, dispatch uncertainty, contract phases and protocol exclusions.
CallQueryTests verifies source/DLL identities, anonymous regions, constructor/
accessor selection, incoming scope, availability and paging. QueryCliTests also
executes the CLI extracted from the SDK package. Full Debug and focused Release
acceptance runs on Windows and Linux/WSL.
Both published launchers pass callees/callers against a fresh matching SDK
consumer after dist; an FPAS verifier checks the JSON reports. See the
[deployment workflow](../docs/build/development.md#deploy-the-local-release-compiler).

## Own CoreCLR connection tests

`NativeCoreTests` uses the packaged DbgShim directly.
It compiles real FPAS DLL/PDB/source sets and verifies native source breakpoints,
local reads, stepping, exit, stopped-target cleanup, invalid cwd, a host that
exits before CLR startup, and cancellation with a registered suspended target.
It checks process exit and exclusive DLL/PDB access before fixture cleanup.
The [native reference](../src/Fpas.Debugging/Native/README.md) owns dependency
versions, ABI sources and lifetime rules.

`NativeAssetTests` exercises the extracted SDK CLI against valid, missing,
incompatible and wrong-architecture shim files with a valid external copy on
the native search path. ARM64 packages supply genuine negative inputs only;
the delivered SDK retains its Windows x64 and Linux x64 assets.

`CoreSessionTests` exercises the supervised production host: fresh/stale stop
identities, resume/pause races, startup cancellation, worker crash/hang, native
callback failure, actual exit codes/signals, UTF-8 output bounds and unrelated
process survival. The [session reference](../docs/architecture/debugger-core.md)
owns the implemented supervision contract. These suites are verified in Debug
and Release tool builds on native Windows x64 and WSL2 Linux x64. Debug and fully
optimized target configurations are independent of the tool-build configuration.

`SymbolContextTests` adds recursive and shadowed frame bindings, library frames
with equal source basenames, native path-case rules, source/PDB preflight and
replacement of an already loaded DLL/PDB. Run its filter to check the symbol and
stack interface independently of the later product frontends.

`SourceControlTests` covers the private entry stop, source breakpoint replacement,
duplicates, rejection and module load/unload, changes during execution and native
into/over/out. The fixtures assert recursive frame depths/values, same-line
columns, repeated loop conditions, step-through behavior and interrupted steps.
Foreign bad PDBs and repeated loads of the main assembly have real regressions.

`ExceptionInspectionTests` checks policies, passive stored messages and exception
stacks with explicit clipping; hostile CLR overrides detect accidental execution.
`AsyncControlTests` checks real and already-completed awaits, a concurrent instance,
different CLR threads, stable logical IDs, hoisted int32 values, nested generic
entry, repeated suspension, Task/ValueTask/ConfigureAwait variants and pending-step
pause/stop/fault/cancellation cleanup. Logical step-out covers unique, missing and
multiple callers, duplicate assembly loads, optimized targets and unchanged
event revisions after an operation is rejected before execution.

`ScalarInspectionTests` covers integer widths, floating special values, decimal
scale/range, enum storage, UTF-16/surrogates and bounded binding pages. Fully
optimized first-JIT cases disable tiering in the target configuration, verify
missing/current locals, assert file effects occur after their source stops and
reject missing mappings or an excessive binding prefix without losing the session.

`CompoundInspectionTests` covers passive generic/boxed objects, array ranks and
actual lower bounds, aliases, cycles, expired references and expansion limits.
Sentinel files detect accidental getter, proxy or display execution.
`StaticInspectionTests` checks selected generic/thread contexts, passive reads
of uninitialized types, literal/runtime fields and expired static-value paths.
`CaptureInspectionTests` covers source/raw captures, marked mutable cells,
ordinary/async receivers and lexical shadowing. Compiler metadata tests check
the emitted storage contract independently of the debugger reader.
`SourceProjectionTests` checks record/union/domain source and raw views, decoded
range extrema/defaults and isolation of malformed optional presentation metadata.
`ExpressionInspectionTests` checks passive FPAS operations, exact scalar rules,
short-circuit type checking, stored indexes, AST/read bounds and expired frames.
`ModuleContentIdentityTests` changes referenced constants while preserving the
FPAS source/PDB and checks distinct module IDs, deterministic repeated bytes
and rejection of a real file replacement after preflight.
`RectangularInteropTests` checks canonical metadata and real calls in both C#
interop directions, as well as diagnostics for unsupported signature shapes.

After building, run with Debug or Release on native Windows and native WSL Linux:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~NativeCoreTests|FullyQualifiedName~CoreSessionTests|FullyQualifiedName~SymbolContextTests|FullyQualifiedName~SourceControlTests|FullyQualifiedName~ExceptionInspectionTests|FullyQualifiedName~AsyncControlTests|FullyQualifiedName~ScalarInspectionTests|FullyQualifiedName~CompoundInspectionTests|FullyQualifiedName~StaticInspectionTests|FullyQualifiedName~CaptureInspectionTests|FullyQualifiedName~SourceProjectionTests|FullyQualifiedName~ExpressionInspectionTests|FullyQualifiedName~ModuleContentIdentityTests|FullyQualifiedName~RectangularInteropTests' --logger 'trx;LogFileName=native-core.trx' --results-directory .temp-data/own-debugger/results
```

## Batch and DAP frontends

`TargetIoTests` checks exact argv elements, explicit/default stdin, blocked input,
raw-byte output limits, incremental UTF-8 decoding and owned shutdown.
`BatchDebuggerTests` invokes the real CLI and parses one complete detached report:
source-stop effects, exact scalars, independent errors, missed breakpoints,
exceptions, explicit input, Unicode/quoted paths, deadlines, output and value
budgets. It checks that no session/frame/value handles escape the report.

`DapDebuggerTests` drives product stdio through an independent client. It checks
split framing, configuration order, path/coordinate negotiation, scopes, passive
expressions, pages and stale handles, source stepping, exception information,
malformed/oversized messages, duplicate request sequences and output backpressure.
A suspended real worker proves shutdown completes with empty and full ordinary
request queues. Actual Windows
ConPTY Ctrl+C, native Linux SIGINT and client EOF verify worker/target exit before
harness cleanup. No external debugger or optional environment switch is required.

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~BatchDebuggerTests|FullyQualifiedName~DapDebuggerTests|FullyQualifiedName~TargetIoTests' --logger 'trx;LogFileName=frontends.trx' --results-directory .temp-data/own-debugger/results
```

Run the same suites in Release on native Windows and native WSL Linux. SDK tests
verify both native RID assets and notices at their exact deployed paths and
working packaged help. The [debugging guide](../docs/pascal/tools/debugging.md)
describes batch and DAP usage. `FrontendParityTests` compares both frontends at
the same source stop against independently expected exact values and expression
failures. `LocalDeploymentTests` also exercises both delivered protocols with the
checkout, feed and package cache hidden and invalid project inputs beside the target.

## Documentation examples

DocExamplesFixture builds the FPAS application with a fresh packaged SDK. It checks
complete README, handbook and agent-reference fences and explicit fragment contexts.
LanguageSpecificationTests uses its assemblies, checks PDB/XML outputs and executes
programs with existing behavioral assertions. Companion units use project labels. File/environment/HTTP
inputs are controlled; timer counts and public network availability are not
ordinary test prerequisites.

DocExamplesBehaviorTests deliberately breaks marker syntax, IDs, paths, contexts,
includes, companions, expected diagnostics and runtime expectations. It verifies
documented manifests without source rewriting, empty-selection rejection and
timeout/cancellation cleanup of actual programs and descendants.

AgentReferenceTests checks policy, unique complete examples, local links, and
exact o200k_base token budgets using test-only packaged vocabulary. HandbookTests
checks navigation, repository documentation and hidden skill links/anchors, explicit fragments,
and current copied inputs. Its preparation regression verifies that normal and
design-time metadata preparation retain staged documentation before copying. See
[documentation acceptance](coverage.md#documentation-acceptance) and
[example conventions](../docs/pascal/tools/documentation-examples.md).

## See also

- [Compiler architecture](../docs/architecture/README.md)
- [Build acceptance](coverage.md#build-acceptance)
- [Open language requirements](../docs/future/language-gaps.md)

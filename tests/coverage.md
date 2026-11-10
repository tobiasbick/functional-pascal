# Verification coverage

This map identifies tests to read or select for a behavior. It describes checked
contracts, not support for every .NET signature or deferred language feature.
Commands and configuration selection belong in the
[development workflow](../docs/build/development.md).

## Toolchain environment

EnvironmentCommandTests owns the [environment inspection](../docs/pascal/tools/environment.md)
CLI contract: actual SDK selection, package/cache identity and bounds, unavailable
components, argument JSON, external version probes and owned interruption.

## Measurements and diagnostic artifacts

CLI-facing acceptance is owned by DiagnosticsWorkflowTests (profile/dump
commands and collector cleanup), TracepointCliTests (detached repeated values
and hit limits), TerminalWorkflowTests (real native PTY/ConPTY input, Unicode,
resize, natural exit and descendant reaping), ReportSchemaTests (independent
validation of actual report bytes) and MaintenanceTests (SDK restore/clean).
TestingPlatformTests owns FPAS MTP execution, native JSON identities, TRX,
filter/extension transport and timeout reaping. FrameworkExtensionTests covers
imported custom data providers and actual Code Coverage output;
ExternalAdapterTests verifies an external native adapter against a real FPAS
method, including a no-adapter negative control.

Profiling.DiagnosticsWorkflowTests is the primary owner of the
[measurement](../docs/pascal/tools/benchmarks.md) and
[diagnostic](../docs/pascal/tools/diagnostic-workflows.md) application workflows.
It builds the checked-in FPAS sources through a newly packaged SDK and drives the
manifest-pinned Microsoft tools. Independent arithmetic expectations verify the
CPU/allocation results. Sample counts, fresh-file ownership, rejected child
output and deadline/reaping checks protect the timing boundary and failure path.
Actual nettrace decoding checks GC starts and allocation ticks; topN resolves the
FPAS CPU kernel and counter CSVs contain positive allocation/collection metrics.
Heap dump analysis recovers managed frames and the held exception after target
exit; the original portable-PDB stack includes source lines. Native Windows/WSL
Debug/Release acceptance runs the same contracts without platform skips.

## Native GUI

`GuiHostTests` builds the public FPAS counter, gallery and `Fpas.Ui.Gui` through a fresh SDK.
It checks native activation, disabled buttons, deferred transitions, first-frame
geometry, column stretch/padding, view and subview reuse, resize, detached
snapshots, worker posting, thread ownership, bounded admission, native/application
close and failure cleanup with software and OpenGL rendering.
The gallery contract also checks controlled Unicode edits and rejected proposals,
checkbox/selection/table/tab input, theme/resize state, modal focus restoration,
resource removal, idle completion and window repaint/input while worker bodies
and cancellation callbacks are held at gates. Native close stays responsive
until both gates finish. `CuiHostLayoutTests` covers CUI weighted growth from
zero, frozen bounds, fixed extents, shrinkable content hints, intrinsic canvas
size/pixels and controlled End/wheel input with either scrollbar
setting. `CuiHostResourceTests` checks native texture accounting across removal,
replacement and repeated sessions, and normalized Unicode selection offsets.
`CuiPresentationTests` checks explicit presentation and frame-local control tracking.

`Fpas.Ui.Gui.Cui.Tests` covers native window lifetime, thread/frame ownership,
software/OpenGL control input and capture, and strict UTF-8 buffer boundaries.
The window tests check nested row and column geometry and route text input
between editors using programmatic focus and a non-ASCII control ID.
The text area test checks its requested height and edits a multiline Unicode
buffer before moving focus to a single-line field in the same frame.
`CuiTextEditingTests` checks that reported selections and the next edit respect
grapheme boundaries, and that a replacement document uses the supplied selection.
`CuiContainerTests` checks that a scroll offset moves content within a fixed
viewport and that a modal dialog intercepts outside clicks and reports dismissal.
`CuiImageTests` checks rendered pixels when replacing a queued image in the same
frame. It also verifies window and UI-thread ownership and teardown with a live image.
The texture lifetime case counts actual software-renderer allocations and
releases across replacements, rejected uploads and repeated window teardown.
The selection tests activate list items, table rows and tabs. They compare
captured backgrounds before the click, while the host keeps the old selection
and after the host applies the new selection. The table case also exercises UTF-8
column titles and text cells.
`CuiWakeupTests` waits for a noncontinuous window to go idle, posts a wakeup from
a worker and checks that the window rejects new wakeups after disposal. Native
CTest cases link the control adapters through both library variants and check
line breaks, multiline keyboard navigation and undo isolation across document
replacements. The Win32 case exercises live-resize callback registration and
detachment.
`FpasProjectTests` builds the internal `Fixtures/gui` project with a fresh local
FPAS SDK and the solution-built CLR binding. It verifies project-reference native
asset/notices delivery, execution from another working directory, button and
checkbox interaction, multiline Unicode edits, layout, dialogs, image pixels
and FPAS `using` cleanup on normal and exceptional exits. Both renderers must
actually be selected.
The opt-in publishing case restores the GUI package through a C# project
reference and checks missing or unsupported RID diagnostics. It publishes and
signs all five modes into the same owned directory. It verifies both renderers,
the CUI asset and notice hashes, stale file cleanup and signature rejection
after CUI library or notice tampering. Single-file and Native AOT run with only
the executable present. The case verifies the extracted CUI file for single-file
and checks that Native AOT neither imports nor extracts CUI.

Run the GUI suite under Xvfb on headless Linux; see the
[binding reference](../src/Fpas.Ui.Gui.Cui/README.md#source-build-and-verification).
Window cases are skipped on Linux without `DISPLAY`. Encoding tests still run.

`GuiPublishingTests` packages Fpas.Ui.Mvu, Fpas.Ui and Fpas.Ui.Gui with a fresh SDK and
the matching native binding. A package-only consumer runs the real gallery in
all five delivery modes under software and OpenGL. It checks Unicode selection,
controlled widgets, modal focus, scrolling, theme state, background completion
and cancellation, resource removal and repeated public-session teardown.
NativeAOT/single-file run from an executable-only directory without .NET host
search; the test checks extraction and ELF imports on Linux. Capture checks
cover the public owner-thread PNG operation and preserve the session on failure.

The separate Linux X11 case drives actual X server events through XTest:
pointer/button input, Ctrl+A and typing, native resize, modal Escape and
WM_DELETE_WINDOW close. It checks published model values, selection of the
responsive layout, focus and theme preservation. Other GUI input tests inject
directly through CUI; they are not evidence of OS event delivery. Native texture
accounting and editor/image counts establish owned-resource cleanup; process
working set is not the acceptance criterion. Windows platform evidence remains
in the [GUI plan](../docs/future/gui/verification-and-documentation.md).

## Shared widget contracts

`WidgetContractTests` compiles [Fpas.Ui](../src/Fpas.Ui/README.md) as a
standalone FPAS library and prepares one view under all three profiles.
It checks typed cache reuse, invalidation and removal, and path-qualified
diagnostics for incompatible units or extensions, duplicate IDs and keys,
malformed tables, invalid selections and split graphemes.

## Terminal UI

Both hosts use the same [Fpas.Ui](../src/Fpas.Ui/README.md) tree. GuiHostTests
compiles a portable view with the same model/update callbacks for both hosts
and compares checkbox, editor, selection, tab and modal messages, values,
focus commands and background-result delivery. It also runs the same view and
ThemeFor callback through both hosts, verifies all light/dark accent palettes
by native readback, custom palettes, logical fixed sizing and normalized growth.
The terminal host retains its cell renderer and CUI-compatible allocation.

The terminal CUI fixture checks proportional shrinkage, bounded and fractional
growth, fixed overflow, logical dimensions on each axis, inherited styles and
alpha compositing against the containing surface.

The shared terminal fixture checks disabled collection focus, deferred checkbox
mapping, Space/Enter activation, disabled-row navigation, rejected controlled
edits, host-owned scrollbar options, fresh routing with reused frames and
preflight diagnostics before drawing. TerminalBoundaryTests compiles and runs
the terminal libraries with an unavailable GUI build command, no CUI files,
and checks that their assembly references contain no GUI dependency.

`HeadlessTuiTests` and `TuiGalleryTests` exercise the [TUI host](../src/Fpas.Ui.Tui/README.md), including
frame updates, queue ordering, stale results after replacing an effect, delivery
of a worker failure on a later step, and joining cancelled workers before exit.
The effect fixture drives the same update function in the TUI and directly
through [Fpas.Ui.Mvu](../src/Fpas.Ui.Mvu/README.md). It holds a worker before its first
await and holds cancellation callbacks during replacement and explicit cancel,
then checks that another button can still update the TUI. It also checks
release of a blocked writer and observes callback completion and failure at
shutdown.
It also checks initial and resized viewports in the gallery, canvas size
notifications and the priority of typed characters in a focused editor.
The gallery verifies that an empty text field shows its placeholder after
focus and theme changes and after clearing a pasted value.
The headless fixture checks themed dividers, title clipping at a wide glyph
and initial focus on the adjacent button.
Status line coverage checks formatting from shortcut bindings, complete
hints at narrow widths and the switch to the overlay's Escape hint.
The gallery checks themed button variants, disabled, hovered and focused
appearance, live palette changes and activation of the destructive variant.
The multiline fixture checks grapheme-safe deletion, newline normalization,
caret visibility on resize and navigation without an edit message. The
gallery DAP test restores a document's model-owned selection, follows its caret
after a document switch and pastes several lines through the private terminal.
The gallery DAP test resizes the private terminal across the layout threshold
with a modal overlay open and closes it with Enter through the terminal.
The headless gallery checks the overlay caps, body scrolling and the
footer-only layout on a short terminal. The Mandelbrot DAP test drives
the actual explorer with terminal keyboard and mouse input, then resizes
the terminal and checks the new RGB image.
The widget gallery runs through the headless control pipe and in a private
debug terminal. Its tests check that focus and activation are independent,
that an update can focus a newly displayed control, and that closing the
overlay restores focus after a resize. They also check selection after
changing themes. The tab checks cover keyboard and mouse selection,
disabled and missing keys, narrow terminals, and focus when a panel
is removed. The table checks cover fixed headers, resizing columns,
row activation, and clipping wide glyphs at column edges.
`TerminalModeTests` reads Win32 input and output modes or Linux termios state
before, during and after an interactive run in a private terminal. It checks
exact restoration after both normal exit and a native output failure.

## Language and metadata

ForeignHabitDiagnosticsTests verifies targeted Pascal/Delphi/Ada guidance,
exact operator edits, compile/analysis parity and preservation of valid `Result`
storage and comment-like text in literals. DiagnosticCliTests checks the JSON
guidance and output preservation; CheckFixTests applies the Ada inequality edit
through a fresh SDK consumer and rechecks the corrected project.

Syntax.TokenBoundaryTests varies every declared boundary individually and all
boundaries together across complete programs: empty gaps where permitted,
spaces/tabs, LF/CRLF/CR, Unicode whitespace, inline/standalone/detached comments.
The independently listed language atoms cover generics next to `=`/`=>`, type
suffixes, arrays/indexes, qualified/member dots, numeric ranges, call/formal lists,
escaped names, literals and word/punctuation operators. Checks preserve raw token
spellings, parsing, compilation, analysis and idempotent formatting.
Syntax.AtomicBoundaryTests splits every internal boundary of compound operators,
comment/quote escapes, numeric/character literals, escaped names and keywords;
both compiler and formatter must reject those inputs. Explicit word-separator
cases prevent merging named endings. TokenBoundaryExecutionTests verifies actual
before/after execution, including `notValue` versus `not Value`, adjacent signs
and division followed by a comment (`/ //` versus `///`).

Formatting tests exercise the fixed source style, comment/documentation and
literal preservation, display-width wrapping, parser diagnostics, cancellation,
idempotence and unchanged method bodies. FormatCorpusTests covers every complete
handbook source and copied FPAS fixture, plus actual before/after execution with
mutable captures, exceptions/cleanup, case expressions and numeric literals.
FormatCliTests covers check/list/stdout/JSON, exact-byte UTF-8 publication,
unchanged timestamps/permissions, recursive exclusions, links, argument errors
and parsing all sources before writing. FormatProjectTests verifies evaluated
Compile conditions/exclusions/links and solution selection without building.

Compiler tests bind through FpasCompiler and inspect diagnostics/metadata.
Execution tests invoke the actual CLI and .NET host. Topic-specific MsBuildTests
build mixed FPAS/C# consumers and compare exported contracts with .NET behavior.

| Area | Compiler/metadata tests | Execution and mixed-build tests |
| --- | --- | --- |
| Analysis without output, completion, cancellation and compile diagnostic parity | AnalysisTests, AnalysisDocumentationTests, AnalysisCancellationTests | Existing CLI/execution/SDK regressions exercise shared preparation |
| Structured expectations/fixes/related locations, UTF-16 ends, bounded recovery and absent setup locations | RichDiagnosticsTests | DiagnosticCliTests, DiagnosticWorkspaceTests (including actual MSBuild error events) |
| Detached spans, declaration ownership/source slices, flow/callable/generic identities and expression policies | SemanticSnapshotTests, DeclarationContextTests, SemanticConstructTests, SemanticPolicyTests | SemanticDocumentationTests |
| Evaluated contexts, explicit preparation/existing-input policies, mapped/mixed graphs, generators, stale binaries and snapshots | | AnalysisContextTests, AnalysisPreparationTests, ExistingInputTests, AnalysisCompatibilityTests, AnalysisBoundsTests, SourceQueryTests |
| Immutable unsaved text/version identity, checked edits, shared publication/recheck and typed rename validation | | TextSnapshotTests, DiagnosticFixPlanTests, FileWorkspaceTests, RenameSemanticTests, RenameGraphTests |
| Private reference images, live document versions/revisions, current source dependencies, passive evaluation, stale foreign metadata, reuse and lifecycle | MetadataImageTests | LiveDocumentTests, LiveWorkspaceTests, LiveGraphTests, LiveCancellationTests, PassivePreparationTests |
| Partial semantic coverage, syntax/body recovery, incomplete declarations, conservative dependency boundaries and strict rename | PartialAnalysisTests | PartialQueryTests, QueryCliTests, LspDiagnosticsTests |
| Stdio LSP lifecycle, versioned UTF-16 document diagnostics, watched inputs, message limits and EOF/shutdown cleanup | — | LspLifecycleTests, LspDiagnosticsTests, LspCancellationTests (Windows/Linux owned-process acceptance) |
| LSP navigation, complete bounded searches, checked versioned rename, fixed-style formatting and stale/cancelled requests | — | LspFeatureTests, LspRenameBehaviorTests, LspScopeTests, LspRequestLifecycleTests (Windows/Linux process acceptance) |
| Check/query JSON, source coverage, no execution, preserved outputs and cancellation | | CheckTests, QueryCliTests, AnalysisSignalTests |
| Init preservation/preview, project/solution build and start selection, source/DLL execution, inherited streams, JSON, automatic original-failure reports and process/cancellation ownership | | InitTests, BuildRunTests, SourceRunTests, FailureReportTests, RunOwnershipTests, ProjectSignalTests |
| Literal declaration search/paging and exact bounded inspection with direct types | DeclarationContextTests | SourceQueryTests, QueryCliTests |
| Source-body ownership, checked call/reference sites, dispatch and protocol exclusions | SourceOperationTests, SemanticPolicyTests | CallQueryTests, QueryCliTests, SourceQueryTests |
| Parsing, annotations, locations, return/unused-result rules | DiagnosticsTests, ExpressionDiagnosticsTests, RoutineTests | CompilerCliTests, PrimitiveExecutionTests, ControlFlowExecutionTests |
| Numeric widths, promotions, real division of integrals, checked/unchecked, native/decimal | ScalarDiagnosticsTests, ScalarMetadataTests | NumericExecutionTests, ScalarExecutionTests, MsBuildTests.Scalars |
| Mixed logical operator grouping and evaluation | ExpressionDiagnosticsTests, FormatterTests | ControlFlowExecutionTests, ExpressionInspectionTests |
| Multiline string values, source spans and formatting | DiagnosticsTests, FormatterTests | MultilineStringExecutionTests |
| Lifted nullable operators, explicit null safety and CLR eligibility | NullableOperatorDiagnosticsTests | MsBuildTests.NullableOperators (C# result/exception matrix, numeric widths, domains, generic calls, evaluation order and await), LanguageSpecificationTests (NullableArithmetic) |
| Units, visibility, source/reference conflicts, deterministic ordering | MultiSourceTests, EmissionTests | MultiSourceCliTests, MsBuildTests.MultiSource |
| CLR members, boxing, nullable flow, struct storage, cleanup | InteropDiagnosticsTests, NullableEffectTests | InteropExecutionTests, MsBuildTests.Interop |
| Prefix array types, nullable and jagged shapes, UTF-16, indexers, enumeration | CollectionsDiagnosticsTests | CollectionsExecutionTests, MsBuildTests.Collections/NestedTypes |
| Delegates, shared captures, callbacks, method groups | CallableDiagnosticsTests, CallableMetadataTests | CallableExecutionTests, MsBuildTests.Callables |
| Records, private members by default, defaults/copy helpers, equality, imported metadata | RecordTests, RecordMetadataTests | RecordExecutionTests, MsBuildTests.Records |
| Variants, payload patterns, guards, coverage, imported cases | VariantTests, CaseCoverageTests, VariantMetadataTests | VariantExecutionTests, CaseExecutionTests, MsBuildTests.Variants |
| Generic constraints, closures, type/method parameter identity | GenericRoutineTests, GenericRoutineMetadataTests, GenericTypeTests, GenericTypeMetadataTests | GenericRoutineExecutionTests, GenericTypeExecutionTests, MsBuildTests.GenericRoutines/GenericTypes |
| Nested CLR type paths, enclosing arguments, constraints and identities | | MsBuildTests.NestedTypes |
| Constrained generic operators, checked dispatch, static members/defaults | GenericRoutineTests | MsBuildTests.GenericMath, LanguageSpecificationTests |
| Complete record/interface graph before field and alias constraints | GenericTypeTests, ScalarDiagnosticsTests | MsBuildTests.Baseline |
| Inference, variance, contextual arrays/variants, overload evidence | InferenceTests | InferenceExecutionTests, MsBuildTests.Inference |
| Contracts, ref/out/in, optional/params/named calls | ContractDiagnosticsTests, ParameterDiagnosticsTests | ContractExecutionTests, ParameterExecutionTests, MsBuildTests.ContractsAndParameters/ParameterDiagnostics |
| Source overloads and CLR interface implementation/defaults | OverloadAndInterfaceDiagnosticsTests | OverloadExecutionTests, MsBuildTests.Overloads/Interfaces |
| Source interface declarations, inheritance, constraints, metadata and XML IDs | SourceInterfaceTests | MsBuildTests.SourceInterfaces, LanguageSpecificationTests (InterfaceSources) |
| Async faults, cancellation, references, contexts, disposal | AsyncDiagnosticsTests | AsyncExecutionTests, MsBuildTests.Async |
| Portable PDBs, sequence points, scopes, closures, async metadata | PortableDebugInfoTests, AsyncDebugInfoTests | DebugSymbolsCliTests, MsBuildTests.DebugSymbols |
| Standard debugger attributes, typed views and symbol/JIT policy | DebuggerAttributeTests | DebugSymbolsCliTests, MsBuildTests.DebugSymbols |
| Direct native CoreCLR launch/callbacks, source breakpoint/local/step, early-host failure and registered-start cancellation | PortableDebugInfoTests | NativeCoreTests (Windows/WSL, Debug/Release) |
| Deployed native shim selection, missing/incompatible/ARM64 negative assets and refusal to substitute an external library | — | NativeAssetTests (Windows/WSL, Debug/Release) |
| Supervised debugger session/stop identities, control races, bounded output, native/worker failures, actual exit status and owned cleanup | NativeCoreTests | CoreSessionTests (Windows/WSL, Debug/Release) |
| Own debugger source/PDB and loaded-module identities, native path cases, recursion/shadowing, library frames, paging and stale frames | PortableDebugInfoTests | SymbolContextTests (Windows/WSL, Debug/Release) |
| Own debugger entry/configuration, source breakpoint/module lifecycle and synchronous source stepping | PortableDebugInfoTests, LoopTests | SourceControlTests (Windows/WSL, Debug/Release) |
| Own debugger exception policies, stored messages, throw/current stacks, passive overrides and omission bounds | PortableDebugInfoTests | ExceptionInspectionTests (Windows/WSL, Debug/Release) |
| Own debugger await families, nested/generic entry, repeated suspension, real thread/instance identity, logical callers, optimized mappings and step cleanup | AsyncDebugInfoTests | AsyncControlTests (Windows/WSL, Debug/Release) |
| Own debugger passive scalar/decimal/enum data, UTF-16, paged bindings, fully optimized first-JIT maps and per-breakpoint limits | DebuggerAttributeTests | ScalarInspectionTests (Windows/WSL, Debug/Release) |
| Own debugger raw stored objects, generic identities, boxed values, actual array bounds, cycles and bounded stale references | — | CompoundInspectionTests (Windows/WSL, Debug/Release) |
| Own debugger static fields, generic/thread context, metadata literals and passive uninitialized types | — | StaticInspectionTests (Windows/WSL, Debug/Release) |
| Source capture metadata, marked cells, ordinary/async closures, lexical initialization and shadowing | CaptureStorageMetadataTests | CaptureInspectionTests (Windows/WSL, Debug/Release) |
| Passive record/variant/domain source views, exact decoded/default ranges and independent invalid presentation | — | SourceProjectionTests (Windows/WSL, Debug/Release) |
| Passive FPAS expressions, scalar promotion, stored indexes, short-circuit type checks, resource bounds and expired frames | — | ExpressionInspectionTests (Windows/WSL, Debug/Release) |
| Content-derived module IDs for reference-driven emission, deterministic identical output and real preflight/file replacement | — | ModuleContentIdentityTests (Windows/WSL, Debug/Release) |
| Batch/DAP exact scalar and expression-failure parity at the same source stop | — | FrontendParityTests (Windows/WSL, Debug/Release) |
| Canonical rectangular CLR array signatures and calls in both C# interop directions; rejected noncanonical bounds/sizes | — | RectangularInteropTests (Windows/WSL, Debug/Release) |
| Own CLR source control, values, captures, async, exceptions, batch and DAP | CallableMetadataTests, AsyncDebugInfoTests | NativeCoreTests, CoreSessionTests, AsyncControlTests, BatchDebuggerTests, DapDebuggerTests |
| Comments, XML IDs, public boundaries, warning policy | DocumentationTests | DocumentationCliTests, MsBuildTests.Documentation/DocumentationIds |
| Global fields, visibility, initialization, nullable metadata | GlobalTests | GlobalExecutionTests, MsBuildTests.Globals |
| Local helper placement, recursion, capture initialization, generic/async environments | NestedRoutineTests | NestedRoutineExecutionTests, MsBuildTests.NestedRoutines |
| Typed/repeat loops, extreme endpoints, flow, iteration captures | LoopTests | LoopExecutionTests, MsBuildTests.Loops |
| Event metadata/accessors, storage, delegates, nullable captures | EventTests, EventMetadataTests | MsBuildTests.Events/EventDiagnostics |
| Type tokens, generic identity, initialization effects | TypeOfTests | TypeOfExecutionTests, MsBuildTests.TypeOf |
| Attribute constants, targets, usage, flags | AttributeTests | MsBuildTests.Attributes/AttributeDiagnostics |
| CLR operators, overloads, checked and lifted calls, and unsupported signatures | ComparisonOperatorTests | ComparisonExecutionTests, MsBuildTests.Comparisons/ImportedOperators/SignatureLimits, LanguageSpecificationTests (TimeAndDuration) |

Files live in [compiler tests](Fpas.Compiler.Tests/) and
[integration tests](Fpas.IntegrationTests/). MsBuildTests uses topic-based partial
files; the shared fixture owns packaging and process setup.

Compiler files are grouped in Analysis, Types, Routines, ControlFlow, and Metadata;
integration files in Cli, Execution, Documentation, ApiDocumentation, Debugging,
Processes, Build and Workspaces.
Shared setup lives in each topic's Support directory.

## Behavioral checks

Refactoring compares bindings before/after speculative source edits. Tests reject
valid-but-captured names, group source interface implementations, retain overloads,
preserve unrelated strings/comments, and cover types/fields/parameters, generic
constraints, aliases, unit import prefixes and linked source owners. CLI cases
validate reviewed plans, explicit apply, altered/stale inputs, output preservation
and fresh final checks. Cross-DLL cases rebuild and execute the renamed program.

Editing covers automatic diagnostic edit ownership, original UTF-16 ranges,
overlap/duplicate handling, UTF-8 BOM/line endings, permissions, staging/cleanup,
stale bytes/context and cancellation. CheckFixTests uses fresh packaged SDK
consumers to verify preview/apply/recheck, parser and type hints, review-only
proposals, incomplete contexts and idempotence. WindowsPublicationTests protects
explicit Windows access rules during replacement and rollback. AnalysisSignalTests
interrupts check --fix during owned dependency preparation on Windows and Linux.

ApiDocumentation exercises `fpas doc` through actual SDK consumers and the public
workspace service. C#-generated XML is the oracle for constructors, overloads,
nested generics, parameter modes, arrays, indexers, fields, events and conversions.
Source inspection supplies Markdown; project/reference scopes preserve origins.
Tests cover framework APIs unused by source, XML budgets/identity/entities,
changed XML snapshots, missing text, paging, diagnostics, deadlines and packaged
CLI execution. AnalysisSignalTests verifies console interruption and owned
preparation cleanup on Windows and Linux.

CompilerCliTests and ArgumentParsingTests cover generated help/aliases, version
output, required/allowed/duplicate options, repeatable references, literal source
paths after `--`, stdin, JSON response files, and structured argument errors.
StandaloneSdkGuideTests executes consumers using only the packaged compiler and
its dependencies in Debug and Release.

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

BatchDebuggerTests verifies detached one-shot reports, exact observations, input, output and limits. DapDebuggerTests drives the product adapter through an independent framed client: configuration, control, values, stale handles, cancellation, EOF and backpressure. Required native cases run on Windows and Linux without opt-in backend settings. See [frontend tests](README.md#batch-and-dap-frontends).

FailureReportTests exercises `run --failure-report` through the real CLI: original
exception/local capture without replay, inherited Unicode input and both output
modes, natural crash/nonzero exits, handled exceptions, storage/symbol rejection
before side effects, and deadline cleanup.

Aggregate checks cover constructor/default order, private state, copy helpers,
equality/hash behavior, recursive generic variants, and corrupted versioned
contracts. Variant coverage has an independent truth-table oracle. Scalar-domain
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

CLR operator tests exercise both operand owners, ambiguity, once-only evaluation,
checked and lifted dispatch, and safe nullable flow. Signature-limit builds
check methods, method groups,
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

DocExamplesFixture builds [the FPAS checker](../apps/doc-examples/) with a freshly
packaged SDK and verifies application SDK/compiler identities. The checker
compiles all Pascal fences from the copied [README](../README.md) and
[handbook](../docs/pascal/README.md), including explicit fragment contexts and
Markdown includes. LanguageSpecificationTests uses the resulting PDB/XML/assemblies
for existing metadata and runtime assertions. Units compile independently with
their unit companions; new complete programs require execution assertions.

DocExamplesBehaviorTests verifies the application boundary with valid and broken
markers, fences, IDs, contexts/includes, paths, companions, expected compiler errors
and runtime results. Manifest cases build the unmodified .fpasproj and sources.
Context/companion errors cannot pass as primary example errors. Controlled
nonterminating examples and descendants verify timeout and console-cancellation
cleanup before harness cleanup on Windows and Linux.

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

See also [native test frameworks](#native-test-frameworks) for test-library SDK consumers.

Projects.PublishingTests owns [application publishing](../docs/build/publishing.md).
Native Windows x64 and WSL Linux x64 runs use Debug/Release tool builds and targets
through Native AOT, single-file, self-contained, apphost and managed DLL modes.
Mixed FPAS/C# programs exercise Core metadata, generics, closures, contracts and
async entry points. Independent deliveries execute without source/compiler/feed;
AOT and single-file cases copy only the executable and disable .NET host search.
Solution/standalone selection, no-build reuse, output conflicts, mode changes,
failure preservation and deadline cleanup are checked through real processes.
Signing cases compare public keys with C# metadata, independently verify RSA
signatures, change keys/modes and reject invalid keys. FPAS and C# AOT hosts use
PFX certificates with environment-only passwords; independent CMS checks and the
SDK verification target reject changed output and unexpected certificates.

LocalDeploymentTests runs the platform's actual dist script twice in an isolated
checkout/cache, from a different working directory. It verifies a matching
launcher/SDK through find/inspect, development-version cache refresh, preservation
of other package versions and unrelated deployed files, and first-run bootstrap.
Only the executable is copied into a fresh delivery directory. With the build
checkout hidden and .NET host search disabled, concurrent first starts compile
real sources, return JSON diagnostics and export the matching embedded SDK.
The exported SDK contains a loadable CUI shared library, a static archive and
all six notices for the host RID. Packing for a supported RID whose assets are
absent reports the missing RID and leaves no package.
The same test checks export conflicts, explicit replacement, executable-local
asset-cache hashes, repair, recreation after deletion and an explicit cache override,
SDK invocation through the executable and rejection of different compiler bytes.
The copied launcher also publishes, certificate-signs and executes an external
application through its matching exported SDK, exercising bundled cryptography.
Both delivered LSP entry points negotiate capabilities and perform diagnostics,
definition, search and versioned rename against a fresh external consumer. File
hashes and timestamps prove no implicit source/output writes. LspCliParityTests
compares CLI/LSP results and failures over identical snapshots; environment tests
verify passive server identity reporting. Language-server acceptance covers Debug
and Release on native Windows and WSL Linux, with evaluation exit checked before
harness cleanup. Symbolic-link and Windows ACL cases run on their applicable platform.

LspDocumentationTests compares CLI lookup, real LSP hover, resolved completion and
signature help for the same source comments under LF, CRLF and CR. It checks
selected overloads and declarations after the call site, Markdown headings,
nested lists and indented code, plus literal rendering of
imported XML text without changing the CLI's XML payload.

SDK packaging verifies the own debugger, exact native RID asset paths and licenses,
managed dependencies and working packaged help. Native Windows x64 and WSL2 Linux
x64 acceptance covers Debug and Release tool builds with Debug and optimized
targets, native startup/control/values, process cleanup, batch/DAP parity and
delivered launchers without checkout/feed/cache access. See
[debugger tests](README.md#batch-and-dap-frontends).

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
tasks, records/variants, generic pipelines/trees, and lexical helpers.

Incremental checks edit/add/remove sources and references, change compiler/task
assets/options, and delete/corrupt requested outputs. They verify no-change skips,
source diagnostics, input/output protection, prior-output preservation, and
publication rollback. Documentation cases cover custom paths, XML IDs compared
with C#, cached warnings, comment-only edits, generation toggles, C# sidecar
propagation, and Clean retaining unrelated XML.

MsBuildTests.Acceptance covers parallel configurations and independent projects,
GetTargetFrameworks/GetTargetPath, reference-list invalidation, and assembly/PDB/XML
equivalence after a clean rebuild. Debug/Release and portable/none symbols are
covered. Other frameworks/platforms remain outside this verified scope; see
[language gaps](../docs/future/language-gaps.md). Editor workflows have their own
[VS Code verification](../src/VSCode/verification.md).

## Native test frameworks

Integration Testing/ uses fresh packaged FPAS SDK consumers and the actual VSTest
adapters for xUnit 2, MSTest and NUnit. NativeFrameworkTests runs generated
templates through dotnet test and checks library/runtime/dependency outputs.
TestCliTests executes real FPAS fixtures covering data rows, Task-returning tests,
setup/cleanup, skips, failing assertions, user exceptions and captured output.
xUnit adds MemberData, IAsyncLifetime/IClassFixture and a compiler formatter API
test written in FPAS. Assertions remain the frameworks' own APIs.

TestTemplateTests covers dry-run/idempotence/conflicts, framework arguments,
library source validation and unsupported MTP mode. TestCommandErrors covers
argument/build failures, non-test inputs and explicit reuse of previous artifacts.
TestSolutionTests covers enabled test selection, configuration mappings, explicit
project selection and a mixed FPAS/C# test solution. TestCliTests also verifies
native discovery, filters, runsettings, zero matches and unique result directories.
TrxReportTests protects skips, nested data results and invalid native reports.
TestReportLimitTests checks a real native result above the JSON budget, preserving
the TRX artifact and returning one bounded failure. Native TRX run messages retain
framework warning/error semantics without parsing localized console diagnostics.
TestLifetimeTests verifies deadline cleanup of an actual waiting test host, plus
console interruption on native Windows and SIGINT on Linux.

## Native Windows CLI acceptance

Projects, Processes, Testing, ApiDocumentation, Editing, Refactoring, Formatting
and Diagnostics run against native Windows processes and freshly packaged SDK
consumers. They cover project templates, mixed-solution configuration/start-project
selection, native test frameworks/discovery/TRX/runsettings, source/metadata/XML
documentation, checked corrections and rename families, linked owners and DLL
boundaries, stale plans, read-only files, rollback and process deadlines.

BuildRunTests checks UTF-8 piped input, separate stdout/stderr, working directories,
Unicode/space arguments, empty/quoted arguments, trailing backslashes and exit codes
in inherited and JSON capture modes. WindowsConsoleTests drives an isolated ConPTY
session and verifies actual console handles, Unicode input/output, exit codes and
Ctrl+C while a program waits for console input.

ProjectSignalTests, AnalysisSignalTests and TestLifetimeTests use the same scenarios
on Windows and Linux. Windows uses ConPTY input to deliver Ctrl+C with separate
redirected protocol streams; Linux sends SIGINT to an owned process group. They
require one JSON response, exit 130 and child exit before test-harness cleanup,
including cancellation after correction/rename publication. ConPTY tests require
Windows 10 version 1809 or newer; they do not open a visible console window.

## See also

- [Test organization](README.md)
- [Compiler architecture](../docs/architecture/README.md)
- [CLR mapping](../docs/specs/clr-mapping.md)
- [MSBuild contract](../docs/build/msbuild.md)

# Evaluated project analysis

Fpas.Workspaces supplies project/solution contexts, live documents, checks and
source queries. It references Fpas.Compiler, Fpas.Processes, Fpas.SdkResolver and
the official Microsoft.VisualStudio.SolutionPersistence library. The CLI does not load a
private MSBuild runtime or another type checker. The compiler does not discover
projects or depend on the workspace layer.

Projects.ProjectDriver supplies build invocation, evaluated target/output selection
and executable disambiguation for the [project CLI](../pascal/program-structure/project-commands.md).
It shares official solution mappings and owned dotnet/MSBuild processes with
analysis. Execution does not require a successful FPAS semantic snapshot; a mixed
solution can select a C#/F# host. Build and analysis retain their own output rules.

Testing.TestDriver shares those solution mappings and owned processes. It
evaluates IsTestProject/TargetFramework/TargetPath through MSBuild, discovers
names through native VSTest artifacts and executes installed adapters through
dotnet vstest on the evaluated assembly. TrxReport reads native XML cases,
outcomes and run messages. Running that assembly directly preserves output paths
that depend on the solution context. The CLI owns selection,
deadlines and bounded reporting; it does not implement a test discovery engine,
framework scheduler or assertion runtime. See [testing](../pascal/tools/testing.md).

Documentation.DocumentationQuery reuses Find/Inspect and explicit semantic coverage for source declarations and
reads public CLR metadata with System.Reflection.Metadata. It neither loads
reference code nor binds another language model. Adjacent XML files supply exact
member documentation; their contents/absence join the result snapshot identity
and input verification. Name/type/member selection, limits and presentation are
defined by [API lookup](../pascal/tools/api-documentation.md).

## Context preparation

WorkspaceLoader.LoadAsync accepts ContextOptions for an existing `.fpasproj` or
`.slnx`, configuration and preparation policy. SolutionPersistence reads solution
structure, build inclusion and configuration mappings. Owned dotnet/MSBuild
processes evaluate project references, imported settings and actual compiler
inputs. The supported FPAS baseline is net10.0/AnyCPU with Exe or Library output.
Foreign C#/F# edges may lead to other checked FPAS projects and supply metadata.

The SDK exports a versioned typed JSON manifest through caller-owned scratch
files. Its task uses the host SDK's MSBuild assemblies without loading the
compiler into MSBuild. Missing targets, malformed data, incompatible compiler
hashes and unsupported settings produce setup errors. The CLI and SDK must
contain the same compiler; repack a development SDK after compiler changes.

Cancellation joins owned evaluation processes before returning. Private manifests
are removed after that join. Windows sharing or lock failures during scratch
cleanup are retried for at most five seconds.

FpasPrepareCompilation shares reference resolution and generated-source hooks
between CoreCompile and analysis. Use BeforeTargets=FpasPrepareCompilation or
FpasPrepareCompilationDependsOn. Arbitrary Build/CoreCompile preparation hooks,
ProjectReference AdditionalProperties, removal of solution context globals and
multiple required configurations for one project are diagnosed explicitly.
Configuration/platform mappings and standard property removal are evaluated by
MSBuild, including mixed F# references.

Default preparation restores and builds required dependencies. NoRestore skips
restore only. It does not allow stale dependency metadata. Analysis does not
build the selected root as a substitute for checking, publish its own assembly
or modify its successful-build cache. A checked library may also be built as
another project's dependency; those outputs are declared separately.
Ordinary bin/obj files and MSBuild task execution are preparation side effects.

ContextPreparation.ExistingInputs skips restore, dependency builds and
FpasPrepareCompilation hooks. It resolves references with BuildProjectReferences
disabled and consumes existing generated files already declared as Compile items.
Package SDK resolution uses the installed-only Fpas.SdkResolver inside MSBuild,
with the downloading NuGet SDK resolver disabled. Imported and dynamic SDK
references follow the same rule. Missing restore assets, sources and foreign
dependency metadata require explicit preparation. This library policy
is separate from the CLI's NoRestore flag. Context verification retains the chosen
policy instead of enabling preparation. MSBuild evaluation/reference-resolution
caches may still use obj. A preprocessing-only pass inspects actual imports
before targets run. Custom initial/analysis hooks, overridden SDK targets,
analysis dependency changes and target-added Compile items are rejected.
Declare generated files in evaluated ItemGroups. Build-only preparation hooks
remain skipped. MSBuild property evaluation still assumes trusted project files.

## Input consistency and ownership

WorkspaceContext contains immutable source/reference requests, graph edges,
requested FPAS projects, preparation issues and dependency outputs. Sources are
captured once. Snapshots.WorkspaceTextSnapshot holds one physical UTF-16 text per
path; TextDocumentSnapshot carries an optional document version and a content
identity that preserves individual code units. Linked owners must agree on text
and version. Content hashes cover sources, references, projects/imports,
compiler/task assets and relevant restore/build proofs. Snapshot identity also
includes target and semantic policies.

Before a FPAS DLL is linked to checked source, its hash must match the successful
build fingerprint for that dependency's current inputs. The same fingerprint
calculation is used by compilation and export. Matching names/signatures alone
cannot attach unrelated or stale binaries to source.

Call VerifyInputsAsync before publishing answers. It checks captured content and
reevaluates preparation to catch changed globs, imports and references. Changes
produce inputs_changed; results are not silently retried or combined across
snapshots. WorkspaceAnalysis analyzes dependencies in deterministic order,
collects independent diagnostics and marks blocked projects explicitly.

Each operation owns its processes and scratch directory. Fpas.Processes supplies
Linux sessions/groups and subreaper cleanup, or suspended Windows process
creation and a kill-on-close job. Build workers and compiler servers are disabled
for analysis preparation. Cancellation stops and joins owned processes; retained
stdout/stderr is bounded to 1 MiB and manifests to 16 MiB. Normal build artifacts
are not deleted as scratch. Windows cleanup explicitly terminates the job and
waits for all its processes before releasing handles. Native Windows and
Linux/WSL acceptance covers packaged check/query, cancellation and process cleanup.

BuildProcess.RunTool uses the same bounded capture and process ownership as
dotnet/MSBuild execution for explicit executable probes. The CLI's Toolchain
topic uses it for environment inspection without loading a workspace context.

## Source edits

Editing.TextEditSet checks original-offset edits against immutable text snapshots,
without reading source files. It rejects unknown owners, conflicting/overlapping
edits and mismatched original text, and retains detached original/replacement
text and versions. UTF-8 byte buffers are created independently for explicit disk
publication. DiagnosticFixPlan selects automatic compiler proposals and retains
review-only suggestions without applying them. SourcePublication
shares byte verification, staging, permission preservation and rollback between
corrections and formatting. FileWorkspace owns the file-backed workflow: capture
and analyze, prepare/validate edits, reverify before publication, then reload and
analyze with the original preparation policy. It tracks actual written files,
recheck state and recovery paths; failed or cancelled reanalysis cannot expose
pre-edit diagnostics as current results. CLI handlers own arguments, plan-file
I/O and reports. See
[automatic corrections](../pascal/tools/diagnostics.md#apply-automatic-corrections).

Refactoring.RenameService follows canonical source/DLL identities and recorded
interface implementation relationships. It checks changed source using private
in-memory dependency images, then compares all indexed bindings and interface relations
after mapping source offsets. Prepare consumes captured source text; Validate
checks typed plan identity, context and snapshot and regenerates its semantic
proposal. Neither operation reads a plan file or rereads original sources.
FileWorkspace applies the checked edit set through the common publisher and
final project analysis. See
[renaming](../pascal/tools/refactoring.md) for scope and limits.

## Live workspace

Live.LiveWorkspace serializes Open/Change/Save/Close and watched-input Invalidate
mutations. Each accepted mutation advances its revision, cancels superseded work
and clears Current. Changes use one-based UTF-16 ranges against consecutive text
states; invalid batches are atomic and versions must increase. Save retains the
open buffer; Close drops it so the next analysis reads disk. Unowned open files
receive syntax diagnostics and stay outside semantic ownership. Linked project
owners share one physical text/version while retaining separate compiler facts.

SourceSnapshots preparation resolves existing inputs without restore/build or
generator execution. Restored FPAS projects supply current dependency metadata
through SourceGraphAnalysis, including when their DLLs do not exist. The same
graph service validates rename overlays. Compiler MetadataImage inputs own
immutable bytes under logical reference paths; analysis/emission do not read the
replaced disk DLL. No dependency DLL/PDB is written by this service.

C#/F# project boundaries require existing metadata with matching portable symbols,
verified document checksums and the evaluated source list. Missing, mismatched or
changed foreign inputs make that boundary unavailable until an explicit build.
Ordinary binary/package/framework references are captured by actual content.
Failed analysis remains the current failed result rather than exposing an old
successful semantic snapshot. Current partial snapshots retain only checked
regions. A broken dependency supplies no private image, so its consumers remain
blocked while independent projects retain their current analysis.

SourceGraphAnalysis caches up to 64 detached project results/private images by
source/reference/configuration/policy identity. Changes invalidate dependants
through reference-image hashes; unrelated facts can be reused. Retention accounts
for image bytes, source text and conservative serialized-fact size, with eviction
before exceeding the 128 MiB session budget. Documents are limited to 4 MiB UTF-8
text and total open text to 64 MiB. At most 64 owned operations run or wait, with a
30-second default work deadline. Disposal cancels and joins owned operations.

LiveSnapshot carries the exact revision, document versions and detached results.
Call LiveWorkspace.VerifyAsync before publishing: it rechecks current disk/project
inputs while open text remains authoritative, and rejects foreign/superseded
snapshots. Verification failure invalidates Current. Consumers use common query,
correction, rename and formatting APIs; the live workspace never applies edits.

## Source queries

SourceQuery consumes checked regions of the current WorkspaceAnalysis. Explicit source
positions select compiler facts from [semantic snapshots](compiler.md#semantic-snapshots).
Find accepts a typed name search without a manufactured source position; inspect
reuses definition's canonical selection and exact source/metadata mapping.
Linked files require an owning project when ambiguous. Definition/type return
self-contained structured descriptors; refs searches all analyzed FPAS projects
within the declared context, excluding declarations and generated occurrences.

Every query result carries WorkspaceSemanticCoverage: complete/partial/unavailable
project availability and source-region coverage. Selection in an unavailable
region fails with analysis_unavailable; absence in checked text is not_found.
References, Find and incoming-call counts cover checked facts only; incomplete
coverage remains separate from pagination/truncation. Inspection labels an
unchecked body unavailable rather than bodyless. Rename preparation and overlay
validation require successful complete analysis of the affected context.

Public FPAS uses imported through DLLs map back through validated dependency
outputs and exact CLR export identities. External .NET members retain metadata
definitions without guessed source locations. Interface calls select their
static contract. Implicit bindings such as args/self have types and references
but no fabricated declaration; definition reports not_found for them.

Reference results sort by project, path, offset and role. Output truncation is
independent of successful scope analysis. Binary-only reference paths state the
search boundary. A project context cannot claim callers in unknown consumers,
and C#/F# sources are not part of the FPAS reference index.

Find searches explicit declarations in checked FPAS sources, including private
and local declarations. Owner filtering narrows searched coverage without
weakening complete-context analysis. Literal ordinal name/kind filters precede
deterministic paging. Linked sources remain separate per project; metadata-only
and generated symbols are excluded. Pages are compatible only within the same
snapshot identity.

Inspect slices immutable captured source by compiler-recorded extents. It keeps
declaration, body and attached documentation availability explicit, and adds one
hop of declared surface types through existing canonical identities. Body-local
types and recursive related-member expansion are excluded. Related source limits
and a 64 KiB aggregate UTF-8 text budget omit whole slices with explicit counts;
these bounds do not weaken analysis coverage. Neither operation executes user
code or retains compiler objects. Both verify inputs before publishing.

Callees and callers share one lazily built body/site index and the existing
canonical export mapping. Outgoing lookup uses the selected body; incoming
lookup uses the statically selected target. Linked compilations stay separate,
and generic matches preserve each site's actual substitutions. Constructor and
accessor selections can additionally restrict the exact CLR operation. Explicit
body selection supports anonymous functions and initializer regions without
creating persistent selectors or resolving source names again.

Call pages resolve descriptors only for returned sites. Whole-body/context
summaries keep unresolved dispatch and excluded protocol counts separate from
page truncation. Metadata-only and bodyless outgoing targets have explicit
availability without empty-body claims. Incoming results distinguish invocations
from callable references and never infer runtime delegate/virtual targets.
Source operations, not emitted helpers, define the index on both sides of an
FPAS DLL boundary.

The [CLI reference](../pascal/program-structure/cli.md) owns selectors, JSON
envelopes, exit codes, limits and examples. These services do not implement an
LSP transport; live document ownership belongs to LiveWorkspace.

## Language server adapter

Fpas.LanguageServer hosts stdio LSP over StreamJsonRpc, using OmniSharp's
MIT-licensed protocol models and serializers. The transport enforces byte/frame
budgets; the reader serializes document notifications before accepting the next
message. One coalescing diagnostics worker analyzes and verifies the current
LiveWorkspace generation. A bounded output queue drops obsolete diagnostic
publications and preserves close notifications. Setup failures use protocol
logging rather than fabricated source locations.

Feature requests capture session/workspace revisions on the reader. StreamJsonRpc
dispatch carries that admission into bounded owned operations. Definitions,
references and symbols adapt SourceQuery; prepare/rename use RenameService and
typed edit sets; formatting calls FpasFormatter on open text. Before response
serialization the adapter checks the captured revision again. Semantic replies
also reverify evaluated inputs. Partial/truncated searches fail when the standard
reply cannot represent coverage. Shutdown/EOF cancel and join owned requests.

The CLI only selects the host. Protocol types, zero-based UTF-16 conversion,
capabilities and watched-file registration stay in the adapter; the workspace
and compiler remain independent. Lifecycle, limits and configuration belong in
the [language-server reference](../pascal/tools/language-server.md).

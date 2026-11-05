# Evaluated project analysis

Fpas.Workspaces supplies explicit project/solution contexts to check and source
queries. It references Fpas.Compiler, Fpas.Processes and the official
Microsoft.VisualStudio.SolutionPersistence library. The CLI does not load a
private MSBuild runtime or another type checker. The compiler does not discover
projects or depend on the workspace layer.

Projects.ProjectDriver supplies build invocation, evaluated target/output selection
and executable disambiguation for the [project CLI](../pascal/program-structure/project-commands.md).
It shares official solution mappings and owned dotnet/MSBuild processes with
analysis. Execution does not require a successful FPAS semantic snapshot; a mixed
solution can select a C#/F# host. Build and analysis retain their own output rules.

## Context preparation

WorkspaceLoader.LoadAsync accepts ContextOptions for an existing `.fpasproj` or
`.slnx`, configuration and restore policy. SolutionPersistence reads solution
structure, build inclusion and configuration mappings. Owned dotnet/MSBuild
processes evaluate project references, imported settings and actual compiler
inputs. The supported FPAS baseline is net10.0/AnyCPU with Exe or Library output.
Foreign C#/F# edges may lead to other checked FPAS projects and supply metadata.

The SDK exports a versioned typed JSON manifest through caller-owned scratch
files. Its task uses the host SDK's MSBuild assemblies without loading the
compiler into MSBuild. Missing targets, malformed data, incompatible compiler
hashes and unsupported settings produce setup errors. The CLI and SDK must
contain the same compiler; repack a development SDK after compiler changes.

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

## Input consistency and ownership

WorkspaceContext contains immutable source/reference requests, graph edges,
requested FPAS projects, preparation issues and dependency outputs. Sources are
captured once. Content hashes cover sources, references, projects/imports,
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

## Source queries

SourceQuery consumes a complete successful WorkspaceAnalysis. Explicit source
positions select compiler facts from [semantic snapshots](compiler.md#semantic-snapshots).
Find accepts a typed name search without a manufactured source position; inspect
reuses definition's canonical selection and exact source/metadata mapping.
Linked files require an owning project when ambiguous. Definition/type return
self-contained structured descriptors; refs searches all analyzed FPAS projects
within the declared context, excluding declarations and generated occurrences.

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
editor session, unsaved buffers, a formatter, refactoring, LSP or query MCP tools.

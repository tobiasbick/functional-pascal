# CLR editor gaps

This document records editor capabilities missing from the CLR port. It is a
gap inventory, not an implementation plan. It does not authorize implementation,
define an order of work, select dependencies, or change FPAS syntax.

The comparison uses the Rust toolchain's
[editor integration](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/tools/editor-integration.md).
The current CLR [MSBuild contract](../build/msbuild.md) places editor and language
server integration outside the initial build scope. The CLR branch has no
equivalent editor extension or language server yet.

CLI commands, the formatter engine, checked source-query/refactoring backends,
and debugger protocols are tracked in [CLI gaps](cli-gap.md). This document
records the editor-facing capabilities that depend on those services. TUI work
and VM-specific debugger operations are outside this inventory.

## Existing foundations

The CLR compiler already parses and binds the language, reads actual assembly
metadata, and produces source-located diagnostics. The project SDK resolves
`.fpasproj` dependencies and mixed `.slnx` solutions through MSBuild. Portable
PDBs and optional XML API documentation are available outputs.

These capabilities do not provide an editor session, analysis of unsaved buffers,
LSP requests, project discovery in an editor, or a VS Code extension. The
[compiler API](../../src/Fpas.Compiler/Compilation.cs) provides both compilation
and semantic validation without emission, including a detached navigation index.
Fpas.Workspaces supplies evaluated contexts and definition/type/refs queries.
An interactive editor session and unsaved-buffer analysis remain missing.

The [workspace service](../architecture/workspaces.md) implements the shared
analysis/project-context foundation for check and three CLI queries. Editor
sessions, unsaved-buffer recovery and an LSP remain separate work.

## Language services

All entries below are missing from the CLR editor toolchain.

| Capability present in the Rust editor | CLR editor gap |
| --- | --- |
| Language registration and syntax highlighting | `.fpas` detection, grammar/highlighting for the current CLR syntax, and suitable language configuration. Old case-insensitive keywords and old terminators cannot be assumed. |
| Comments, brackets, indentation, and folding | Editor rules and source-aware ranges consistent with the current grammar. |
| Live diagnostics | Parse/type diagnostics for the current unsaved document and its project context, with correct ranges, severity, and related locations. |
| Document formatting | Whole-document formatting and format-on-save integration using the same formatter as the CLI. The formatter engine itself is a separate gap. |
| Document symbols | Hierarchical declarations and a useful outline for programs, units, routines, records, unions, domains, and members. |
| Hover | Resolved FPAS/CLR signatures, substituted generic types, and declaration documentation. Referenced XML documentation needs a lookup path. |
| Definition and type-definition navigation | Source declaration locations and meaningful external metadata targets. |
| References and highlights | Resolved source uses, workspace reference search, and same-document read/write highlights. |
| Workspace symbol search | Bounded symbol discovery across selected projects, referenced sources, and available metadata. |
| Selection ranges | Syntax-aware expansion of the editor selection. |
| Completion | Context-sensitive, visibility-aware suggestions for source and imported CLR APIs, including aliases, members, generic substitution, and lazy documentation. |
| Signature help | Applicable routine/constructor overloads, active parameters, argument modes, named arguments, and generic substitutions. |
| Auto-import and snippets | Unambiguous import completion and snippets for the current explicit-alias syntax and supported language constructs. |
| Semantic highlighting | Compiler-backed classification with ordinary syntax highlighting as a fallback. |
| Quick fixes | Source edits for supported diagnostics, including eligible import fixes. Old diagnostic codes and fix recipes are not CLR contracts. |
| Checked refactorings | Editor preparation and application of rename, local-value extraction, and import organization, preserving comments and source meaning. Shared analysis and edit validation belong to the common tooling backend. |
| Call hierarchy | Incoming/outgoing direct-call navigation with explicit handling of stale or incomplete analysis and unavailable dynamic targets. |

The compiler can resolve actual .NET APIs. The missing editor work concerns
exposing those facts for the user's current source, not recreating a `Std.*`
wrapper catalog or implementing C# semantics in the extension.

## Documents and project context

The Rust language service uses source snapshots and unsaved-buffer overlays.
The CLR editor toolchain lacks corresponding support for:

- Document open/change/close handling, versions, and consistent coordinate
  conversion between source offsets and LSP UTF-16 positions.
- Analysis that combines the current unsaved document with unsaved dependencies
  without saving files or publishing build artifacts.
- Project and solution discovery using evaluated `.fpasproj`/MSBuild inputs,
  selected configuration/framework, and resolved package/project references.
- Source, project, and reference change tracking that invalidates affected
  results and refreshes navigation and diagnostics.
- Useful results for incomplete source while the user is typing.
- Request cancellation, bounded analysis/results, and rejection or omission of
  results computed for an older document version.
- Shared symbol identities and analysis facts for CLI queries, LSP requests,
  refactorings, and read-only inspection views.

The Rust source-query interface also serves checked `find`, `inspect`,
`definition`, `refs`, `type`, `callers`, and `callees` operations over editor
snapshots. A CLR editor-snapshot service and its transport remain missing. Their
wire schema and implementation remain open.

## Read-only source views

Two Rust editor views have no CLR equivalent yet:

- Public signatures for the current unit, derived from the unsaved source and
  updated with its version. This is a declaration view, not a source interface
  feature or a saved/generated file.
- Declaration inspection containing the selected signature, implementation,
  directly relevant types, and explicit omissions. Source changes invalidate
  an old view; inspection does not save files or invoke a build.

Both depend on compiler-backed analysis and suitable source presentation.
Emitted XML documentation alone does not provide either view.

## Editor workflows and debugging

IDE debugging is complete through the
[Microsoft VS Code route](../pascal/tools/debugging.md). It is not an open editor
gap. The [MCP debugger](../architecture/mcp-debugging.md) serves agents independently.
Automatic project/language services are
separate editor features.

| Rust editor capability | CLR editor gap or boundary |
| --- | --- |
| Project check/build/run/test/format workflows | Extension commands and task integration using the selected CLR toolchain. Their command backends are tracked in CLI gaps. |
| Problems integration | Consistent association of diagnostics with source documents and project context. |
| Testing view | Test discovery, source locations, selection, cancellation, results, and output, once a CLR FPAS test workflow exists. |
| Active project and terminal runs | Explicit project selection/status, application arguments and working directory, terminal launch, and process lifecycle handling. A mixed solution can launch a non-FPAS host. |
| Server restart and diagnostics | Language-server restart, output channels, and actionable toolchain/protocol errors. |

Debugger evaluation follows the selected backend's expression syntax. A complete
Pascal evaluator is outside the current scope.

## Extension and toolchain distribution

The Rust toolchain has a VS Code-compatible extension and a separate stdio
language server. The CLR port lacks their equivalents and their installation
contract. Open points include:

- Selecting a compatible installed CLR FPAS toolchain and locating its language
  server and optional debugger components.
- Separating protocol stdout from logs and reporting launch/version mismatches.
- Adapting the extension's project discovery and commands to `.fpasproj`,
  `.slnx`, and the CLR compiler/SDK layout.
- Packaging the extension independently of a local compiler build, with current
  grammar, snippets, settings, and supported editor workflows.
- Verification of protocol requests, unsaved-document behavior, project refresh,
  cancellation, and extension/toolchain interoperability.

The existing TypeScript/editor code may contain reusable parts. This document
does not decide which components to reuse or which .NET LSP library to adopt.
Ordinary project builds remain usable without an editor extension.

## Keeping the inventory current

Narrow or remove entries when the owning services and editor integration have
relevant verification. Implemented CLI commands, valid PDBs, or successful
MSBuild invocation alone do not close editor-session gaps. Source language and
build baseline requirements remain in their existing specifications.

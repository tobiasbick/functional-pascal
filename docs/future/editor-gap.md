# CLR editor gaps

Open editor work builds on [workspace analysis](../architecture/workspaces.md).
Usable commands belong in the [CLI reference](../pascal/program-structure/cli.md).
The common [formatter](../pascal/tools/fmt-style.md) supplies complete-source
formatting. Open command/refactoring backends remain in [CLI gaps](cli-gap.md).
No language server, extension library or new FPAS syntax is selected here.

## Language services

| Capability | Open work |
| --- | --- |
| Language registration | `.fpas` detection, highlighting and language configuration for current case-sensitive keywords and terminators. |
| Editing rules | Comments, brackets, indentation and syntax-aware folding/selection ranges. |
| Live diagnostics | Current unsaved document/project diagnostics with accurate ranges, severity and related locations. |
| Formatting | Whole-document and format-on-save integration with the common formatter. |
| Document symbols | Hierarchical outlines for programs, units, routines, records, unions, domains and members. |
| Hover | Resolved FPAS/CLR signatures, generic substitutions and source/referenced XML documentation. |
| Navigation | Definition/type-definition targets, references and same-document read/write highlights. |
| Workspace search | Bounded symbol discovery across selected projects, referenced sources and available metadata. |
| Completion | Visibility-aware source/CLR suggestions, aliases, members, generic substitution and lazy documentation. |
| Signature help | Overloads, active parameters, modes, named arguments and substitutions. |
| Auto-import/snippets | Unambiguous aliases/imports and snippets for supported syntax. |
| Semantic highlighting | Compiler-backed classification with syntax highlighting as fallback. |
| Quick fixes | Supported diagnostic edits and eligible import fixes through the common diagnostic model. |
| Refactorings | Checked preparation/application preserving comments and meaning. |
| Call hierarchy | Incoming/outgoing navigation with explicit stale/incomplete results and dynamic-target uncertainty. |

## Documents and project context

- Handle open/change/close, document versions and UTF-16 coordinate conversion.
- Analyze unsaved documents/dependencies without saving them or publishing build
  artifacts; provide useful recovery for incomplete source.
- Discover projects/solutions using evaluated `.fpasproj`/MSBuild inputs,
  configuration/framework and resolved dependencies.
- Track source/project/reference changes and invalidate affected results.
- Support cancellation, bounded requests and rejection of obsolete results.
- Share symbol identities and analysis facts across queries, LSP, refactorings
  and read-only views; define an editor-snapshot service and transport.

Integrate [call-query facts](../pascal/program-structure/cli.md#callers-and-callees)
with editor snapshots and call-hierarchy transport.

## Read-only source views

Provide versioned public-signature and declaration-inspection views from unsaved
source, including implementation, directly relevant types and explicit omissions.
Invalidate stale views. Inspection must not save files or invoke a build.

## Editor workflows

| Capability | Open work |
| --- | --- |
| Project workflows | Check/build/run/test/format commands and tasks using the selected toolchain. |
| Problems | Associate diagnostics with source documents and project context. |
| Testing view | Discovery, locations, selection, cancellation, results and output after a CLR FPAS test workflow exists. |
| Active project/runs | Project selection/status, arguments, working directory, terminal launch and process ownership, including non-FPAS hosts in mixed solutions. |
| Server diagnostics | Restart, output channels and actionable launch/protocol errors. |

## Extension distribution and verification

Select compatible installed compiler/SDK/language-server versions, isolate
protocol stdout from logs and diagnose mismatches. Package grammar, snippets,
settings and supported workflows independently of a local compiler build.
Choose reusable editor components and an LSP implementation only when designing
that work.

Verify requests, unsaved buffers, incomplete-source recovery, project refresh,
cancellation and extension/toolchain interoperability. CLI results alone cannot
prove editor-session behavior. IDE debugger setup belongs in the
[debugging guide](../pascal/tools/debugging.md), outside this inventory.

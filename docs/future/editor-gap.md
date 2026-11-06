# CLR editor gaps

Open editor work builds on [workspace analysis](../architecture/workspaces.md).
Usable commands belong in the [CLI reference](../pascal/program-structure/cli.md).
The common [formatter](../pascal/tools/fmt-style.md) supplies complete-source
formatting. Open command/refactoring backends remain in [CLI gaps](cli-gap.md).
The [stdio language server](../pascal/tools/language-server.md) exposes common
workspace services. Editor extensions and new FPAS syntax are separate work.

## Language services

| Capability | Open work |
| --- | --- |
| Language registration | `.fpas` detection, highlighting and language configuration for current case-sensitive keywords and terminators. |
| Editing rules | Comments, brackets, indentation and syntax-aware folding/selection ranges. |
| Formatting | Editor format-on-save integration with the language server. |
| Document symbols | Hierarchical outlines for programs, units, routines, records, unions, domains and members. |
| Hover | Resolved FPAS/CLR signatures, generic substitutions and source/referenced XML documentation. |
| Navigation | Type-definition targets and same-document read/write highlights. |
| Workspace search | Discovery beyond checked FPAS declarations, including referenced metadata. |
| Completion | Visibility-aware source/CLR suggestions, aliases, members, generic substitution and lazy documentation. |
| Signature help | Overloads, active parameters, modes, named arguments and substitutions. |
| Auto-import/snippets | Unambiguous aliases/imports and snippets for supported syntax. |
| Semantic highlighting | Compiler-backed classification with syntax highlighting as fallback. |
| Quick fixes | Supported diagnostic edits and eligible import fixes through the common diagnostic model. |
| Refactorings | Additional checked refactorings beyond rename. |
| Call hierarchy | Incoming/outgoing navigation with explicit stale/incomplete results and dynamic-target uncertainty. |

## Documents and project context

- Adapt shared analysis facts to editor-specific views and additional services.

Keep [native platform acceptance](lsp-acceptance-plan.md) separate from additional
capabilities, and remove entries here as they are accepted.

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
| Testing view | Connect native FPAS test discovery, locations, selection, cancellation, results and output to the editor's testing UI. |
| Active project/runs | Project selection/status, arguments, working directory, terminal launch and process ownership, including non-FPAS hosts in mixed solutions. |
| Server diagnostics | Restart, output channels and actionable launch/protocol errors. |

## Extension distribution and verification

Select compatible installed compiler/SDK/language-server versions, isolate
protocol stdout from logs and diagnose mismatches. Package grammar, snippets,
settings and supported workflows independently of a local compiler build.
Choose reusable editor components when designing extension work. The shared
server implementation and its transport are owned by the LSP plan.

Verify requests, unsaved buffers, incomplete-source recovery, project refresh,
cancellation and extension/toolchain interoperability. CLI results alone cannot
prove editor-session behavior. IDE debugger setup belongs in the
[debugging guide](../pascal/tools/debugging.md), outside this inventory.

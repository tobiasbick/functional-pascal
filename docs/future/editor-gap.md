# CLR editor gaps

Open editor work builds on [workspace analysis](../architecture/workspaces.md).
Usable commands belong in the [CLI reference](../pascal/program-structure/cli.md).
The common [formatter](../pascal/tools/fmt-style.md) supplies complete-source
formatting. Open command/refactoring backends remain in [CLI gaps](cli-gap.md).
The [stdio language server](../pascal/tools/language-server.md) exposes common
workspace services. Only additional services and editor integration are listed
here. Editor extensions and new FPAS syntax are separate work.

## Priority

Address this inventory last, after the non-editor gap inventories. This includes
additional LSP services such as hover, document symbols, completion and signature
help, editor extensions and editor-specific acceptance.

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

Call hierarchy should adapt the existing
[call-query facts](../pascal/program-structure/cli.md#callers-and-callees) to live
snapshots and its LSP requests.

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

## Diagnostic presentation

Verify actual Visual Studio error-list display, start/end navigation, warning
severity and help expansion on Windows. Check that explanation and related-note
messages do not create extra error-list entries. SDK integration tests verify
the emitted MSBuild events, but cannot establish the IDE's presentation.
The [diagnostic reference](../pascal/tools/diagnostics.md) owns the shared payload
and CLI/MSBuild behavior.

## Extension distribution and verification

Select compatible installed compiler/SDK/language-server versions, isolate
protocol stdout from logs and diagnose mismatches. Package grammar, snippets,
settings and supported workflows independently of a local compiler build.
Choose reusable editor components when designing extension work. The shared
server's current behavior belongs in the
[language-server contract](../pascal/tools/language-server.md).

Verify requests, unsaved buffers, incomplete-source recovery, project refresh,
cancellation and extension/toolchain interoperability. CLI results alone cannot
prove editor-session behavior. IDE debugger setup belongs in the
[debugging guide](../pascal/tools/debugging.md), outside this inventory.

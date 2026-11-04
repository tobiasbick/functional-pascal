# CLR diagnostic gaps

Extend the shared [diagnostic model](../../src/Fpas.Compiler/Compilation.cs),
[CLI renderer](../../src/Fpas.Cli/Compilation/CompilerCommand.cs) and
[MSBuild adapter](../../src/Fpas.Build.Tasks/FpasCompile.cs) for the open work below.
Field names, edit coordinates and schema evolution still need a design.

| Capability | Open work |
| --- | --- |
| Actionable help | Separate explanation, correction and canonical FPAS spelling from the primary message. |
| Expected/found data | Supply structured expected and actual token/type/value data without requiring message parsing. |
| Proposed corrections | Describe fix titles, exact replacement ranges/text and automatic applicability versus suggestions needing review. |
| Rich terminal output | Add numbered source excerpts, range/caret markers and separate help, with Unicode alignment, terminal color and plain redirected output. |
| Foreign-language hints | Cover common Delphi, Free Pascal and Ada spellings with corrections matching the current CLR grammar. |
| Error recovery | Report later independent problems after a compilation failure while limiting cascades. |
| Unavailable positions | Distinguish real source positions from non-source failures instead of synthetic locations. |
| Related locations/notes | Model declaration conflicts, explanatory notes and navigation targets. |

Keep codes and correction advice consistent across CLI text, JSON, build reporting
and future editor services. Decide whether broad code categories need refinement.
Edit application belongs in [CLI gaps](cli-gap.md); editor actions belong in
[editor gaps](editor-gap.md).

## Build and editor presentation

- Forward useful end positions, help/documentation links and related information
  without creating extra error-list entries for explanatory lines.
- Define how richer help reaches CLI, JSON, MSBuild and editor consumers.
- Verify actual Visual Studio error-list display, source navigation and expanded
  help where supported. MSBuild logging alone does not provide quick fixes.

Follow the [MSBuild diagnostic format](https://learn.microsoft.com/en-us/visualstudio/msbuild/msbuild-diagnostic-format-for-tasks)
and [task logging API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.build.utilities.tasklogginghelper.logerror).
Runtime exceptions retain ordinary .NET behavior; a universal FP error wrapper
or VM-specific failure machinery is outside this inventory.

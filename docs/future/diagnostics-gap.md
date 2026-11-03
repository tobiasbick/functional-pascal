# CLR diagnostic gaps

This document records missing diagnostic capabilities in the CLR toolchain. It
is a gap inventory, not an implementation plan. It does not authorize
implementation, select a schema or library, schedule work, or change FPAS syntax.

The comparison uses the Rust toolchain's
[diagnostic contract](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/tools/diagnostics.md)
and [diagnostic model](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/crates/fpas-diagnostics/src/diagnostic.rs).
The current CLR owners are the [diagnostic documentation](../pascal/tools/diagnostics.md),
[compiler API](../../src/Fpas.Compiler/Compilation.cs),
[CLI renderer](../../src/Fpas.Cli/Compilation/CompilerCommand.cs), and
[MSBuild adapter](../../src/Fpas.Build.Tasks/FpasCompile.cs).
Recheck these sources before changing an entry's status.

## Existing CLR support

The CLR compiler already provides:

- Documented `FPxxxx` codes, error/warning severity, a message, and source
  path/line/column/length through `CompilerDiagnostic`.
- A normal CLI diagnostic line in the form
  `path(line,column): error FPnnnn: message`, or its warning equivalent.
- Structured output through `--diagnostics json`, containing compilation
  success, output path, and the diagnostic list.
- Warning suppression and promotion through CLI and SDK settings.
- Structured MSBuild errors/warnings carrying the FP code, message, file, line,
  and column through `LogError` and `LogWarning`.

Own FP codes, JSON diagnostics, and basic MSBuild reporting are therefore
present. They are not open gaps. Some existing messages already explain the
specific source problem; the missing capability is a richer shared model and
consistent presentation of its details.

## Missing diagnostic capabilities

| Capability | Current CLR gap |
| --- | --- |
| Actionable help | No separate help field for an explanation, correction, or canonical FPAS spelling. Such information cannot currently be consumed independently of the primary message. |
| Expected and found values | No structured producer-supplied expectation and actual token/type/value. Agents must currently infer these details from message text when available. |
| Proposed source corrections | No diagnostic-associated fix title, exact replacement ranges/text, or distinction between automatically applicable edits and suggestions needing review. |
| Rich terminal output | No numbered source excerpt, range/caret marker, or separate help presentation comparable to the Rust renderer. Unicode alignment, terminal-only color, and plain redirected output lack an equivalent rendering contract. |
| Foreign-language spelling hints | No equivalent systematic coverage for common Delphi, Free Pascal, or Ada spellings, explaining the supported CLR FPAS form through structured help and expectation fields. Suggestions must follow the current grammar. |
| Error recovery | A `CompilationFailure` currently ends compilation and returns its single diagnostic. Recovery that reports later independent problems while limiting cascaded errors is missing. |
| Unavailable source positions | The current diagnostic location is mandatory, and some non-source errors use synthetic positions. An explicit distinction between a real source position and an unavailable position is missing. |

Rust already distinguished machine-applicable fixes from suggestions needing
review. Equivalent information is missing from the CLR diagnostic model.
Applying edits through a future CLI command belongs to [CLI gaps](cli-gap.md);
offering them in an editor belongs to [editor gaps](editor-gap.md).
This document records the missing diagnostic data those consumers would need.

The existing FP code catalog remains the starting point. Richer diagnostics do
not require reproducing Rust's numeric assignments, old syntax, or VM stages.
Whether broad existing code categories need more precise distinctions remains
an open diagnostic-design question.

## LLM and output consistency

The existing JSON mode is already useful for automation. It lacks the help,
expectation, and fix data described above. A complete machine-readable contract
for those details is missing; an LLM should not need to parse an English error
message or a terminal excerpt to recover them.

The missing common model would let CLI text, JSON, build reporting, and later
editor services describe the same diagnostic without inventing different codes
or correction advice in each consumer. Exact field names, optional values,
coordinate conventions for edits, and schema evolution remain undecided.

Related source locations and explanatory notes would also support declaration
conflicts and editor navigation. They are additional open modeling questions,
not a claim that the Rust diagnostic record already provided those fields.

## MSBuild and Visual Studio compatibility

Microsoft's [diagnostic format](https://learn.microsoft.com/en-us/visualstudio/msbuild/msbuild-diagnostic-format-for-tasks)
explicitly permits application-specific error/warning codes. Codes must be
locale-neutral and contain no spaces. `FPxxxx` satisfies that convention.
The [task logging API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.build.utilities.tasklogginghelper.logerror)
also accepts an error code, message, file, and source coordinates separately.
Using FP codes and helpful messages does not conflict with CLR or MSBuild rules.

The open gaps concern how richer data reaches each consumer:

- The MSBuild adapter currently forwards the primary message and start
  position. It supplies no end position, help keyword, or documentation link.
- Help, notes, and excerpts need a presentation that preserves the normal
  error-list entry and does not count explanatory lines as additional errors.
- Documentation lookup, related-location navigation, and editor code actions
  need their own integration. MSBuild logging alone does not provide quick fixes.
- Actual Visual Studio error-list display, source navigation, and any expanded
  help presentation have not been practically verified here. General FPAS
  project/editor integration remains a separate editor gap.

## Runtime boundary

This inventory concerns compiler and build diagnostics. CLR execution retains
ordinary .NET exception types, messages, inner exceptions, and stack traces.
Replacing framework exceptions with a universal FP error wrapper is not a gap
or a requirement. Rust VM failure capture and VM-specific runtime diagnostic
machinery are outside this comparison.

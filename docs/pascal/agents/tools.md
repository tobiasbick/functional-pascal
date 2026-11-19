# FPAS development and repository workflows

Use the [agent CLI reference](cli.md) for public commands and checked refactorings.
Use the [development workflow](../../build/development.md) to build the compiler,
bootstrap the matching SDK and run the repository tests. Keep scratch sources,
reports and measurements under `.temp-data/`.

## Select the owning reference

| Task | Read |
| --- | --- |
| Change project boundaries, fixtures, examples, or SDK packaging | [Repository layout](../../architecture/project-layout.md) |
| Parse, bind, type-check, or lower a language feature | [Handbook](../README.md#start-here), [grammar](../../specs/syntax.md), and [compiler architecture](../../architecture/README.md) |
| Check explicit sources and references from C# | [FpasCompiler.Analyze](../../architecture/compiler.md#compiler-api-and-output) |
| Change public types, signatures, dispatch, or emitted metadata | [Interop](../interop/README.md) and [CLR mapping](../../specs/clr-mapping.md) |
| Change project reference, restore, or incremental behavior | [MSBuild contract](../../build/msbuild.md) |
| Build, test, or debug from VS Code | [VS Code extension](../../../src/VSCode/README.md#project-test-and-debug-workflows) |
| Preview and apply checked editor changes | [Editor refactorings](../tools/refactoring.md#vs-code-refactorings) |
| Match stepping, locals, closures, or async frames to source | [Debug information](../../architecture/debug-information.md) and [debugging guide](../tools/debugging.md) |
| Choose tests for a behavior | [Test overview](../../../tests/README.md) and [acceptance map](../../../tests/coverage.md) |
| Build graphical or terminal applications | [GUI](../../ui/gui/README.md), [TUI](../../ui/tui/README.md), and [MVU](../../ui/mvu/README.md) |
| Check implementation status or proposals | [Language scope](../getting-started/overview.md#initial-language-scope) and [language gaps](../../future/language-gaps.md) |
| Update the agent reference or verify examples and budgets | [Maintenance](maintenance.md) |

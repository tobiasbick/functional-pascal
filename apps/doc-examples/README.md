# Documentation example checker

This CLR FPAS application compiles Markdown examples with explicit contexts,
checks intentional compiler errors and runs selected programs against declared
output/exception expectations. Its organization follows the FPAS checker on main;
the CLR implementation uses actual .NET APIs and the current compiler CLI.

[The guide](../../docs/pascal/tools/documentation-examples.md) owns arguments,
fence instructions, reports, limits, contexts and companion projects.

## Build and run

Build the Debug compiler and pack its SDK with the
[source quickstart](../../README.md#build-from-source). The application is a
net10.0 FPAS consumer with project references to Fpas.Processes.Workflows and
Fpas.Ui.Gui. Run from the repository root after setting
[FPAS_REFERENCE_DIR](../../docs/build/development.md#direct-compiler-invocation).

```sh
cli=src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll
dotnet "$cli" build apps/doc-examples/DocExamples.fpasproj
dotnet "$cli" run apps/doc-examples/DocExamples.fpasproj --no-build -- --help
dotnet "$cli" run apps/doc-examples/DocExamples.fpasproj --no-build -- \
  --compiler "$cli" --reference-dir "$FPAS_REFERENCE_DIR" \
  --document docs/pascal/language/types/records.md --json
```

The default selection includes the UI handbook and uses the UI and managed
GUI assemblies copied by the build. The native testing guide requires explicit
xUnit references.
Keep `DocExamples.sdk.txt` when copying the build. It identifies the SDK compiler.

## Source layout

- src/model.fpas defines immutable inputs, mappings and reports.
- src/markdown.fpas validates fences and instructions.
- src/preparation.fpas expands explicit contexts and companions with source maps.
- src/diagnostics.fpas maps compiler diagnostics to their actual origins.
- src/validation.fpas performs compilation, manifest and execution checks.
- src/main.fpas owns selection, batching, reporting and cancellation.

Integration tests build a fresh SDK consumer, check all published examples and
exercise malformed inputs and process lifetime. Existing behavioral assertions
use the checker's compiled assemblies instead of compiling them again.

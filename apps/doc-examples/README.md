# Documentation example checker

This CLR FPAS application compiles Markdown examples with explicit contexts,
checks intentional compiler errors and runs selected programs against declared
output/exception expectations. Its organization follows the FPAS checker on main;
the CLR implementation uses actual .NET APIs and the current compiler CLI.

[The guide](../../docs/pascal/tools/documentation-examples.md) owns arguments,
fence instructions, reports, limits, contexts and companion projects.

## Build and run

Prepare the matching local compiler and SDK with the repository's
[development workflow](../../docs/build/development.md). The application is a
normal net10.0 FPAS consumer with a project reference to Fpas.Processes.

```sh
fpas build apps/doc-examples/DocExamples.fpasproj
fpas run apps/doc-examples/DocExamples.fpasproj --no-build -- --help
fpas run apps/doc-examples/DocExamples.fpasproj --no-build -- --document docs/pascal/language/types/records.md --reference-dir "$FPAS_REFERENCE_DIR" --json
```

The full selection also requires explicit xUnit references for the native testing
guide. The build writes DocExamples.sdk.txt beside the application to identify
the compiler from the SDK that built it. Keep this sidecar when copying a build.

## Source layout

- src/model.fpas defines immutable inputs, mappings and reports.
- src/markdown.fpas validates fences and instructions.
- src/preparation.fpas expands explicit contexts and companions with source maps.
- src/processes.fpas owns bounded processes, streams and cancellation.
- src/diagnostics.fpas maps compiler diagnostics to their actual origins.
- src/validation.fpas performs compilation, manifest and execution checks.
- src/main.fpas owns selection, batching, reporting and cancellation.

Integration tests build a fresh SDK consumer, check all published examples and
exercise malformed inputs and process lifetime. Existing behavioral assertions
use the checker's compiled assemblies instead of compiling them again.

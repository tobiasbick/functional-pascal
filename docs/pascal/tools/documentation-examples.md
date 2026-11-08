# Documentation examples

The FPAS application [doc-examples](../../../apps/doc-examples/) checks Pascal
fences in the repository README, handbook and agent reference. Markdown remains
the source; generated files are inspection artifacts. Integration tests build
the application with a fresh packaged SDK and invoke it automatically.

Complete examples are compiled with portable PDB and XML documentation.
LanguageSpecificationTests uses those assemblies for its existing output,
metadata and controlled file/environment/HTTP checks. Fragments compile in
explicit contexts. SemanticDocumentationTests separately verifies detached indexes.

## Invocation and bootstrap

Build the compiler and tests with the [development workflow](../../build/development.md).
Its documentation filter includes DocExamplesBehaviorTests, which exercises the
actual application boundary and deliberately broken examples.

To run the checker directly, follow the [source quickstart](../../../README.md#build-from-source)
to build the Debug compiler and pack its matching SDK. Run the following commands
from the repository root after setting [FPAS_REFERENCE_DIR](../../build/development.md#direct-compiler-invocation).
The full check uses the xUnit core and assert assemblies copied by the integration
test build for the testing guide's xUnit fragment.

```sh
cli=src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll
dotnet "$cli" run apps/doc-examples/DocExamples.fpasproj -- \
  --compiler "$cli" --reference-dir "$FPAS_REFERENCE_DIR" \
  --reference tests/Fpas.IntegrationTests/bin/Debug/net10.0/xunit.core.dll \
  --reference tests/Fpas.IntegrationTests/bin/Debug/net10.0/xunit.assert.dll
dotnet "$cli" run apps/doc-examples/DocExamples.fpasproj -- \
  --compiler "$cli" --reference-dir "$FPAS_REFERENCE_DIR" \
  --document docs/pascal/language/types/records.md --json
```

`--compiler` selects the managed compiler DLL. Without it, the checker looks
for `bin/Fpas.Cli.dll` under `--root`. `--dotnet` selects the .NET host explicitly.
Without `--dotnet`, the checker uses `DOTNET_HOST_PATH` if set. If neither is set,
it looks under `DOTNET_ROOT` and then on `PATH`. `--reference` is repeatable.
`--contexts` overrides the default `apps/doc-examples/contexts` directory. Quote
paths with spaces.

--document is repeatable and replaces the default selection. --root selects the
repository and resolves document/include paths. --output must name a new directory;
default .temp-data/doc-examples/run-ID. Existing output is never overwritten.
--jobs accepts 1..8, default 4. --timeout accepts 1..3600 seconds per child process,
default 10. Each stdout/stderr stream is bounded to 512 KiB. stdin is closed.
Cancellation/deadlines terminate and join owned children.

## Fence instructions

| Instruction | Meaning |
| --- | --- |
| `pascal` | Compile a complete program or unit without executing it. |
| `id=NAME` | Give an example a document-local name for reports and includes. |
| `fragment context=NAME` | Insert source into the named context template. |
| `context=NAME` | Supply a context for a complete source, such as companion units. |
| `error=FPnnnn` | Require compiler rejection with this exact diagnostic code. |
| `project=NAME file=PATH` | Group distinct files from the same document; Pascal companions can use default filenames. |
| `manifest=PATH` | Also build the actual documented .fpasproj through its SDK. |
| `run stdout=PATH` | Compare stdout with a supporting text fence; require exit 0 and empty stderr. |
| `run stdout=empty` | Require exit 0 and no stdout/stderr. |
| `run exception=TYPE` | Require a nonzero exit and the named .NET 10 unhandled CLR exception in stderr. |

Without id, reports use line-N for the opening fence. Explicit names contain only
ASCII letters, digits, hyphens and underscores. Output paths are relative, without
empty, dot or parent components, drive prefixes or backslashes. Unknown/repeated
instructions, duplicate IDs/files, missing contexts and malformed/unclosed fences
fail. Backtick and tilde fences support LF/CRLF/CR input. Use canonical lowercase
pascal tags. No skip instruction or wrapper search exists.

This deterministic fragment compiles and runs:

```pascal id=checked-fragment fragment context=body run stdout=empty
const Answer: int := 6 * 7;
if Answer <> 42 then
  raise new Sys.InvalidOperationException('Unexpected answer');
end if;
```

Intentional compiler rejection is checked independently:

```pascal id=rejected-fragment fragment context=body error=FP3002
const Answer: int := 'forty-two';
```

Every error must have the expected code and map into that example's own source.
Errors in contexts, includes or companions cannot satisfy the expectation.
Unexpected success, another error, timeout or runtime mismatch fails. run requires
successful compilation and exactly one stdout/exception expectation. Units cannot
run. Network, interactive and host-file examples need controlled fixtures; existing
C# tests retain those behavior checks. Execution has ordinary host access; the
checker is not a sandbox.

## Contexts and companion projects

[Templates](../../../apps/doc-examples/contexts/) are NAME.fpas.txt files with
exactly one whole line `// @example`. Imports, declarations and fixture values
make prerequisites explicit; templates never rewrite the excerpt's code.
The body context supplies Sys, Console, IO, Math and Collections aliases.

A whole line `// @include docs/pascal/PAGE.md#ID` inserts another current Pascal
fence. Includes insert raw source without recursively applying its context.
Optional contexts/NAME/*.fpas companions also support includes and must expand
to units. The import recipe reuses the documented Geometry unit this way.
Every generated source line keeps its Markdown or context origin.

Project companions share project=NAME within a document and distinct file paths.
Supporting xml, text and json fences require project and file. A manifest must
exist among those files and select an available SDK; its content is preserved
and built through the selected compiler CLI. Each unit also compiles independently
with its unit companions. Run uses the manifest's actual TargetPath, runtime
configuration and dependencies. Standalone sources execute their directly compiled
assembly. No program is inferred from an empty selection.

## Reports and verification

Generated directories retain sources, manifests, compiler/manifest/target output,
execution results and report.json. Schema 1 records repository, compiler path/hash,
framework/additional references, application-build SDK compiler path/hash, output,
totals and per-example results. Direct compiler bootstrap without an SDK identity
sidecar reports that identity as unavailable. Results distinguish preparation,
compilation, manifest and execution stages, with Markdown locations, IDs,
compile/run status and generated assembly paths.

--json owns stdout with one JSON document, including input/setup failures. Human
failures use stderr and a concise stdout summary. Exit codes are 0 success/help,
1 validation/setup/deadline failure and 130 console cancellation. Empty selections
fail. Regressions cover malformed input, contexts/includes, paths, companions,
expected diagnostics, output/exception mismatches, nontermination and child cleanup
on native Windows and Linux/WSL.

## See also

- [Application and bootstrap](../../../apps/doc-examples/README.md)
- [Agent reference maintenance](../agents/maintenance.md)
- [Acceptance coverage](../../../tests/coverage.md#documentation-acceptance)

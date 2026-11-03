# Tests

Fpas.Compiler.Tests exercises diagnostics, source locations, emitted metadata,
and deterministic assemblies through the compiler API. Fpas.IntegrationTests
invokes the actual compiler CLI and .NET host, including Hello, stdin, CLR calls,
argument/encoding errors, and output preservation. It also packs and tests the
actual FPAS SDK/MSBuild task. Both projects use xUnit.

Build/test commands and focused documentation checks are in the
[development workflow](../docs/build/development.md). Compile the tests before
running them; copied documentation/examples are build inputs.

Directory.Build.targets derives reference inputs from the actual test project
framework references. TestSupport contains shared bounded process and temporary
directory helpers. The Hello source is copied from examples during the build;
Fixtures/msbuild contains Shared.csproj → Game.fpasproj → Host.csproj and a
mixed-language .slnx. Each test copies it to an isolated directory and resolves a
uniquely versioned SDK from a temporary local feed. Tests execute the host, check
transitive project/package assets and public FPAS signatures, and exercise
incremental builds, source diagnostics, source selection, Clean, and Release
Rebuild. The [acceptance map](../docs/implementation/comparisons-and-acceptance.md)
describes the verified net10.0/AnyCPU language/build scope and the remaining gaps.

LanguageSpecificationTests compiles complete handbook programs/units directly
from the copied Markdown, checks PDB/XML outputs, and executes programs with
controlled file, environment, and HTTP failure inputs. Comparison and signature
tests verify real CLR operators and explicit unsupported-feature diagnostics.
The [agent reference](../docs/pascal/agents/README.md) uses the same runner, with
explicit project labels for companion units. AgentReferenceTests checks its
entrypoint policy, complete/unique examples, local links/fragments, and exact
o200k_base token budgets using test-only Microsoft.ML.Tokenizers packages with
the packaged O200kBase vocabulary.
Additional mixed builds check parallel project/configuration isolation, framework/
target-path negotiation, compiler asset/reference-list invalidation, and clean
assembly/PDB/XML equivalence.

Expression/control-flow tests execute arithmetic, widening/casts, checked
overflow, short-circuit and branch side effects, recursion, lexical scopes,
nested loops, and boundary-safe counting loops. Diagnostic cases check typing,
read-only bindings, return paths, and loop exits. The SDK suite builds the exact
Mandelbrot example and compares all 1,920 output pixels with an independent .NET
Complex calculation; C# also consumes exported FPAS bool/double functions.

Multi-source cases verify unit ownership in CLR metadata, deterministic output
under source reordering/renaming, mutually recursive unit calls, visibility and
source/reference conflicts, secondary-file diagnostics and encoding failures,
stdin plus files, and CLI input protection. SDK tests build the Units example
and a multi-unit FPAS library between C# projects, then check secondary edits and
added/removed units through incremental builds. Another FPAS project consumes
that library through its emitted CLR metadata, including string nullability and
transitive runtime assets.

CLR object cases exercise constructors, chained calls, virtual/interface dispatch,
fields/properties, reference conversions, boxing, struct storage/defensive copies,
nullable flow, Nullable<T>, and nullable metadata exported to C#. Resource fixtures
verify disposal order across return/break/continue, failed acquisition, throwing
cleanup, filter ordering, and stack-preserving rethrow. The checked-in StreamReader
project is built and executed with its copied sample file.

# Tests

The repository uses xUnit for compiler and end-to-end verification. See the
[coverage map](coverage.md) to select a topic's diagnostics, metadata, execution,
and mixed-project checks. Restore/build/test commands and focused filters live
in the [development workflow](../docs/build/development.md).

## Projects and fixtures

| Location | Responsibility |
| --- | --- |
| [Fpas.Compiler.Tests](Fpas.Compiler.Tests/) | Compiler API diagnostics, source locations, flow analysis, emitted metadata, and deterministic images |
| [Fpas.IntegrationTests](Fpas.IntegrationTests/) | Actual CLI/host execution, handbook examples, and packaged SDK/mixed FPAS/C# projects |
| [Fixtures/msbuild](Fixtures/msbuild/) | Shared.csproj → Game.fpasproj → Host.csproj and a mixed .slnx solution |
| [TestSupport](TestSupport/) and [Directory.Build.targets](Directory.Build.targets) | Shared bounded-process and temporary-directory helpers plus resolved framework reference inputs |

Build the toolchain and tests before executing them. Do not rebuild toolchain
binaries while tests use them. Markdown and checked-in examples are copied during
the test build; rebuild after changing those inputs. HandbookTests checks that
the copied Markdown tree exactly matches the sources, including deleted pages.

SDK tests package the already built tools with a unique version into an isolated
local feed, copy the fixture into a temporary directory, and build actual
consumers. Those fixture builds are part of the test, not a second toolchain build.
They verify dependency copying, exported signatures, incremental inputs/outputs,
configuration isolation, diagnostics, and cleanup. Do not rely on a cached local
development SDK version.

## Documentation examples

LanguageSpecificationTests compiles complete README, handbook, and agent-reference fences
through the CLI, checks PDB/XML outputs, and executes programs with explicit
behavioral assertions. Companion units use project labels. File/environment/HTTP
inputs are controlled; timer counts and public network availability are not
ordinary test prerequisites.

AgentReferenceTests checks policy, unique complete examples, local links, and
exact o200k_base token budgets using test-only packaged vocabulary. HandbookTests
checks navigation, repository documentation links/anchors, explicit fragments,
and current copied inputs. See
[documentation acceptance](coverage.md#documentation-acceptance) and
[example conventions](../docs/pascal/tools/documentation-examples.md).

## See also

- [Compiler architecture](../docs/architecture/README.md)
- [Build acceptance](coverage.md#build-acceptance)
- [Open language requirements](../docs/future/language-gaps.md)

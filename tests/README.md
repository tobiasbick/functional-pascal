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

Compiler tests use Types, Routines, ControlFlow, and Metadata directories with
matching namespaces. Integration tests use Cli, Execution, Documentation, Build,
and Debugging. Each project's Support directory owns its shared setup; compiler tests
compile through TestCompilation rather than another test class.

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

## NetCoreDbg integration tests

[NetCoreDbgTests](Fpas.IntegrationTests/Debugging/NetCoreDbgTests.cs) drive a real,
unmodified NetCoreDbg process over stdio DAP against newly compiled FPAS/PDB
fixtures. They cover source breakpoints, parameters/locals, next, true async
suspension, handled/user-unhandled exceptions and normal disconnect. Named and
lambda captures in a generic referenced library verify live mutable cells;
another fixture checks shadowed captures with distinct stored names.
Further fixtures check record/union/domain/nullable/array/generic values,
escaped generic async routines across two incomplete awaits, shared sibling
captures, source step-in, hidden compiler temporaries, and hoisted block scopes
with colliding source names.
Launch/lifecycle fixtures also verify pending-to-bound breakpoints without an
entry stop, relative CLI sources with a separate runtime cwd, UTF-8 arguments,
categorized stdout/stderr, source step-out, pause of running user code and a
failed launch followed by disconnect.

Set `FPAS_NETCOREDBG_PATH` to the executable from the tested NetCoreDbg release
3.2.0-1092. No debugger is downloaded by the tests. Without the variable, xUnit
reports these external-backend tests as skipped, not passed; an invalid supplied
path or failed protocol assertion fails the test. Both platform acceptances need
an actual configured-backend run.

After building the tests:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~NetCoreDbgTests' --logger 'console;verbosity=normal' --logger 'trx;LogFileName=netcoredbg.trx' --results-directory .temp-data/debugger/test-results
```

Optionally set `FPAS_DEBUGGER_TEST_OUTPUT` to an absolute directory under
`.temp-data/debugger/` to retain isolated fixture sources, assemblies, PDBs and
DAP traces. Otherwise fixtures are deleted. The serial test client has bounded
messages/output and a 45-second scenario deadline. It tracks the launch-owned
target using the DAP process event and verifies normal disconnect/exit. It does
not establish crash/hang recovery for an interactive product session.

These are regression tests, not a product frontend or batch debugger. Batch
debugging remains blocked in the [debugger plan](../docs/future/debugger/README.md).

## Documentation examples

LanguageSpecificationTests compiles complete README, handbook, and agent-reference fences
through the CLI, checks PDB/XML outputs, and executes programs with explicit
behavioral assertions. Companion units use project labels. File/environment/HTTP
inputs are controlled; timer counts and public network availability are not
ordinary test prerequisites.

AgentReferenceTests checks policy, unique complete examples, local links, and
exact o200k_base token budgets using test-only packaged vocabulary. HandbookTests
checks navigation, repository documentation and hidden skill links/anchors, explicit fragments,
and current copied inputs. See
[documentation acceptance](coverage.md#documentation-acceptance) and
[example conventions](../docs/pascal/tools/documentation-examples.md).

## See also

- [Compiler architecture](../docs/architecture/README.md)
- [Build acceptance](coverage.md#build-acceptance)
- [Open language requirements](../docs/future/language-gaps.md)

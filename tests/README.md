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

Compiler tests use Analysis, Types, Routines, ControlFlow, and Metadata directories with
matching namespaces. Integration tests use Cli, Execution, Documentation, Build,
Debugging, Processes, Projects and Workspaces. Each topic's Support directory owns its shared setup; compiler tests
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

Workspaces tests exercise evaluated project/solution contexts, check/query JSON,
exact definitions/types/references, name-search paging, bounded declaration
inspection, linked-source ownership, stale metadata,
bounded output and owned cancellation. SemanticDocumentationTests analyzes the
complete handbook/agent examples and checks detached index consistency alongside
their existing compilation/execution assertions. Compiler Analysis tests cover
exact spans, symbol/flow identity and policies for every expression form.

SourceOperationTests verifies body ownership, exact operations, callable
references, dispatch uncertainty, contract phases and protocol exclusions.
CallQueryTests verifies source/DLL identities, anonymous regions, constructor/
accessor selection, incoming scope, availability and paging. QueryCliTests also
executes the CLI extracted from the SDK package. Full Debug and focused Release
acceptance runs on Windows and Linux/WSL, with actual NetCoreDbg for Debug.
Both published launchers pass callees/callers against a fresh matching SDK
consumer after dist; an FPAS verifier checks the JSON reports. See the
[deployment workflow](../docs/build/development.md#deploy-the-local-release-compiler).

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

These are regression tests, not a product frontend or batch debugger.

## MCP agent debugger tests

`McpDebugTests` uses the official MCP client over real pipes to exercise
`fpas debug --netcoredbg <path>` against the same external backend. Set
`FPAS_NETCOREDBG_PATH` as above. It covers discovery/schemas, source controls,
all step kinds, live captures/generics/async values, evaluation, exceptions,
paging, stale/invalid inputs, failed launch/relaunch and concurrent status/control.
An observed server verifies actual stdin EOF, clean server exit and target removal.
An additional backend test verifies native target ownership before killing
NetCoreDbg alone, then checks the failed state and orphan removal on Windows and Linux.

`McpInfrastructureTests` verifies UTF-8 output retention/cursors without an external
debugger. Linux-only disposable adapter fixtures verify malformed frames,
deadlines and concurrent cleanup of pending requests. `OwnedProcessTests` uses
an FPAS parent/child fixture on both platforms to verify cleanup after early
parent exit, Unicode/quoted arguments and environment overrides. Windows tests
verify that job termination joins every process; temporary-directory cleanup
allows a bounded delay for image-file locks after process exit. SDK packaging
tests verify managed dependencies without bundling NetCoreDbg. Native Windows
Codex CLI [agent-host acceptance](../docs/pascal/tools/debugging.md#agent-host-acceptance)
also verifies actual agent tool calls through the deployed launcher.

`McpContractTests` checks live discovery, parameter constraints, effect hints,
initialization instructions and machine-readable text fallback without launching
a backend. A live-target regression checks success fallback as well.
[McpEvaluationTests](Fpas.IntegrationTests/Debugging/McpEvaluationTests.cs) verifies
the ten fixed [inspection questions](Fixtures/debugging/README.md) through paged
MCP values, two source frames and categorized target output.

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~McpDebugTests|FullyQualifiedName~McpInfrastructureTests' --logger 'trx;LogFileName=mcp.trx' --results-directory .temp-data/debugger/test-results
```

## Documentation examples

LanguageSpecificationTests compiles complete README, handbook, and agent-reference fences
through the CLI, checks PDB/XML outputs, and executes programs with explicit
behavioral assertions. Companion units use project labels. File/environment/HTTP
inputs are controlled; timer counts and public network availability are not
ordinary test prerequisites.

AgentReferenceTests checks policy, unique complete examples, local links, and
exact o200k_base token budgets using test-only packaged vocabulary. HandbookTests
checks navigation, repository documentation and hidden skill links/anchors, explicit fragments,
and current copied inputs. Its preparation regression verifies that normal and
design-time metadata preparation retain staged documentation before copying. See
[documentation acceptance](coverage.md#documentation-acceptance) and
[example conventions](../docs/pascal/tools/documentation-examples.md).

## See also

- [Compiler architecture](../docs/architecture/README.md)
- [Build acceptance](coverage.md#build-acceptance)
- [Open language requirements](../docs/future/language-gaps.md)

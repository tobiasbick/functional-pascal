# Native .NET tests

FPAS tests use existing .NET framework packages, attributes and assertions.
VSTest and the framework's adapter discover and execute the emitted CLR methods.
No FPAS naming convention, assertion wrapper or test-specific syntax is needed.

## Create a test project

```sh
fpas init test Tests --framework xunit --sdk-feed sdk/feed
fpas test Tests/Tests.fpasproj
dotnet test Tests/Tests.fpasproj
```

`--framework` selects `xunit` (the default), `mstest` or `nunit`. SDK version/feed,
dry-run and conflict handling follow the [project commands](../program-structure/project-commands.md).
The template selects net10.0, IsTestProject=true, IsPackable=false and explicit
NuGet package versions. It disables inherited central package management for this
self-contained project; use your own central versions when managing dependencies
centrally. Restore needs the FPAS SDK feed and the framework packages.

Test projects contain units. The FPAS SDK compiles them as libraries, disables
generated source entry points and requests ordinary deps/runtimeconfig files.
Do not add a dummy program. Public routines become static methods on the unit's
CLR container. Public records provide instance methods and class-level attributes.
Framework accessibility, constructors and lifecycle rules still apply.

## Write tests

In an xUnit unit, import `Xunit as X` and declare public routines:

```pascal fragment
[X.FactAttribute]
public procedure Addition();
begin
  X.Assert.Equal<int>(4, 2 + 2);
end procedure;

[X.TheoryAttribute]
[X.InlineDataAttribute(1, 2, 3)]
[X.InlineDataAttribute(2, 3, 5)]
public procedure Add(Left: int; Right: int; Expected: int);
begin
  X.Assert.Equal<int>(Expected, Left + Right);
end procedure;
```

Use complete CLR attribute names, including Attribute. For an ignored xUnit test,
write `[X.FactAttribute(Skip := 'reason')]`. Imported attribute constructors and
named properties follow the normal [attribute rules](../interop/attributes.md).
Defining custom attribute classes is outside the language's supported class model.

MSTest uses a public record with `[T.TestClassAttribute]`, where T aliases
Microsoft.VisualStudio.TestTools.UnitTesting. Instance methods carry
`[T.TestMethodAttribute]`; data rows add `[T.DataRowAttribute(...)]`.
Assert with `T.Assert.AreEqual<int>(Expected, Actual)`.

NUnit uses a public record with `[N.TestFixtureAttribute]`, where N aliases
NUnit.Framework. Instance methods carry `[N.TestAttribute]` or repeated
`[N.TestCaseAttribute(...)]`. The template imports NUnit.Framework.Legacy as L
and calls `L.ClassicAssert.AreEqual(Expected, Actual)`.

| Framework | Verified APIs | Template packages |
| --- | --- | --- |
| xUnit 2 | Fact, Theory/InlineData/MemberData, Skip, Task tests, IAsyncLifetime, IClassFixture constructor injection | xunit 2.9.3; xunit.runner.visualstudio 3.1.4 |
| MSTest | TestClass/TestMethod, DataRow, Ignore, Task tests, TestInitialize/TestCleanup | MSTest.TestFramework/TestAdapter 4.5.2 |
| NUnit | TestFixture/Test/TestCase, Ignore, Task tests, SetUp/TearDown | NUnit 5.0.0; NUnit3TestAdapter 6.3.1 |

All templates use Microsoft.NET.Test.Sdk 17.14.1. Assertions, exceptions, data
expansion, output capture, setup/cleanup and skips belong to the framework.
For async tests, return System.Threading.Tasks.Task from an async function.
xUnit fixtures use public record constructors: record fields become constructor
parameters. Data providers return the framework's actual enumerable types.
This does not imply support for every framework extension or C# class hierarchy.
Microsoft.Testing.Platform self-hosted applications are rejected by this VSTest
baseline; they require a separate runner integration.

## Discover and run

```sh
fpas test Tests/Tests.fpasproj --list --report json
fpas test Demo.slnx --filter 'FullyQualifiedName~Addition' --report json
fpas test Demo.slnx --project Tests/Tests.fpasproj --configuration Release
fpas test Tests/Tests.fpasproj --no-build --settings tests.runsettings
```

Pass an explicit `.fpasproj`, `.csproj`, `.fsproj` or `.slnx` path. Paths resolve
from the invocation directory. Solutions select enabled projects whose evaluated
IsTestProject is true and honor configuration/platform mappings. `--project`
narrows execution to an included test project; the context is built as a whole.
A context with no selected test project is an error. Each project needs one
explicit TargetFramework. Configuration defaults to Debug.

Build/restore runs first. `--no-restore` skips restore only; `--no-build` explicitly
uses existing outputs and implies no restore. A failed build never runs stale
tests. `--filter` passes a native VSTest expression. `--settings` selects an
existing .runsettings file, including native parallelism settings.

`--list` discovers without executing test bodies and prints fully qualified
method identities, deduplicated per project. Frameworks may expand/name data rows
differently during execution. Discovery uses VSTest's native name artifact;
execution uses native TRX results. Neither parses localized console output.
A native run selecting no tests can succeed with an explicit total of zero.
Framework warnings remain warnings: xUnit's adapter can report an invalid filter
as a warning with zero tests. JSON preserves native run messages; inspect them
when no tests match. FPAS does not reinterpret localized adapter messages.

## Results and process ownership

The whole operation has a 120-second deadline; `--timeout` accepts 1..3600 seconds.
Cancellation and deadlines terminate and join owned build/runner/test-host
processes. Frameworks own test scheduling; projects run sequentially.

Each invocation creates a unique `TestResults/fpas-<id>/` under the context's
parent directory, with numbered per-project folders. `--results-directory`
changes the parent. Run artifacts contain results.trx; list artifacts contain
discovery.txt. Earlier runs are preserved. Normal build outputs stay in bin/obj.

Human mode sends build diagnostics to stderr and native runner output to stdout.
List mode prints only discovered identities to stdout. `--report json` owns
stdout with one document, including failures. It reports success/status/mode,
context, resultsDirectory, build, per-project runner diagnostics, discovery or
TRX counts/cases/run messages, and an error when applicable. Cases preserve native outcomes,
duration, captured output, assertion/exception messages and stacks. Skipped counts
come from leaf NotExecuted results because legacy TRX summary fields may stay zero.

Owned stdout/stderr capture shares a 1 MiB budget per process. Discovery artifacts
are limited to 4 MiB, TRX XML to 16 MiB characters and JSON reports to 4 MiB UTF-8.
An exceeded limit, missing report or malformed report fails explicitly. An oversized
JSON response directs the caller to native artifacts instead of reporting success.

| Exit | Meaning |
| --- | --- |
| 0 | Successful run/list, including a valid zero-test selection |
| 1 | Test, build, runner, artifact, output-limit or deadline failure |
| 2 | Invalid arguments, non-test selection or unsupported runner mode |
| 130 | User cancellation after owned-process cleanup |

Native Windows process/console acceptance remains tracked in [CLI gaps](../../future/cli-gap.md).

## See also

- [Projects](../program-structure/projects.md)
- [Attributes](../interop/attributes.md)
- [MSBuild contract](../../build/msbuild.md)
- [Test coverage](../../../tests/coverage.md#native-test-frameworks)

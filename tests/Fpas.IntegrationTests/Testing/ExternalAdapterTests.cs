using System.Text.Json;
using System.Xml.Linq;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Testing;

public sealed class ExternalAdapterTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task An_explicit_external_adapter_discovers_and_executes_an_actual_FPAs_method()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk, "Tests", "library");
        var xml = XDocument.Load(project); xml.Root!.Element("PropertyGroup")!.Add(new XElement("IsTestProject", "true"));
        xml.Root.Add(new XElement("ItemGroup", new XElement("PackageReference", new XAttribute("Include", "Microsoft.NET.Test.Sdk"), new XAttribute("Version", "17.14.1")))); xml.Save(project);
        File.WriteAllText(directory.File("Tests/Tests.fpas"), "unit ExternalTests; public function Answer(): int; begin return 42; end function; end unit;");
        Directory.CreateDirectory(directory.File("Adapter"));
        File.WriteAllText(directory.File("Adapter/Adapter.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>netstandard2.0</TargetFramework><LangVersion>latest</LangVersion><Nullable>enable</Nullable><AssemblyName>External.TestAdapter</AssemblyName></PropertyGroup>
              <ItemGroup><PackageReference Include="Microsoft.TestPlatform.ObjectModel" Version="17.14.1" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Adapter/Adapter.cs"), Adapter);
        var configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        foreach (var input in new[] { project, directory.File("Adapter/Adapter.csproj") })
        {
            var build = await ProjectCliFixture.Cli(directory, "build", input, "--configuration", configuration, "--report", "json");
            Assert.True(build.ExitCode == 0, build.Output + build.Error);
        }
        var options = new[] { "test", project, "--configuration", configuration, "--no-build", "--report", "json" };
        var absent = await ProjectCliFixture.Cli(directory, [.. options, "--", "RunConfiguration.TreatNoTestsAsError=true"]);
        Assert.Equal(1, absent.ExitCode);
        var run = await ProjectCliFixture.Cli(directory, [.. options, "--adapter-path", directory.File($"Adapter/bin/{configuration}/netstandard2.0"), "--", "RunConfiguration.TreatNoTestsAsError=true"]);
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using var report = JsonDocument.Parse(run.Output);
        Assert.Equal(1, report.RootElement.GetProperty("projects")[0].GetProperty("result").GetProperty("counts").GetProperty("passed").GetInt32());
        var discovered = await ProjectCliFixture.Cli(directory, [.. options, "--list", "--adapter-path", directory.File($"Adapter/bin/{configuration}/netstandard2.0")]);
        Assert.True(discovered.ExitCode == 0, discovered.Output + discovered.Error);
        using (var list = JsonDocument.Parse(discovered.Output)) Assert.Equal("ExternalTests.Answer", Assert.Single(list.RootElement.GetProperty("projects")[0].GetProperty("discovery").EnumerateArray()).GetProperty("fullyQualifiedName").GetString());
        var core = directory.File($"Tests/bin/{configuration}/net10.0/Fpas.Core.dll"); var previous = File.ReadAllBytes(core);
        File.WriteAllText(directory.File($"Adapter/bin/{configuration}/netstandard2.0/Fpas.Core.dll"), "conflicting dependency");
        var conflict = await ProjectCliFixture.Cli(directory, [.. options, "--adapter-path", directory.File($"Adapter/bin/{configuration}/netstandard2.0")]);
        Assert.Equal(1, conflict.ExitCode); Assert.Equal(previous, File.ReadAllBytes(core));
        using var rejected = JsonDocument.Parse(conflict.Output); Assert.Equal("extension_conflict", rejected.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    private const string Adapter = """
        using System;
        using System.Collections.Generic;
        using System.Reflection;
        using Microsoft.VisualStudio.TestPlatform.ObjectModel;
        using Microsoft.VisualStudio.TestPlatform.ObjectModel.Adapter;
        using Microsoft.VisualStudio.TestPlatform.ObjectModel.Logging;
        [FileExtension(".dll"), DefaultExecutorUri(Executor.Address)]
        public sealed class Discoverer : ITestDiscoverer
        {
            public void DiscoverTests(IEnumerable<string> sources, IDiscoveryContext context, IMessageLogger logger, ITestCaseDiscoverySink sink)
            { foreach (var source in sources) sink.SendTestCase(new TestCase("ExternalTests.Answer", new Uri(Executor.Address), source)); }
        }
        [ExtensionUri(Address)]
        public sealed class Executor : ITestExecutor
        {
            public const string Address = "executor://fpas-extension/v1";
            public void Cancel() { }
            public void RunTests(IEnumerable<string>? sources, IRunContext? context, IFrameworkHandle? handle)
            {
                var tests = new List<TestCase>();
                foreach (var source in sources ?? Array.Empty<string>()) tests.Add(new TestCase("ExternalTests.Answer", new Uri(Address), source));
                RunTests(tests, context, handle);
            }
            public void RunTests(IEnumerable<TestCase>? tests, IRunContext? context, IFrameworkHandle? handle)
            {
                if (handle is null) return;
                foreach (var test in tests ?? Array.Empty<TestCase>())
                {
                    handle.RecordStart(test); var result = new TestResult(test);
                    try
                    {
                        var method = Assembly.LoadFrom(test.Source).GetType("ExternalTests")!.GetMethod("Answer")!;
                        result.Outcome = (int)method.Invoke(null, null)! == 42 ? TestOutcome.Passed : TestOutcome.Failed;
                    }
                    catch (Exception error) { result.Outcome = TestOutcome.Failed; result.ErrorMessage = error.ToString(); }
                    handle.RecordResult(result); handle.RecordEnd(test, result.Outcome);
                }
            }
        }
        """;
}

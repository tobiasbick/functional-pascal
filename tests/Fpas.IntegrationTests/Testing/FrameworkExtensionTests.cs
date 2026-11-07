using System.Text.Json;
using System.Xml.Linq;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Testing;

public sealed class FrameworkExtensionTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Imported_custom_framework_data_sources_and_native_runsettings_execute_real_test_cases()
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "mstest");
        Directory.CreateDirectory(directory.File("Provider"));
        File.WriteAllText(directory.File("Provider/Provider.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable></PropertyGroup>
              <ItemGroup><PackageReference Include="MSTest.TestFramework" Version="4.5.2" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Provider/Cases.cs"), """
            using System;
            using System.Collections.Generic;
            using System.Reflection;
            using Microsoft.VisualStudio.TestTools.UnitTesting;
            namespace External;
            [AttributeUsage(AttributeTargets.Method)]
            public sealed class CasesAttribute : Attribute, ITestDataSource
            {
                public IEnumerable<object?[]> GetData(MethodInfo methodInfo) => new[] { new object?[] { 2, 3, 5 }, new object?[] { 7, 11, 18 } };
                public string? GetDisplayName(MethodInfo methodInfo, object?[]? data) => methodInfo.Name + "-external-" + data?[0];
            }
            """);
        var xml = XDocument.Load(project); xml.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Provider/Provider.csproj")))); xml.Save(project);
        File.WriteAllText(directory.File("Tests/Tests.fpas"), """
            unit FrameworkExtension;
            uses Microsoft.VisualStudio.TestTools.UnitTesting as T, External as Data;
            [T.TestClassAttribute]
            public type Tests = record
              [T.TestMethodAttribute]
              [Data.CasesAttribute]
              procedure Addition(Left: int; Right: int; Expected: int);
              begin T.Assert.AreEqual<int>(Expected, Left + Right); end procedure;
            end record;
            end unit;
            """);
        var config = TestEnvironment.Metadata("FpasBuildConfiguration");
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--configuration", config, "--report", "json");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var run = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--collect", "Code Coverage",
            "--report", "json", "--", "RunConfiguration.TreatNoTestsAsError=true");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using (var report = JsonDocument.Parse(run.Output))
        {
            Assert.Equal(2, report.RootElement.GetProperty("projects")[0].GetProperty("result").GetProperty("counts").GetProperty("passed").GetInt32());
            var results = report.RootElement.GetProperty("resultsDirectory").GetString()!;
            Assert.Contains(Directory.GetFiles(results, "*.coverage", SearchOption.AllDirectories), file => new FileInfo(file).Length > 0);
        }
        var empty = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--filter", "FullyQualifiedName~Absent", "--report", "json", "--", "RunConfiguration.TreatNoTestsAsError=true");
        Assert.Equal(1, empty.ExitCode);
    }
}

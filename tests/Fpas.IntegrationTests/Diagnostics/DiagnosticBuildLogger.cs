using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Diagnostics;

internal static class DiagnosticBuildLogger
{
    internal static async Task<string> Prepare(TemporaryDirectory project)
    {
        Directory.CreateDirectory(project.File("DiagnosticLogger"));
        File.WriteAllText(project.File("DiagnosticLogger/DiagnosticLogger.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings></PropertyGroup>
              <ItemGroup><Reference Include="Microsoft.Build.Framework" HintPath="$(MSBuildBinPath)/Microsoft.Build.Framework.dll" Private="false" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("DiagnosticLogger/Logger.cs"), """
            using Microsoft.Build.Framework;
            using System.Text.Json;
            public sealed class DiagnosticLogger : ILogger
            {
                public LoggerVerbosity Verbosity { get; set; }
                public string Parameters { get; set; }
                public void Initialize(IEventSource source)
                {
                    source.ErrorRaised += (_, error) =>
                    {
                        if (error.Code?.StartsWith("FP", StringComparison.Ordinal) != true) return;
                        var record = JsonSerializer.Serialize(new { error.Code, error.File, error.LineNumber, error.ColumnNumber,
                            error.EndLineNumber, error.EndColumnNumber, error.Message });
                        File.AppendAllText(Parameters, record + Environment.NewLine);
                    };
                }
                public void Shutdown() { }
            }
            """);
        var build = await ProcessRunner.RunAsync(["build", project.File("DiagnosticLogger/DiagnosticLogger.csproj"), "-m:1", "--disable-build-servers"]);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        return "-logger:" + project.File("DiagnosticLogger/bin/Debug/net10.0/DiagnosticLogger.dll") + ";" + project.File("diagnostics.jsonl");
    }
}

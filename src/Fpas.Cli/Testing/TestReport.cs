// [Documentation](../../../docs/pascal/tools/testing.md)
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Testing;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Testing;

internal sealed record TestError(string Code, string Message);
internal sealed record ProjectTestResult(TestProject Project, BuildResult Runner, string? ResultsFile, TrxReport? Result, IReadOnlyList<DiscoveredTest>? Discovery)
{
    public IReadOnlyList<TestRerun>? Reruns { get; init; }
}
internal sealed record TestReport(bool Success, string Status, string Mode, string? Context)
{
    internal const int MaximumBytes = 4_194_304;
    public string? ResultsDirectory { get; init; }
    public BuildResult? Build { get; init; }
    public IReadOnlyList<ProjectTestResult> Projects { get; init; } = [];
    public TestError? Error { get; init; }

    internal bool Write(bool json)
    {
        if (json)
        {
            var text = JsonSerializer.Serialize(this, TestJson.Default.TestReport);
            var limited = Encoding.UTF8.GetByteCount(text) > MaximumBytes;
            if (limited && Projects.Any(project => project.Reruns is not null))
            {
                text = JsonSerializer.Serialize(this with { Projects = Projects.Select(project => project with { Reruns = null }).ToArray() }, TestJson.Default.TestReport);
                limited = Encoding.UTF8.GetByteCount(text) > MaximumBytes;
            }
            if (limited)
                text = JsonSerializer.Serialize(this with { Success = false, Status = "failed", Build = null, Projects = [],
                    Error = new("report_limit", "The test report exceeds 4 MiB; inspect the native artifacts in resultsDirectory.") }, TestJson.Default.TestReport);
            Console.WriteLine(text); return !limited;
        }
        if (Mode == "list")
        {
            foreach (var project in Projects)
            {
                foreach (var test in project.Discovery ?? []) Console.WriteLine(test.FullyQualifiedName);
                Console.Error.Write(project.Runner.Error);
                if (project.Runner.ExitCode != 0) Console.Error.Write(project.Runner.Output);
            }
        }
        if (Error is { } error) Console.Error.WriteLine(Diagnostics.DiagnosticRenderer.Safe(error.Code + ": " + error.Message));
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(TestReport))]
[JsonSerializable(typeof(TestRerun))]
internal partial class TestJson : JsonSerializerContext;

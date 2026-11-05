using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Projects;

internal sealed record ProjectError(string Code, string Message, IReadOnlyList<string>? Candidates = null);
internal sealed record ProgramResult(int ExitCode, string Output, string Error, bool Truncated);
internal sealed record PlannedFile(string Path, string State, string Content);
internal sealed record ProjectReport(string Command, bool Success, string Status, string? Context)
{
    public ProjectOutput? Target { get; init; }
    public BuildResult? Build { get; init; }
    public ProgramResult? Program { get; init; }
    public bool DryRun { get; init; }
    public IReadOnlyList<PlannedFile>? Files { get; init; }
    public ProjectError? Error { get; init; }

    internal void Write(bool json)
    {
        if (json) { Console.WriteLine(JsonSerializer.Serialize(this, ProjectJson.Default.ProjectReport)); return; }
        if (Build is { } build) { Console.Error.Write(build.Output); Console.Error.Write(build.Error); }
        if (Files is not null)
            foreach (var file in Files) Console.WriteLine($"{file.State}: {file.Path}");
        if (Error is { } error)
        {
            Console.Error.WriteLine($"{error.Code}: {error.Message}");
            foreach (var candidate in error.Candidates ?? []) Console.Error.WriteLine("  " + candidate);
        }
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(ProjectReport))]
internal partial class ProjectJson : JsonSerializerContext;

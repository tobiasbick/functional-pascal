// [Documentation](../../../docs/pascal/program-structure/project-commands.md#inspect-the-workspace)
using Fpas.Cli.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Projects;
using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Projects;

internal sealed class WorkspaceCommand
{
    private readonly Argument<string> context = new("context") { Description = "An existing .slnx, .fpasproj, .csproj or .fsproj." };
    private readonly Option<string> file = new("--file") { Description = "List projects and target frameworks whose evaluated Compile items include this file." };
    private readonly Option<string[]> affected = new("--affected") { Description = "List source owners, dependent projects and affected tests. Repeat for multiple paths (up to 128).",
        Arity = ArgumentArity.OneOrMore, AllowMultipleArgumentsPerToken = false };
    private readonly Option<string> configuration = new("--configuration") { Description = "Solution or project configuration.", DefaultValueFactory = _ => "Debug" };
    private readonly Option<int> timeout = new("--timeout") { Description = "Evaluation deadline in seconds (1..3600).", DefaultValueFactory = _ => 30 };

    internal Command Command { get; } = new("workspace", """
        List evaluated projects, configuration mappings, roles and project references as JSON.
        Uses installed SDKs without building, restoring or running project targets.

        Examples:
          fpas workspace Example.slnx --configuration Release

          fpas workspace Example.slnx --file src/View.fpas

          fpas workspace Example.slnx --affected src/View.fpas
        """);

    internal WorkspaceCommand()
    {
        timeout.Range(1, 3600);
        Command.Arguments.Add(context);
        Command.Options.Add(configuration);
        Command.Options.Add(file);
        Command.Options.Add(affected);
        Command.Options.Add(timeout);
        Command.Excludes(file, affected, parse => parse.GetValue(file) is not null, parse => parse.GetResult(affected) is { IdentifierTokenCount: > 0 });
        Command.SetAction(Execute);
    }

    internal int ArgumentError(ParseResult parse, string message)
        => new WorkspaceReport("failed", CommandArguments.Supplied(parse, context), CommandArguments.Supplied(parse, configuration) ?? "Debug",
            [], new("invalid_argument", message)).Write(2);

    private async Task<int> Execute(ParseResult parse, CancellationToken token)
    {
        var path = parse.GetValue(context)!;
        var selected = parse.GetValue(configuration)!;
        var sourceFile = parse.GetValue(file);
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
        deadline.CancelAfter(TimeSpan.FromSeconds(parse.GetValue(timeout)));
        try
        {
            path = Path.GetFullPath(path);
            sourceFile = sourceFile is null ? null : Path.GetFullPath(sourceFile);
            if (parse.GetValue(affected) is { Length: > 0 } files)
            {
                if (files.Length > 128) return Fail("invalid_argument", "Use at most 128 --affected paths.", 2);
                var (impacted, impact) = await WorkspaceInventory.ReadAffectedAsync(path, selected, files, deadline.Token);
                return new WorkspaceReport(impact.Complete ? "ok" : "partial", path, selected, impacted, null) { Impact = impact }
                    .Write(impact.Complete ? 0 : 1);
            }
            var projects = await WorkspaceInventory.ReadAsync(path, selected, deadline.Token, sourceFile);
            var complete = projects.All(project => project.Issue is null);
            return new WorkspaceReport(complete ? "ok" : "partial", path, selected, projects, null) { File = sourceFile }.Write(complete ? 0 : 1);
        }
        catch (ContextFailure failure)
        {
            return new WorkspaceReport("failed", path, selected, [], AnalysisError.From(failure)) { File = sourceFile }.Write(failure.Code == "invalid_argument" ? 2 : 1);
        }
        catch (OperationCanceledException) when (deadline.IsCancellationRequested)
        {
            return token.IsCancellationRequested ? Fail("cancelled", "Workspace evaluation cancelled.", 130)
                : Fail("timeout", "Workspace evaluation exceeded --timeout.", 1);
        }
        catch (ArgumentException exception) { return Fail("invalid_argument", exception.Message, 2); }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or System.ComponentModel.Win32Exception or PlatformNotSupportedException or System.Xml.XmlException)
        { return Fail("evaluation_failed", exception.Message, 1); }

        int Fail(string code, string message, int exit)
            => new WorkspaceReport(code == "cancelled" ? code : "failed", path, selected, [], new(code, message)) { File = sourceFile }.Write(exit);
    }
}

internal sealed record WorkspaceReport(string Status, string? Context, string Configuration, IReadOnlyList<WorkspaceProject> Projects, AnalysisError? Error)
{
    /// <summary>Gets the version of the workspace report layout.</summary>
    public int SchemaVersion { get; init; } = 1;
    /// <summary>Gets the absolute source path used to filter project ownership, when selected.</summary>
    public string? File { get; init; }
    /// <summary>Gets the impact summary when the command evaluates affected sources.</summary>
    public WorkspaceImpact? Impact { get; init; }

    internal int Write(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this, WorkspaceJson.Default.WorkspaceReport);
        if (bytes.Length > QueryResult.OutputLimit)
        {
            bytes = JsonSerializer.SerializeToUtf8Bytes(this with { Status = "failed", Projects = [], Impact = null, Error = new("result_limit", "Workspace report exceeds 4 MiB.") },
                WorkspaceJson.Default.WorkspaceReport);
            exit = 1;
        }
        using var output = Console.OpenStandardOutput();
        output.Write(bytes);
        Console.WriteLine();
        return exit;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(WorkspaceReport))]
internal partial class WorkspaceJson : JsonSerializerContext;

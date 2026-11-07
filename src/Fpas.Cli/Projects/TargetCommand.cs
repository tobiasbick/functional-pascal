using Fpas.Cli.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Projects;
using System.CommandLine;

namespace Fpas.Cli.Projects;

internal sealed class TargetCommand
{
    private readonly Argument<string> context = new("context") { Description = "An existing project or .slnx solution." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Debug", Description = "Build or solution configuration." };
    private readonly Option<string> project = new("--project") { Description = "Select an included executable project." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 30, Description = "Evaluation deadline in seconds (1..3600)." };

    internal Command Command { get; } = new("target", """
        Describe the executable selected by a project or solution as JSON.
        Uses the evaluated target path and mapped configuration. Does not restore or compile.

        Examples:
          fpas target examples/hello/Hello.fpasproj

          fpas target App.slnx --configuration Release --project Host/Host.csproj
        """);

    internal TargetCommand()
    {
        ContextCommandOptions.Range(timeout, 1, 3600);
        Command.Arguments.Add(context);
        foreach (var option in new Option[] { configuration, project, timeout }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        deadline.CancelAfter(TimeSpan.FromSeconds(result.GetValue(timeout)));
        string? path = null;
        try
        {
            path = Path.GetFullPath(result.GetValue(context)!);
            var target = await ProjectDriver.SelectExecutableAsync(path, result.GetValue(configuration)!, result.GetValue(project), deadline.Token);
            new ProjectReport("target", true, "ok", path) { Target = target }.Write(json: true);
            return 0;
        }
        catch (ContextFailure failure)
        { return Fail(path, failure.Code, failure.Message, failure.Code is "invalid_argument" or "ambiguous_project" or "not_executable" ? 2 : 1, failure.Candidates); }
        catch (OperationCanceledException) when (deadline.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            return Fail(path, cancelled ? "cancelled" : "timeout", cancelled ? "Evaluation cancelled." : "Evaluation exceeded --timeout.", cancelled ? 130 : 1);
        }
        catch (Exception error) when (error is IOException or ArgumentException or System.Text.Json.JsonException or UnauthorizedAccessException or System.Xml.XmlException)
        { return Fail(path, "evaluation_failed", error.Message, 1); }
    }

    internal static int Fail(string? path, string code, string message, int exit, IReadOnlyList<string>? candidates = null)
    {
        new ProjectReport("target", false, code == "cancelled" ? code : "failed", path) { Error = new(code, message, candidates) }.Write(json: true);
        return exit;
    }
}

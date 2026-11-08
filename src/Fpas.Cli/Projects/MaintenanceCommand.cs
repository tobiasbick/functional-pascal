using Fpas.Cli.Analysis;
using Fpas.Cli.Reporting;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using System.CommandLine;

namespace Fpas.Cli.Projects;

internal sealed class MaintenanceCommand
{
    private readonly Argument<string> context = new("context") { Description = "Existing project or .slnx solution." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Debug", Description = "MSBuild configuration." };
    private readonly Option<string> report = new("--report") { Description = "Emit one project JSON report." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 120, Description = "Whole-operation deadline, 1..3600 seconds." };
    internal Command Command { get; }

    internal MaintenanceCommand(string name)
    {
        Command = new(name, $"Run the SDK's {name} operation for an explicit project or solution.\n" +
            "MSBuild owns the affected files and package resolution. No directory is recursively removed by FPAS.\n\n" +
            $"Examples:\n  fpas {name} examples/hello/Hello.fpasproj --report json\n\n  fpas {name} App.slnx --configuration Release\n");
        report.AcceptOnlyFromAmong("json"); ContextCommandOptions.Range(timeout, 1, 3600);
        Command.Arguments.Add(context);
        foreach (var option in new Option[] { configuration, report, timeout }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal void ArgumentError(ParseResult result, string message) => Failure(null, "invalid_argument", message + $" See fpas {Command.Name} --help.").Write(Json(result));
    private bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    private ProjectReport Failure(string? input, string code, string message) => new(Command.Name, false, code is "cancelled" or "timeout" ? code : "failed", input) { Error = new(code, message) };

    private async Task<int> Execute(ParseResult result, CancellationToken cancellation)
    {
        string? input = null;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var linked = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token);
        try
        {
            input = ProjectDriver.ValidateContext(result.GetValue(context)!);
            var config = result.GetValue(configuration)!;
            if (string.IsNullOrWhiteSpace(config)) throw new ArgumentException("Configuration must not be empty.");
            var liveOutput = ProjectProgress.BuildOutput(Json(result), Command.Name == "restore" ? "Restoring" : "Cleaning", input);
            var built = await BuildProcess.Run(Path.GetDirectoryName(input)!, [Command.Name, input, "--disable-build-servers", "-nr:false",
                "-p:UseSharedCompilation=false", "-p:Configuration=" + MsBuildSyntax.Escape(config), "--verbosity", "minimal"], linked.Token,
                liveOutput: liveOutput);
            var success = built.ExitCode == 0 && !built.Truncated;
            new ProjectReport(Command.Name, success, success ? "ok" : "failed", input) { Build = built,
                Error = success ? null : new(built.Truncated ? "output_limit" : Command.Name + "_failed", "Inspect the SDK output.") }.Write(Json(result));
            return success ? 0 : 1;
        }
        catch (OperationCanceledException)
        { Failure(input, cancellation.IsCancellationRequested ? "cancelled" : "timeout", "Operation interrupted after owned-process cleanup.").Write(Json(result)); return cancellation.IsCancellationRequested ? 130 : 1; }
        catch (Exception failure) when (failure is IOException or UnauthorizedAccessException or ArgumentException or Workspaces.Loading.ContextFailure or System.ComponentModel.Win32Exception)
        { Failure(input, failure is ArgumentException or Workspaces.Loading.ContextFailure ? "invalid_argument" : "operation_failed", failure.Message).Write(Json(result)); return failure is ArgumentException or Workspaces.Loading.ContextFailure ? 2 : 1; }
    }
}

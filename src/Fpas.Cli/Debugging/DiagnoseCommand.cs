// [Documentation](../../../docs/pascal/tools/cli-diagnostics.md#failure-reports-and-dumps)
using System.CommandLine;
using Fpas.Cli.Analysis;
using Fpas.Cli.Profiling;
using Fpas.Cli.Reporting;

namespace Fpas.Cli.Debugging;

internal sealed class DiagnoseCommand
{
    private readonly Command dump = new("dump", "Capture an explicit live .NET PID with the pinned dotnet-dump tool. Dumps contain process memory; retain them privately.\n\nExamples:\n  fpas diagnose dump --process-id 1234 --output failure.dmp\n");
    private readonly Command inspect = new("inspect-dump", "Run bounded SOS commands against a saved dump with the pinned dotnet-dump tool. Always exits the analysis session.\n\nExamples:\n  fpas diagnose inspect-dump failure.dmp --command \"clrstack -all\"\n");
    private readonly Argument<string> file = new("dump") { Description = "Existing dump file from the matching OS/runtime." };
    private readonly Option<int> timeout = new("--timeout") { Recursive = true, DefaultValueFactory = _ => 30, Description = "Whole-operation deadline, 1..3600 seconds." };
    private readonly Option<string> tools = new("--tools-directory") { Recursive = true, DefaultValueFactory = _ => DiagnosticTools.DefaultDirectory, Description = "Pinned tools prepared by fpas profile setup." };
    private readonly Option<int> pid = new("--process-id") { Required = true, Description = "Explicit live target PID." };
    private readonly Option<string> output = new("--output") { Required = true, Description = "Fresh dump filename." };
    private readonly Option<string[]> commands = new("--command") { Description = "SOS command, repeatable; defaults to clrstack -all.", AllowMultipleArgumentsPerToken = false };
    internal Command Command { get; } = new("diagnose", "Capture and inspect runtime dumps with pinned Microsoft tools.\n\nExamples:\n  fpas diagnose dump --help\n\n  fpas diagnose inspect-dump --help\n");

    internal DiagnoseCommand()
    {
        ContextCommandOptions.Range(timeout, 1, 3600); Command.Options.Add(timeout); Command.Options.Add(tools);
        foreach (var sub in new[] { dump, inspect }) { Command.Subcommands.Add(sub); sub.SetAction(Execute); }
        dump.Options.Add(pid); dump.Options.Add(output); inspect.Arguments.Add(file); inspect.Options.Add(commands);
    }
    internal bool Owns(Command command) => command == Command || Command.Subcommands.Contains(command);
    internal static int ArgumentError(string message)
        => OperationReport.Fail("diagnose", "invalid_argument", message + " See fpas diagnose --help.", 2);

    private async Task<int> Execute(ParseResult result, CancellationToken cancellation)
    {
        var name = "diagnose " + result.CommandResult.Command.Name;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var linked = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token);
        try
        {
            var args = new List<string>(); ArtifactCapture? artifact = null;
            if (result.CommandResult.Command == dump)
            {
                if (result.GetValue(pid) <= 0) throw new ArgumentException("--process-id must be positive.");
                artifact = new(result.GetValue(output)!);
                args.AddRange(["collect", "--process-id", result.GetValue(pid).ToString(System.Globalization.CultureInfo.InvariantCulture), "--type", "Heap", "--output", artifact.StagedPath]);
            }
            else
            {
                var path = Path.GetFullPath(result.GetValue(file)!);
                if (!File.Exists(path)) throw new ArgumentException("Select an existing dump.");
                var selected = result.GetValue(commands) ?? ["clrstack -all"];
                if (selected.Length is < 1 or > 16 || selected.Any(item => item.Length > 4096 || item.Contains('\0') || item is "exit" or "quit")) throw new ArgumentException("Supply 1..16 SOS commands of at most 4096 characters, without exit/quit or NUL.");
                args.AddRange(["analyze", path]); foreach (var command in selected) args.AddRange(["-c", command]); args.AddRange(["-c", "exit"]);
            }
            var process = await DiagnosticTools.Run(result.GetValue(tools)!, "dotnet-dump", Environment.CurrentDirectory, args, linked.Token);
            var success = process.ExitCode == 0 && !process.Truncated;
            return new OperationReport(name, success, success ? "ok" : "failed") { Artifact = success ? artifact?.Publish() : artifact?.Partial, Process = process,
                Error = success ? null : new("diagnostic_failed", "Inspect the pinned tool's captured diagnostics.") }.Write(success ? 0 : 1);
        }
        catch (OperationCanceledException) { return OperationReport.Fail(name, cancellation.IsCancellationRequested ? "cancelled" : "timeout", "Diagnostic interrupted after owned-process cleanup.", cancellation.IsCancellationRequested ? 130 : 1); }
        catch (Exception error) when (error is not OutOfMemoryException) { return OperationReport.Fail(name, error is ArgumentException ? "invalid_argument" : "diagnostic_failed", error.Message, error is ArgumentException ? 2 : 1); }
    }
}

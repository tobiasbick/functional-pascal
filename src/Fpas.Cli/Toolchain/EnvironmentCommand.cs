using Fpas.Cli.Analysis;
using System.CommandLine;

namespace Fpas.Cli.Toolchain;

internal sealed class EnvironmentCommand
{
    private readonly Option<bool> json = new("--json") { Description = "Write one structured environment report to stdout." };
    private readonly Option<string> directory = new("--directory") { Description = "Working directory for .NET SDK selection; defaults to the current directory." };
    private readonly Option<int> timeout = new("--timeout") { Description = "Overall inspection deadline in seconds (1..3600).", DefaultValueFactory = _ => 30 };
    internal Command Command { get; } = new("env", """
        Inspect the running FPAS toolchain, installed SDKs, .NET, debugger and stdio language server.

        No project is built, restored or evaluated. SDK cache hashes detect stale compiler files.
        Exit codes: 0 inspection without errors, 1 errors/deadline, 2 arguments, 130 cancellation.

        Examples:
          fpas env

          fpas env --json

          fpas env --directory examples/mandelbrot --json

        """);

    internal EnvironmentCommand()
    {
        ContextCommandOptions.Range(timeout, 1, 3600);
        foreach (var option in new Option[] { json, directory, timeout }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal void ReportArgumentError(ParseResult result, string message)
        => new EnvironmentReport(false, "failed", Environment.CurrentDirectory)
            { Issues = [new("error", "invalid_argument", message + " See fpas env --help.")] }.Write(result.GetValue(json));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var cwd = Environment.CurrentDirectory;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        EnvironmentReport report; int exit;
        try
        {
            cwd = Path.GetFullPath(result.GetValue(directory) ?? cwd);
            if (!Directory.Exists(cwd)) throw new ArgumentException("--directory must identify an existing directory.");
            report = await EnvironmentInspection.Read(cwd, combined.Token);
            exit = report.Success ? 0 : 1;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            report = Failure(cancelled ? "cancelled" : "timeout", cancelled ? "Environment inspection cancelled." : "Environment inspection exceeded --timeout.");
            exit = cancelled ? 130 : 1;
        }
        catch (ArgumentException exception) { report = Failure("invalid_argument", exception.Message); exit = 2; }
        catch (Exception exception) when (EnvironmentInspection.Recoverable(exception)) { report = Failure("inspection_failed", exception.Message); exit = 1; }
        return report.Write(result.GetValue(json)) ? exit : 1;

        EnvironmentReport Failure(string code, string message) => new(false, code is "cancelled" or "timeout" ? code : "failed", cwd)
            { Issues = [new("error", code, message)] };
    }
}

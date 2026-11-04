using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class CheckCommand
{
    private readonly ContextCommandOptions options = new();
    private readonly Option<string> diagnostics = new("--diagnostics") { Description = "Write one JSON result to stdout." };
    internal Command Command { get; } = new("check", """
        Validate an evaluated FPAS project or solution without publishing analysis-owned outputs.
        Preparation may restore packages and build dependencies. Existing root outputs are preserved.
        Exit codes: 0 success/help, 1 analysis/setup failure, 2 invalid arguments, 130 cancellation.

        Examples:
          fpas check Game/Game.fpasproj
          fpas check Game.slnx --configuration Release --diagnostics json
        """);

    internal CheckCommand()
    {
        diagnostics.AcceptOnlyFromAmong("json");
        options.AddTo(Command); Command.Options.Add(diagnostics);
        Command.SetAction(async (result, token) => await Execute(result, token));
    }

    internal bool JsonRequested(ParseResult result) => result.GetResult(diagnostics)?.Tokens.Any(token => token.Value == "json") == true;

    internal void ReportArgumentError(ParseResult result, string message)
        => CheckResult.Failure(result.GetValue(options.Context), result.GetValue(options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas check --help.").Report(JsonRequested(result));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var input = options.Read(result); var selected = input.Context; var selectedConfiguration = input.Configuration;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(options.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult report; int exit;
        try
        {
            var workspace = await WorkspaceLoader.LoadAsync(input, combined.Token);
            var analysis = WorkspaceAnalysis.Analyze(workspace, combined.Token);
            await workspace.VerifyInputsAsync(combined.Token);
            combined.Token.ThrowIfCancellationRequested();
            report = CheckResult.From(analysis); exit = report.Success ? 0 : 1;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            report = CheckResult.Failure(selected, selectedConfiguration, cancelled ? "cancelled" : "timeout", cancelled ? "Analysis cancelled." : "Analysis exceeded --timeout.");
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            report = CheckResult.Failure(selected, selectedConfiguration, failure.Code, failure.Message, failure.OutputTruncated, failure.Candidates);
            exit = failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or ArgumentException or PlatformNotSupportedException)
        {
            report = CheckResult.Failure(selected, selectedConfiguration, "context_failed", exception.Message); exit = 1;
        }
        report.Report(JsonRequested(result)); return exit;
    }
}

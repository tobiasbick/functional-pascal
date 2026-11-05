using Fpas.Cli.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Documentation;

internal sealed class DocCommand
{
    private readonly ContextCommandOptions options = new();
    private readonly Option<string> name = new("--name") { Required = true, Description = "Case-sensitive simple or qualified declaration name. A type also lists its directly declared members." };
    private readonly Option<string> match = new("--match") { DefaultValueFactory = _ => "exact", Description = "Name matching: exact, prefix or contains." };
    private readonly Option<string> project = new("--project") { Description = "Restrict source declarations and reference lookup to one analyzed FPAS project." };
    private readonly Option<string> assembly = new("--assembly") { Description = "Restrict entries to an assembly's exact simple name." };
    private readonly Option<int> limit = new("--limit") { DefaultValueFactory = _ => 20, Description = "Maximum entries (1..100); documentation shares a 64 KiB UTF-8 budget." };
    private readonly Option<int> offset = new("--offset") { DefaultValueFactory = _ => 0, Description = "Skip sorted entries; combine pages only with equal snapshotId." };
    private readonly Option<string> report = new("--report") { Description = "Write one structured JSON document to stdout." };
    internal Command Command { get; } = new("doc", """
        Look up FPAS declarations and public APIs from the project's actual .NET references.
        Shows source Markdown or adjacent assembly XML documentation, with explicit missing-text states.
        Reads metadata without executing referenced code. Preparation may restore/build dependencies.
        External signatures describe CLR metadata; they do not certify FPAS call support.
        Exit codes: 0 success/not found/help, 1 analysis/setup/deadline failure, 2 arguments, 130 cancellation.

        Examples:
          fpas doc examples/mandelbrot/Mandelbrot.fpasproj --name EscapeIterations

          fpas doc examples/hello/Hello.fpasproj --name System.Console.WriteLine --report json

          fpas doc Example.slnx --name System.Collections.Generic.List --limit 50 --report json
        """);

    internal DocCommand()
    {
        options.AddTo(Command); match.AcceptOnlyFromAmong("exact", "prefix", "contains"); report.AcceptOnlyFromAmong("json");
        ContextCommandOptions.Range(limit, 1, 100); ContextCommandOptions.Range(offset, 0, int.MaxValue);
        foreach (var option in new Option[] { name, match, project, assembly, limit, offset, report }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }
    private bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    internal void ReportArgumentError(ParseResult result, string message)
        => DocReport.Failure(result.GetValue(options.Context), result.GetValue(options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas doc --help.").Write(Json(result));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var json = Json(result); var input = result.GetValue(options.Context); var configuration = result.GetValue(options.Configuration)!;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(options.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult? check = null; DocReport response; int exit;
        try
        {
            var search = new DocumentationSearch(result.GetValue(name)!, result.GetValue(match)!, result.GetValue(project), result.GetValue(assembly), result.GetValue(limit), result.GetValue(offset));
            DocumentationQuery.Validate(search);
            var workspace = await WorkspaceLoader.LoadAsync(options.Read(result), combined.Token);
            var analysis = WorkspaceAnalysis.Analyze(workspace, combined.Token); check = CheckResult.From(analysis);
            var query = new DocumentationQuery(analysis); var found = query.Search(search, combined.Token);
            await workspace.VerifyInputsAsync(combined.Token); query.VerifyInputs(combined.Token); combined.Token.ThrowIfCancellationRequested();
            response = DocReport.From(check, found); exit = 0;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            response = DocReport.Failure(input, configuration, cancelled ? "cancelled" : "timeout", cancelled ? "Documentation lookup cancelled." : "Documentation lookup exceeded --timeout.", check);
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            response = DocReport.Failure(input, configuration, failure.Code, failure.Message, check, failure.OutputTruncated, failure.Candidates);
            exit = failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or ArgumentException or BadImageFormatException or InvalidOperationException or PlatformNotSupportedException)
        {
            var argument = exception is ArgumentException && exception is not DecoderFallbackException;
            response = DocReport.Failure(input, configuration, argument ? "invalid_argument" : "documentation_failed", exception.Message, check); exit = argument ? 2 : 1;
        }
        return response.Write(json) ? exit : 1;
    }
}

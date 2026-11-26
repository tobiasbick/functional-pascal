// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-a-project-or-solution)
using Fpas.Workspaces.Lint;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class LintCommand
{
    private static readonly LintLimits Defaults = new();
    private readonly ContextCommandOptions options = new();
    private readonly Option<string> reportFormat = new("--report") { Description = "Write one JSON lint result to stdout." };
    private readonly Option<string> baseline = new("--baseline") { Description = "Apply an explicitly selected lint baseline." };
    private readonly Option<string> writeBaseline = new("--write-baseline") { Description = "Create or refresh a lint baseline from current errors." };
    private readonly Option<int> heldInMind = new("--held-in-mind")
    { Description = "Maximum local tangle size, interface width, or exports learned by one consumer.", DefaultValueFactory = _ => Defaults.HeldInMind };
    private readonly Option<int> maxDirectoryEntries = new("--max-directory-entries")
    { Description = "Maximum immediate FPAS source entries per directory.", DefaultValueFactory = _ => Defaults.MaxDirectoryEntries };
    private readonly Option<int> maxLines = new("--max-lines")
    { Description = "Maximum physical lines per production source file.", DefaultValueFactory = _ => Defaults.MaxLines };
    private readonly Option<int> maxNesting = new("--max-nesting")
    { Description = "Maximum control-flow nesting depth before a warning (default 4).", DefaultValueFactory = _ => Defaults.MaxNesting };

    internal Command Command { get; } = new("lint", """
        Check FPAS source structure in an evaluated project or solution.
        Rules: module-cycle, namespace-cycle, tangle, field-leak, exported-sync, lonely-protocol,
        shallow-module, wide-directory, long-file, deep-nesting, unused-clr-alias.
        Preparation may restore packages and build dependencies.
        Exit codes: 0 no active errors or help, 1 active errors or analysis failure, 2 invalid arguments, 130 cancellation.

        Examples:
          fpas lint App.fpasproj

          fpas lint App.slnx --existing-inputs --report json

          fpas lint App.slnx --max-lines 500 --max-directory-entries 12

          fpas lint App.slnx --baseline lint-baseline.json
        """);

    internal LintCommand()
    {
        reportFormat.AcceptValues("json");
        options.AddTo(Command);
        Command.Options.Add(reportFormat);
        Command.Options.Add(baseline);
        Command.Options.Add(writeBaseline);
        Command.Excludes(baseline, writeBaseline, result => result.GetValue(baseline) is not null, result => result.GetValue(writeBaseline) is not null);
        foreach (var limit in new[] { heldInMind, maxDirectoryEntries, maxLines, maxNesting })
        {
            limit.Range(1, int.MaxValue);
            Command.Options.Add(limit);
        }
        Command.SetAction(Execute);
    }

    private bool JsonRequested(ParseResult result)
        => result.GetResult(reportFormat)?.Tokens.Any(token => token.Value == "json") == true;

    internal int ReportArgumentError(ParseResult result, string message)
    {
        LintReport.Failure(CommandArguments.Supplied(result, options.Context), result.GetValue(options.Configuration) ?? "Debug",
            new("invalid_argument", message + " See fpas lint --help.")).Write(JsonRequested(result));
        return 2;
    }

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var input = options.Read(result);
        var json = JsonRequested(result);
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        deadline.CancelAfter(TimeSpan.FromSeconds(result.GetValue(options.Timeout)));
        var token = deadline.Token;
        try
        {
            var workspace = await FileWorkspace.OpenAsync(input, token);
            var analysis = workspace.Analysis!;
            var check = CheckResult.From(analysis);
            var ready = analysis.Success && analysis.IsComplete;
            var limits = new LintLimits(result.GetValue(heldInMind), result.GetValue(maxDirectoryEntries), result.GetValue(maxLines), result.GetValue(maxNesting));
            var findings = ready ? WorkspaceLinter.Analyze(analysis, limits, token) : [];
            var report = LintReport.From(check, findings, ready);
            var write = result.GetValue(writeBaseline);
            var baselinePath = write ?? result.GetValue(baseline);
            if (ready && baselinePath is not null)
                report = await LintBaseline.ApplyAsync(Path.GetFullPath(baselinePath), write is not null, input, limits, report, workspace, token);
            else
                await workspace.VerifyAsync(token);
            return report.Write(json) && report.Success ? 0 : 1;
        }
        catch (OperationCanceledException) when (token.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            Fail(new(cancelled ? "cancelled" : "timeout", cancelled ? "Lint cancelled." : "Lint exceeded --timeout."));
            return cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            Fail(AnalysisError.From(failure));
            return failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or PlatformNotSupportedException)
        {
            Fail(new("context_failed", exception.Message));
            return 1;
        }

        void Fail(AnalysisError error) => LintReport.Failure(input.Context, input.Configuration, error).Write(json);
    }
}

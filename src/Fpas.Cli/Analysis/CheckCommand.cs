using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class CheckCommand
{
    private readonly ContextCommandOptions options = new();
    private readonly Option<string> diagnostics = new("--diagnostics") { Description = "Write one JSON result to stdout." };
    private readonly Option<bool> fix = new("--fix") { Description = "Apply one batch of automatic diagnostic corrections, then reload and check again." };
    private readonly Option<bool> dryRun = new("--dry-run") { Description = "With --fix, show proposed edits without changing source files." };
    internal Command Command { get; } = new("check", """
        Check an FPAS project or solution without emitting its own outputs.
        Uses evaluated project inputs. Preparation may restore packages and build dependencies.
        Existing root outputs are preserved.
        --fix applies automatic proposals only; --fix --dry-run previews them.
        Refuses conflicting/stale corrections. Remaining errors still return exit 1.
        Exit codes: 0 success/help, 1 analysis/setup failure, 2 invalid arguments, 130 cancellation.

        Examples:
          fpas check examples/mandelbrot/Mandelbrot.fpasproj

          fpas check examples/mandelbrot/Mandelbrot.fpasproj --configuration Release --diagnostics json

          fpas check Example.fpasproj --fix --dry-run --diagnostics json

          fpas check Example.fpasproj --fix
        """);

    internal CheckCommand()
    {
        diagnostics.AcceptOnlyFromAmong("json");
        options.AddTo(Command); Command.Options.Add(diagnostics); Command.Options.Add(fix); Command.Options.Add(dryRun);
        Command.SetAction(async (result, token) => await Execute(result, token));
    }

    internal bool JsonRequested(ParseResult result) => result.GetResult(diagnostics)?.Tokens.Any(token => token.Value == "json") == true;

    internal void ReportArgumentError(ParseResult result, string message)
        => CheckResult.Failure(result.GetValue(options.Context), result.GetValue(options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas check --help.").Report(JsonRequested(result));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        if (result.GetValue(dryRun) && !result.GetValue(fix))
        { ReportArgumentError(result, "--dry-run requires --fix."); return 2; }
        var input = options.Read(result); var selected = input.Context; var selectedConfiguration = input.Configuration;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(options.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult? current = null; CheckFixes? fixes = null; CheckResult report; int exit;
        FileWorkspace? operation = null;
        try
        {
            operation = await FileWorkspace.OpenAsync(input, combined.Token);
            current = CheckResult.From(operation.Analysis!);
            if (result.GetValue(fix))
            {
                var plan = operation.Corrections(combined.Token);
                fixes = CheckFixes.From(plan, operation.Context.SnapshotId, result.GetValue(dryRun));
                if ((current with { Fixes = fixes }).JsonBytes().Length > 4_194_304)
                    throw new ContextFailure("result_limit", "Correction preview exceeds 4 MiB; no sources were changed.");
                if (!result.GetValue(dryRun) && plan.Edits.Documents.Count != 0)
                {
                    await operation.ApplyAsync(plan.Edits, token: combined.Token);
                    fixes = fixes with { State = "applied", Files = fixes.Files.Select(file => file with { Written = true }).ToArray() };
                    current = CheckResult.From(operation.Analysis!);
                }
            }
            if (!operation.Rechecked) await operation.VerifyAsync(combined.Token);
            combined.Token.ThrowIfCancellationRequested();
            if (fixes is { State: "applied" }) fixes = fixes with { Rechecked = true };
            report = current with { Fixes = fixes }; exit = report.Success ? 0 : 1;
        }
        catch (SourcePublicationFailure failure)
        {
            if (failure.Written.Count != 0) current = null;
            if (fixes is not null) fixes = fixes with { State = failure.Written.Count == 0 ? "failed" : "partial", RecoveryBackups = failure.Backups,
                Files = fixes.Files.Select(file => file with { Written = failure.Written.Contains(file.Path) }).ToArray() };
            report = Failure("fix_publication_failed", failure.Message); exit = 1;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            report = Failure(cancelled ? "cancelled" : "timeout", cancelled ? "Analysis cancelled." : "Analysis exceeded --timeout.");
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            report = Failure(failure.Code, failure.Message, failure.OutputTruncated, failure.Candidates);
            exit = failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or ArgumentException or PlatformNotSupportedException)
        {
            report = Failure("context_failed", exception.Message); exit = 1;
        }
        return report.Report(JsonRequested(result)) ? exit : 1;

        CheckResult Failure(string code, string message, bool truncated = false, IReadOnlyList<string>? candidates = null)
        {
            if (operation is not null)
            {
                current = operation.Analysis is { } analysis ? CheckResult.From(analysis) : null;
                if (fixes is not null && operation.Written.Count != 0)
                    fixes = fixes with { State = fixes.State == "partial" ? "partial" : "applied", Rechecked = operation.Rechecked,
                        Files = fixes.Files.Select(file => file with { Written = operation.Written.Contains(file.Path) }).ToArray() };
            }
            var failed = CheckResult.Failure(selected, selectedConfiguration, code, message, truncated, candidates);
            if (current is not null) failed = failed with { Projects = current.Projects, Diagnostics = current.Diagnostics,
                Sources = current.Sources, Preparation = current.Preparation, Coverage = current.Coverage with { Complete = false } };
            return failed with { Fixes = fixes is { State: "planned" } ? fixes with { State = "failed" } : fixes };
        }
    }
}

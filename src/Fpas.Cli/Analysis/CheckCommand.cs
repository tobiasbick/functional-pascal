// [Documentation](../../../docs/pascal/program-structure/cli.md#check-a-project-or-solution)
using Fpas.Cli.Compilation;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class CheckCommand
{
    private readonly bool editorControlled;
    private readonly ContextCommandOptions options = new();
    private readonly DiagnosticPageOptions diagnosticOptions = new();
    private readonly Option<string> reportFormat = new("--report") { Description = "Write one JSON result to stdout." };
    private readonly Option<string> overlay = AnalysisOverlay.FileOption();
    private readonly Option<string> expectedSnapshot = SnapshotGuard.CreateOption();
    private readonly Option<string[]> fixIds = new("--fix-id") { Description = "Select one automatic correction by its preview ID. Repeat for multiple corrections.",
        Arity = ArgumentArity.OneOrMore, AllowMultipleArgumentsPerToken = false };
    private readonly Option<bool> fix = new("--fix") { Description = "Apply one batch of automatic diagnostic corrections, then reload and check again." };
    private readonly Option<bool> dryRun = new("--dry-run") { Description = "With --fix, show proposed edits without changing source files." };
    internal Command Command { get; } = new("check", """
        Check a standalone FPAS program, project or solution without emitting its own outputs.
        Use a .fpas file or '-' for UTF-8 source on stdin. Both use installed .NET 10 references without restore or MSBuild.
        Project preparation may restore packages and build dependencies. Existing root outputs are preserved.
        --overlay checks a project draft with in-memory source inputs.
        --fix applies automatic proposals only; --fix --dry-run previews them.
        Refuses conflicting/stale corrections. Remaining errors still return exit 1.
        Exit codes: 0 success/help, 1 analysis/setup failure, 2 invalid arguments, 130 cancellation.

        Examples:
          echo 'program Probe; begin end; end program;' | fpas check - --report json

          fpas check examples/basics/hello/main.fpas --report json

          fpas check examples/numerics/mandelbrot/Mandelbrot.fpasproj --configuration Release --report json

          fpas check Example.fpasproj --fix --dry-run --report json

          fpas check Example.fpasproj --fix
        """);

    internal CheckCommand(bool editorControlled)
    {
        this.editorControlled = editorControlled;
        options.Context.Description = "Explicit .fpas program, .fpasproj or .slnx path, or '-' for source on stdin.";
        reportFormat.AcceptValues("json");
        options.AddTo(Command); Command.Options.Add(reportFormat); Command.Options.Add(fix); Command.Options.Add(dryRun);
        diagnosticOptions.AddTo(Command);
        Command.Options.Add(overlay);
        Command.Options.Add(expectedSnapshot);
        Command.Options.Add(fixIds);
        Command.Requires(fixIds, fix, result => result.GetResult(fixIds) is { IdentifierTokenCount: > 0 }, result => result.GetValue(fix));
        Command.Excludes(fix, overlay, result => result.GetValue(overlay) is not null);
        Command.Requires(dryRun, fix);
        Command.SetAction(async (result, token) => await Execute(result, token));
    }

    internal bool JsonRequested(ParseResult result) => result.GetResult(reportFormat)?.Tokens.Any(token => token.Value == "json") == true;

    internal void ReportArgumentError(ParseResult result, string message)
        => CheckResult.Failure(CommandArguments.Supplied(result, options.Context), CommandArguments.Supplied(result, options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas check --help.").Report(JsonRequested(result));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var input = options.Read(result); var selectedConfiguration = input.Configuration;
        var piped = result.GetValue(options.Context) == "-";
        var selected = piped ? PipedSource.Path : input.Context;
        var json = JsonRequested(result);
        var diagnostics = json || diagnosticOptions.Requested(result) ? diagnosticOptions.Read(result) : null;
        var timeout = TimeSpan.FromSeconds(result.GetValue(options.Timeout));
        using var deadline = new CancellationTokenSource(timeout);
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult? current = null; CheckFixes? fixes = null; CheckResult report; int exit;
        FileWorkspace? operation = null;
        try
        {
            if (piped || Path.GetExtension(selected).Equals(".fpas", StringComparison.OrdinalIgnoreCase))
            {
                if (piped && editorControlled)
                    throw new ContextFailure("invalid_argument", "Editor-controlled checks use stdin for cancellation. Supply a source file.");
                if (result.GetValue(fix) || result.GetValue(overlay) is not null || diagnostics?.Project is not null ||
                    result.GetResult(options.Configuration) is { Implicit: false })
                    throw new ContextFailure("invalid_argument", "Use a .fpasproj or .slnx with --fix, --overlay, --diagnostic-project or --configuration.");

                var source = await StandaloneCheck.AnalyzeAsync(selected, selectedConfiguration, combined.Token);
                SnapshotGuard.Verify(result, expectedSnapshot, source.SnapshotId);
                if (diagnostics is not null) source = source.PageDiagnostics(diagnostics);
                combined.Token.ThrowIfCancellationRequested();
                return source.Report(json) && source.Success ? 0 : 1;
            }
            if (result.GetValue(overlay) is { } manifest)
            {
                report = await AnalysisOverlay.RunAsync(input, manifest, timeout, (analysis, files, error) =>
                {
                    if (error is null) SnapshotGuard.Verify(result, expectedSnapshot, analysis.Context.SnapshotId);
                    var draft = CheckResult.From(analysis) with { OverlayFiles = files };
                    return error is null ? draft : draft with { Success = false, Status = "failed", SnapshotId = null,
                        Diagnostics = [], Coverage = draft.Coverage with { Complete = false }, Error = AnalysisError.From(error) };
                }, combined.Token);
                if (diagnostics is not null) report = report.PageDiagnostics(diagnostics);
                var draftExit = report.Success ? 0 : report.Error?.Code == "invalid_argument" ? 2 : 1;
                return report.Report(json) ? draftExit : 1;
            }
            operation = await FileWorkspace.OpenAsync(input, combined.Token);
            SnapshotGuard.Verify(result, expectedSnapshot, operation.Context.SnapshotId);
            current = CheckResult.From(operation.Analysis!);
            if (result.GetValue(fix))
            {
                var plan = operation.Corrections(combined.Token, result.GetValue(fixIds));
                fixes = CheckFixes.From(plan, operation.Context.SnapshotId, result.GetValue(dryRun));
                var preview = (current with { Fixes = fixes }).PageDiagnostics(diagnostics ?? new(null, null, 100, 0));
                if (preview.JsonBytes().Length > QueryResult.OutputLimit)
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
            report = Failure(failure.Code, failure.Message) with { Error = AnalysisError.From(failure) };
            exit = failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or ArgumentException or PlatformNotSupportedException)
        {
            report = Failure("context_failed", exception.Message); exit = 1;
        }
        if (diagnostics is not null) report = report.PageDiagnostics(diagnostics);
        return report.Report(json) ? exit : 1;

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

using Fpas.Cli.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Refactoring;
using System.CommandLine;
using System.Text.Json;

namespace Fpas.Cli.Refactoring;

internal sealed class RefactorCommand
{
    private sealed class Operation(string name)
    {
        internal ContextCommandOptions Context { get; } = new();
        internal Option<string> Plan { get; } = new("--plan") { Required = true, Description = "Explicit .json plan path; rename creates it without overwriting, apply reads it." };
        internal Option<string> File { get; } = new("--file") { Required = true, Description = "Source path for symbol selection." };
        internal Option<int> Line { get; } = new("--line") { Required = true, Description = "One-based source line." };
        internal Option<int> Column { get; } = new("--column") { Required = true, Description = "One-based UTF-16 source column." };
        internal Option<string> Project { get; } = new("--project") { Description = "Owning FPAS project for a linked source selection." };
        internal Option<string> NewName { get; } = new("--new-name") { Required = true, Description = "Unescaped simple ASCII identifier; keyword escaping is chosen automatically." };
        internal Option<string> Report { get; } = new("--report") { Description = "Write one JSON response to stdout." };
        internal Command Command { get; } = new(name, name == "rename" ? "Prepare a checked FPAS rename plan without changing sources." : "Revalidate and explicitly apply a rename plan.");
        internal bool Json(ParseResult result) => result.GetResult(Report)?.Tokens.Any(token => token.Value == "json") == true;
        internal void Bind(RefactorCommand owner)
        {
            Context.AddTo(Command); Report.AcceptOnlyFromAmong("json"); Command.Options.Add(Plan); Command.Options.Add(Report);
            if (name == "rename")
            {
                ContextCommandOptions.Range(Line, 1, int.MaxValue); ContextCommandOptions.Range(Column, 1, int.MaxValue);
                foreach (var option in new Option[] { File, Line, Column, Project, NewName }) Command.Options.Add(option);
            }
            Command.SetAction((result, token) => owner.Execute(this, result, token, name == "rename"));
        }
    }
    private readonly Operation[] operations = [new("rename"), new("apply")];
    internal Command Command { get; } = new("refactor", """
        Prepare and explicitly apply checked source refactorings in an FPAS project/solution.
        Rename follows resolved source symbols, including their source interface implementations.
        Plans cover analyzed FPAS sources; C#/F# sources, strings and comments are not rewritten.
        Dependency preparation may restore/build. Source bindings are checked before publication.
        Exit codes: 0 success/help, 1 conflict/setup/stale plan, 2 arguments, 130 cancellation.

        Examples:
          fpas refactor rename Example.fpasproj --file main.fpas --line 5 --column 10 --new-name Calculate --plan .temp-data/rename.json

          fpas refactor apply Example.fpasproj --plan .temp-data/rename.json --report json
        """);
    internal RefactorCommand() { foreach (var operation in operations) { operation.Bind(this); Command.Subcommands.Add(operation.Command); } }
    internal bool Owns(Command command) => command == Command || operations.Any(operation => operation.Command == command);
    internal void ReportArgumentError(ParseResult result, string message)
        => RefactorReport.Failure("invalid_argument", message + " See fpas refactor --help.").Write(operations.FirstOrDefault(operation => operation.Command == result.CommandResult.Command)?.Json(result) == true);

    private async Task<int> Execute(Operation operation, ParseResult result, CancellationToken cancellation, bool preparing)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(operation.Context.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token); var token = combined.Token;
        RefactorReport? current = null; int exit;
        try
        {
            var input = operation.Context.Read(result); var planPath = Path.GetFullPath(result.GetValue(operation.Plan)!);
            if (!Path.GetExtension(planPath).Equals(".json", StringComparison.OrdinalIgnoreCase)) throw new ArgumentException("--plan requires a .json path.");
            RenamePlan? stored = null; byte[]? storedBytes = null;
            if (preparing) _ = IdentifierFacts.Spell(result.GetValue(operation.NewName)!);
            else
            {
                SourcePublication.ValidateRegularPath(planPath);
                if (new FileInfo(planPath).Length > RenamePlanJson.MaximumBytes) throw new ContextFailure("invalid_plan", "Plan exceeds 4 MiB.");
                try { storedBytes = await File.ReadAllBytesAsync(planPath, token); stored = RenamePlanJson.Read(storedBytes); }
                catch (Exception error) when (error is JsonException or ArgumentException) { throw new ContextFailure("invalid_plan", error.Message); }
                if (stored.Context != input.Context || stored.Configuration != input.Configuration)
                    throw new ContextFailure("plan_context_mismatch", "Supply the plan's original context and configuration.");
            }
            var workspace = await WorkspaceLoader.LoadAsync(input, token);
            if (stored is not null && stored.InputSnapshotId != workspace.SnapshotId) throw new ContextFailure("stale_plan", "Project inputs changed after rename planning.");
            var analysis = WorkspaceAnalysis.Analyze(workspace, token);
            current = new(false, "validating", planPath, null, [], false, CheckResult.From(analysis).Diagnostics);
            var selection = stored?.Selection ?? new SourceSelection(result.GetValue(operation.File)!, result.GetValue(operation.Line), result.GetValue(operation.Column), result.GetValue(operation.Project));
            var prepared = await RenameService.PrepareAsync(analysis, selection, stored?.NewName ?? result.GetValue(operation.NewName)!, token);
            if (stored is not null && prepared.Plan.Id != stored.Id) throw new ContextFailure("invalid_plan", "Plan does not match the regenerated checked rename.");
            current = new(true, preparing ? "planned" : "applied", planPath, prepared.Plan,
                prepared.Plan.Files.Select(file => new RefactorFile(file.Path, false)).ToArray(), false, CheckResult.From(analysis).Diagnostics);
            if (current.Bytes().Length > RenamePlanJson.MaximumBytes) throw new ContextFailure("result_limit", "Rename report exceeds 4 MiB; no sources were changed.");
            await workspace.VerifyInputsAsync(token);
            if (preparing) await WritePlan(planPath, RenamePlanJson.Serialize(prepared.Plan), token);
            else
            {
                await SourcePublication.ApplyAsync(prepared.Changes, async check =>
                {
                    await workspace.VerifyInputsAsync(check); SourcePublication.ValidateRegularPath(planPath);
                    if (!(await File.ReadAllBytesAsync(planPath, check)).AsSpan().SequenceEqual(storedBytes))
                        throw new ContextFailure("inputs_changed", "Rename plan changed during application.");
                }, token);
                current = current with { Files = current.Files.Select(file => file with { Written = true }).ToArray(), Diagnostics = [] };
                var refreshed = await WorkspaceLoader.LoadAsync(input with { NoRestore = true }, token);
                var checkedResult = CheckResult.From(WorkspaceAnalysis.Analyze(refreshed, token));
                current = current with { Diagnostics = checkedResult.Diagnostics };
                await SourcePublication.VerifyAsync(prepared.Changes.Select(change => change with { Original = change.Replacement }), token);
                await refreshed.VerifyInputsAsync(token); token.ThrowIfCancellationRequested();
                current = current with { Success = checkedResult.Success, Rechecked = true, Error = checkedResult.Error };
            }
            exit = current.Success ? 0 : 1;
        }
        catch (SourcePublicationFailure error)
        {
            current = (current ?? RefactorReport.Failure("refactor_failed", error.Message)) with { Success = false, Status = "partial", Error = new("refactor_failed", error.Message),
                Files = current?.Files.Select(file => file with { Written = error.Written.Contains(file.Path) }).ToArray() ?? [], RecoveryBackups = error.Backups }; exit = 1;
        }
        catch (OperationCanceledException) when (token.IsCancellationRequested)
        { current = Fail(cancellation.IsCancellationRequested ? "cancelled" : "timeout", cancellation.IsCancellationRequested ? "Refactoring cancelled." : "Refactoring exceeded --timeout."); exit = cancellation.IsCancellationRequested ? 130 : 1; }
        catch (ContextFailure error)
        {
            current = Fail(error.Code, error.Message) with { Error = new(error.Code, error.Message, error.OutputTruncated, error.Candidates) };
            exit = error.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException or JsonException or BadImageFormatException or PlatformNotSupportedException)
        { current = Fail(error is ArgumentException ? "invalid_argument" : "refactor_failed", error.Message); exit = error is ArgumentException ? 2 : 1; }
        return current.Write(operation.Json(result)) ? exit : 1;

        RefactorReport Fail(string code, string message) => (current ?? RefactorReport.Failure(code, message)) with
        { Success = false, Status = current?.Files.Any(file => file.Written) == true ? "applied" : code == "cancelled" ? "cancelled" : "failed", Error = new(code, message) };
    }

    private static async Task WritePlan(string path, byte[] bytes, CancellationToken token)
    {
        var directory = Path.GetDirectoryName(path)!; Directory.CreateDirectory(directory); SourcePublication.ValidateRegularPath(directory);
        var temporary = Path.Combine(directory, ".fpas-plan-" + Guid.NewGuid().ToString("N"));
        try { await File.WriteAllBytesAsync(temporary, bytes, token); token.ThrowIfCancellationRequested(); File.Move(temporary, path, overwrite: false); }
        finally { File.Delete(temporary); }
    }
}

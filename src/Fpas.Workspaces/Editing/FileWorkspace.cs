using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Editing;

/// <summary>File-backed adapter for common checked editing workflows and publication/recheck state.</summary>
public sealed class FileWorkspace
{
    private readonly ContextOptions input;
    public WorkspaceContext Context { get; private set; }
    public WorkspaceAnalysis? Analysis { get; private set; }
    public IReadOnlyList<string> Written { get; private set; } = [];
    public IReadOnlyList<string> RecoveryBackups { get; private set; } = [];
    public bool Rechecked { get; private set; }

    private FileWorkspace(ContextOptions input, WorkspaceContext context, WorkspaceAnalysis analysis)
    { this.input = input; Context = context; Analysis = analysis; }

    public static async Task<FileWorkspace> OpenAsync(ContextOptions input, CancellationToken token = default)
    {
        var context = await WorkspaceLoader.LoadAsync(input, token);
        return new(input, context, WorkspaceAnalysis.Analyze(context, token));
    }

    public DiagnosticFixPlan Corrections(CancellationToken token = default) => DiagnosticFixPlan.Create(Checked(), token);
    public PreparedRename Rename(SourceSelection selection, string name, CancellationToken token = default)
        => RenameService.Prepare(Checked(), selection, name, token);
    public PreparedRename ValidateRename(RenamePlan plan, CancellationToken token = default)
        => RenameService.Validate(Checked(), plan, token);

    public async Task VerifyAsync(CancellationToken token = default)
    { await Context.VerifyInputsAsync(token); token.ThrowIfCancellationRequested(); }

    public async Task ApplyAsync(TextEditSet edits, Func<CancellationToken, Task>? verifyPublicationInput = null, CancellationToken token = default)
    {
        if (Written.Count != 0) throw new InvalidOperationException("An editing operation has already published changes.");
        foreach (var document in edits.Documents)
            if (Context.Texts.Document(document.Original.Path) is var captured &&
                (captured.ContentHash != document.Original.ContentHash || captured.Version != document.Original.Version))
                throw new ContextFailure("inputs_changed", "Edits do not match the captured workspace: " + document.Original.Path);
        var changes = edits.ToSourceChanges();
        try
        {
            await SourcePublication.ApplyAsync(changes, async check =>
            { await VerifyAsync(check); if (verifyPublicationInput is not null) await verifyPublicationInput(check); }, token);
            Written = Array.AsReadOnly(changes.Select(change => change.Path).ToArray());
            Analysis = null;
            Context = await WorkspaceLoader.LoadAsync(input with { NoRestore = true }, token);
            Analysis = WorkspaceAnalysis.Analyze(Context, token);
            await SourcePublication.VerifyAsync(changes.Select(change => change with { Original = change.Replacement }), token);
            await VerifyAsync(token); Rechecked = true;
        }
        catch (SourcePublicationFailure failure)
        {
            Written = Array.AsReadOnly(failure.Written.ToArray()); RecoveryBackups = Array.AsReadOnly(failure.Backups.ToArray());
            if (Written.Count != 0) Analysis = null;
            throw;
        }
        catch
        {
            if (Written.Count != 0) Analysis = null;
            throw;
        }
    }

    private WorkspaceAnalysis Checked() => Analysis ?? throw new InvalidOperationException("Reload and check the workspace before preparing another edit.");
}

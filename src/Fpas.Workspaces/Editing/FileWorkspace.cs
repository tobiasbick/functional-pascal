using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Editing;

/// <summary>File-backed adapter for common checked editing workflows and publication/recheck state.</summary>
public sealed class FileWorkspace
{
    private readonly ContextOptions input;
    private int applying;
    /// <summary>Gets the currently captured context, refreshed after publication.</summary>
    public WorkspaceContext Context { get; private set; }
    /// <summary>Gets current analysis, or null when publication invalidated it and reanalysis failed.</summary>
    public WorkspaceAnalysis? Analysis { get; private set; }
    /// <summary>Gets source paths whose replacements remain published.</summary>
    public IReadOnlyList<string> Written { get; private set; } = [];
    /// <summary>Gets retained backup or staging paths after incomplete recovery.</summary>
    public IReadOnlyList<string> RecoveryBackups { get; private set; } = [];
    /// <summary>Gets whether post-publication reload, analysis and input verification completed.</summary>
    public bool Rechecked { get; private set; }

    private FileWorkspace(ContextOptions input, WorkspaceContext context, WorkspaceAnalysis analysis)
    { this.input = input; Context = context; Analysis = analysis; }

    /// <summary>Loads explicit project inputs and analyzes them without publishing edits.</summary>
    /// <param name="input">Context and preparation policy to retain for rechecks.</param>
    /// <param name="token">Cancels preparation and analysis.</param>
    /// <returns>A workspace containing current analysis, including compilation errors.</returns>
    public static async Task<FileWorkspace> OpenAsync(ContextOptions input, CancellationToken token = default)
    {
        var context = await WorkspaceLoader.LoadAsync(input, token);
        return new(input, context, WorkspaceAnalysis.Analyze(context, token));
    }

    /// <summary>Collects automatic corrections and review-only proposals from current analysis.</summary>
    /// <param name="token">Cancels proposal validation.</param>
    /// <returns>A detached correction plan.</returns>
    public DiagnosticFixPlan Corrections(CancellationToken token = default) => DiagnosticFixPlan.Create(Checked(), token);
    /// <summary>Prepares and semantically validates a source rename.</summary>
    /// <param name="selection">Checked declaration or reference position.</param>
    /// <param name="name">New unescaped simple identifier.</param>
    /// <param name="token">Cancels rename planning.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring Rename(SourceSelection selection, string name, CancellationToken token = default)
        => RenameService.Prepare(Checked(), selection, name, token);
    /// <summary>Prepares removal of an unused parameter and the corresponding direct-call arguments.</summary>
    /// <param name="selection">Checked parameter declaration or named argument.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring RemoveParameter(SourceSelection selection, CancellationToken token = default)
        => FileRefactorPlans.RemoveParameter(Checked(), selection, token);
    /// <summary>Prepares insertion of verified parameter labels at one call.</summary>
    /// <param name="selection">Position within the selected call and optional project owner.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>The checked plan and detached insertion edits.</returns>
    public PreparedFileRefactoring NameArguments(SourceSelection selection, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), "name-arguments", selection, token);
    /// <summary>Regenerates a stored refactoring against the current context and snapshot.</summary>
    /// <param name="plan">Original checked plan to validate.</param>
    /// <param name="token">Cancels semantic revalidation.</param>
    /// <returns>The regenerated matching plan and edits.</returns>
    public PreparedFileRefactoring ValidateRefactoring(RefactorPlan plan, CancellationToken token = default)
        => FileRefactorPlans.Validate(Checked(), plan, token);

    /// <summary>Rechecks disk bytes and evaluated context inputs before answers or edits are published.</summary>
    /// <param name="token">Cancels input verification.</param>
    /// <returns>A task completing when inputs still match.</returns>
    public async Task VerifyAsync(CancellationToken token = default)
    { await Context.VerifyInputsAsync(token); token.ThrowIfCancellationRequested(); }

    /// <summary>Publishes one checked edit batch, then reloads, analyzes and verifies current inputs. Concurrent publications are rejected.</summary>
    /// <param name="edits">Edits matching this workspace's captured texts and versions.</param>
    /// <param name="verifyPublicationInput">Optional final verification of adapter-owned inputs such as a plan file.</param>
    /// <param name="token">Cancels publication or reanalysis; written/recovery state remains observable.</param>
    /// <returns>A task completing after the post-publication check; inspect analysis for remaining errors.</returns>
    public async Task ApplyAsync(TextEditSet edits, Func<CancellationToken, Task>? verifyPublicationInput = null, CancellationToken token = default)
    {
        if (Interlocked.Exchange(ref applying, 1) != 0) throw new InvalidOperationException("An editing operation is already publishing changes.");
        try { await ApplyBatchAsync(edits, verifyPublicationInput, token); }
        finally { Volatile.Write(ref applying, 0); }
    }

    private async Task ApplyBatchAsync(TextEditSet edits, Func<CancellationToken, Task>? verifyPublicationInput, CancellationToken token)
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

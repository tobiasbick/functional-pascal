// [Documentation](../../../docs/architecture/workspaces.md#source-edits)
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Snapshots;

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

    private FileWorkspace(ContextOptions input, WorkspaceContext context, WorkspaceAnalysis? analysis)
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

    /// <summary>Captures original filesystem inputs for planning or applying a checked draft.</summary>
    /// <param name="input">Context path and configuration. Draft dependencies are analyzed from sources without restore.</param>
    /// <param name="token">Cancels input capture.</param>
    /// <param name="creations">Optional files whose prospective ownership must be evaluated.</param>
    /// <returns>A workspace bound to the original files, which may have compilation errors.</returns>
    public static async Task<FileWorkspace> OpenDraftAsync(ContextOptions input, CancellationToken token = default,
        IReadOnlyList<DraftCreation>? creations = null)
    {
        input = input with { NoRestore = true, Preparation = ContextPreparation.SourceSnapshots };
        var paths = (creations ?? []).Select(file => file.Path).ToArray();
        if (paths.Distinct(WorkspaceTextSnapshot.Paths).Count() != paths.Length || (creations ?? []).Any(file =>
                !Path.IsPathFullyQualified(file.Path) || Path.GetFullPath(file.Path) != file.Path ||
                !Path.GetExtension(file.Path).Equals(".fpas", StringComparison.OrdinalIgnoreCase) ||
                !Path.IsPathFullyQualified(file.Project) || !Path.GetExtension(file.Project).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase)))
            throw new ContextFailure("invalid_argument", "Supply unique normalized .fpas paths and absolute .fpasproj owners for draft creations.");
        return new(input, await WorkspaceLoader.LoadAsync(input, paths, token), null);
    }

    /// <summary>Validates full-text draft replacements together and prepares a portable edit plan.</summary>
    /// <param name="replacements">Proposed replacement texts keyed by owned FPAS source path.</param>
    /// <param name="creations">New files paired with their declared owners.</param>
    /// <param name="token">Cancels draft analysis and planning.</param>
    /// <returns>A plan bound to captured original and checked replacement texts.</returns>
    public PreparedFileRefactoring PrepareDraft(IReadOnlyDictionary<string, string> replacements, IReadOnlyList<DraftCreation> creations,
        CancellationToken token = default)
        => FileRefactorPlans.Draft(Context, replacements, creations, token);

    /// <summary>Analyzes a draft with its replacement and prospective source texts.</summary>
    /// <param name="replacements">Proposed replacement texts keyed by owned source path.</param>
    /// <param name="creations">New files paired with their declared owners.</param>
    /// <param name="token">Cancels semantic validation.</param>
    /// <returns>The checked draft, including source diagnostics.</returns>
    public WorkspaceAnalysis AnalyzeDraft(IReadOnlyDictionary<string, string> replacements, IReadOnlyList<DraftCreation> creations,
        CancellationToken token = default)
        => DraftAnalysis.Analyze(Context, replacements, creations, token);

    /// <summary>Collects automatic corrections and review-only proposals from current analysis.</summary>
    /// <param name="token">Cancels proposal validation.</param>
    /// <param name="ids">Optional proposed correction IDs to select.</param>
    /// <returns>A detached correction plan.</returns>
    public DiagnosticFixPlan Corrections(CancellationToken token = default, IReadOnlyList<string>? ids = null)
        => DiagnosticFixPlan.Create(Checked(), token, ids);
    /// <summary>Prepares and semantically validates a source rename.</summary>
    /// <param name="selection">Checked declaration or reference position.</param>
    /// <param name="name">New unescaped simple identifier.</param>
    /// <param name="token">Cancels rename planning.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring Rename(SourceSelection selection, string name, CancellationToken token = default)
        => RenameService.Prepare(Checked(), selection, name, token);
    /// <summary>Prepares insertion of an explicitly typed parameter and a value expression at each direct call.</summary>
    /// <param name="selection">Checked routine declaration or call.</param>
    /// <param name="parameter">Explicit name, FPAS type and call argument.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <param name="closedContext">Confirms that all public signature consumers are in the context.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring AddParameter(SourceSelection selection, RefactorParameter parameter, CancellationToken token = default,
        bool closedContext = false)
        => FileRefactorPlans.Prepare(Checked(), new("add-parameter", selection, selection.Line, selection.Column,
            parameter.Name, parameter.Type, parameter.Argument, ClosedContext: closedContext), token);
    /// <summary>Prepares removal of an unused parameter and the corresponding direct-call arguments.</summary>
    /// <param name="selection">Checked parameter declaration or named argument.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <param name="closedContext">Confirms that all public signature consumers are in the context.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring RemoveParameter(SourceSelection selection, CancellationToken token = default, bool closedContext = false)
        => FileRefactorPlans.RemoveParameter(Checked(), selection, token, closedContext);
    /// <summary>Prepares a checked type or mode change to a routine parameter.</summary>
    /// <param name="selection">Parameter declaration, reference or argument label.</param>
    /// <param name="change">Requested type, mode and explicit replacements.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <param name="closedContext">Confirms that all public signature consumers are in the context.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring ChangeSignature(SourceSelection selection, RefactorSignatureChange change, CancellationToken token = default,
        bool closedContext = false)
        => FileRefactorPlans.Prepare(Checked(), new("change-signature", selection, selection.Line, selection.Column,
            SignatureChange: change, ClosedContext: closedContext), token);
    /// <summary>Prepares checked conformance to a protocol or CLR interface on a record.</summary>
    /// <param name="selection">Record declaration or reference.</param>
    /// <param name="protocol">Protocol type and explicit implementations of missing members.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring ImplementProtocol(SourceSelection selection, RefactorProtocol protocol, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("implement-protocol", selection, selection.Line, selection.Column,
            Protocol: protocol), token);
    /// <summary>Prepares insertion of verified parameter labels at one call.</summary>
    /// <param name="selection">Position within the selected call and optional project owner.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>The checked plan and detached insertion edits.</returns>
    public PreparedFileRefactoring NameArguments(SourceSelection selection, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("name-arguments", selection, selection.Line, selection.Column), token);
    /// <summary>Prepares extraction of the selected expression to a typed local constant.</summary>
    /// <param name="selection">Start of the expression and optional project owner.</param>
    /// <param name="end">Exclusive end of the expression.</param>
    /// <param name="name">Unescaped identifier for the new constant.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring ExtractVariable(SourceSelection selection, RefactorPosition end, string name, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("extract-variable", selection, end.Line, end.Column, name), token);
    /// <summary>Prepares extracting complete statements into a checked nested routine.</summary>
    /// <param name="selection">Start of the statement range and optional project owner.</param>
    /// <param name="end">Exclusive end of the range.</param>
    /// <param name="name">Unescaped name of the nested routine.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring ExtractRoutine(SourceSelection selection, RefactorPosition end, string name, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("extract-routine", selection, end.Line, end.Column, name), token);
    /// <summary>Prepares replacing the reads of a local binding with its initializer.</summary>
    /// <param name="selection">The local declaration or a read of it.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring InlineVariable(SourceSelection selection, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("inline-variable", selection, selection.Line, selection.Column), token);
    /// <summary>Prepares removal of safely unused CLR imports from a source file.</summary>
    /// <param name="file">Source file with the imports.</param>
    /// <param name="project">Optional owning project for a linked file.</param>
    /// <param name="token">Cancels planning and overlay verification.</param>
    /// <returns>A checked plan and detached edits.</returns>
    public PreparedFileRefactoring OrganizeImports(string file, string? project, CancellationToken token = default)
        => FileRefactorPlans.Prepare(Checked(), new("organize-imports", new(file, 1, 1, project), 1, 1), token);
    /// <summary>Regenerates a stored refactoring against the current context and snapshot.</summary>
    /// <param name="plan">Original checked plan to validate.</param>
    /// <param name="token">Cancels semantic revalidation.</param>
    /// <returns>The regenerated matching plan and edits.</returns>
    public PreparedFileRefactoring ValidateRefactoring(RefactorPlan plan, CancellationToken token = default)
        => plan.Kind == "draft" ? FileRefactorPlans.ValidateDraft(Context, plan, token) : FileRefactorPlans.Validate(Checked(), plan, token);

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
    public Task ApplyAsync(TextEditSet edits, Func<CancellationToken, Task>? verifyPublicationInput = null, CancellationToken token = default)
        => ApplyAsync(edits, null, verifyPublicationInput, token);

    /// <summary>Publishes a prepared refactoring and verifies that any added sources match the checked draft.</summary>
    /// <param name="prepared">The regenerated edit plan and captured source edits.</param>
    /// <param name="verifyPublicationInput">Optional final verification of adapter-owned inputs such as a plan file.</param>
    /// <param name="token">Cancels staging or verification. Rollback proceeds independently.</param>
    /// <returns>A task completing after publication and verification.</returns>
    public Task ApplyAsync(PreparedFileRefactoring prepared, Func<CancellationToken, Task>? verifyPublicationInput = null, CancellationToken token = default)
        => ApplyAsync(prepared.Edits, prepared.Plan, verifyPublicationInput, token);

    private async Task ApplyAsync(TextEditSet edits, RefactorPlan? plan, Func<CancellationToken, Task>? verifyPublicationInput, CancellationToken token)
    {
        if (Interlocked.Exchange(ref applying, 1) != 0) throw new InvalidOperationException("An editing operation is already publishing changes.");
        try { await ApplyBatchAsync(edits, plan, verifyPublicationInput, token); }
        finally { Volatile.Write(ref applying, 0); }
    }

    private async Task ApplyBatchAsync(TextEditSet edits, RefactorPlan? plan, Func<CancellationToken, Task>? verifyPublicationInput, CancellationToken token)
    {
        if (Written.Count != 0) throw new InvalidOperationException("An editing operation has already published changes.");
        foreach (var document in edits.Documents)
            if (Context.Texts.Document(document.Original.Path) is var captured &&
                (captured.ContentHash != document.Original.ContentHash || captured.Version != document.Original.Version))
                throw new ContextFailure("inputs_changed", "Edits do not match the captured workspace: " + document.Original.Path);
        var creations = plan?.Creations() ?? [];
        var changes = edits.ToSourceChanges().Concat(creations.Select(file => new SourceChange(file.Path, null, TextDocumentSnapshot.Utf8.GetBytes(file.Text)))).ToArray();
        (WorkspaceContext Context, WorkspaceAnalysis Analysis)? checkedDraft = null;
        try
        {
            await SourcePublication.ApplyAsync(changes, async check =>
            { await VerifyAsync(check); if (verifyPublicationInput is not null) await verifyPublicationInput(check); }, token,
                verifyPublished: creations.Count == 0 ? null : VerifyPublishedDraft);
            Written = Array.AsReadOnly(changes.Select(change => change.Path).ToArray());
            Analysis = null;
            (Context, Analysis) = checkedDraft ?? await CheckPublished(token);
            Rechecked = true;
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

        async Task<(WorkspaceContext, WorkspaceAnalysis)> CheckPublished(CancellationToken check)
        {
            var current = await WorkspaceLoader.LoadAsync(input with { NoRestore = true }, check);
            var analysis = input.Preparation == ContextPreparation.SourceSnapshots
                ? SourceGraphAnalysis.Analyze(current, current.Texts, new(67_108_864), check)
                : WorkspaceAnalysis.Analyze(current, check);
            await SourcePublication.VerifyAsync(changes.Select(change => change with { Original = change.Replacement }), check);
            await current.VerifyInputsAsync(check);
            return (current, analysis);
        }

        async Task VerifyPublishedDraft(CancellationToken check)
        {
            var verified = await CheckPublished(check);
            if (!verified.Item2.Success || !verified.Item2.IsComplete || verified.Item2.Context.SnapshotId != plan!.DraftSnapshotId)
                throw new ContextFailure("inputs_changed", "Published sources or evaluated ownership differ from the checked draft.");
            checkedDraft = verified;
        }
    }

    private WorkspaceAnalysis Checked() => Analysis ?? throw new InvalidOperationException("Reload and check the workspace before preparing another edit.");
}

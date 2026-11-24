// [Documentation](../../../docs/pascal/tools/refactoring.md#validation-and-plans)
using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Snapshots;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

/// <summary>Pairs a portable checked refactoring plan with its detached edits.</summary>
/// <param name="Plan">Versioned plan and its semantic/input identity.</param>
/// <param name="Edits">Validated source changes.</param>
public sealed record PreparedFileRefactoring(RefactorPlan Plan, TextEditSet Edits);

/// <summary>Creates and revalidates portable refactoring plans against the original project inputs.</summary>
public static class FileRefactorPlans
{
    internal static PreparedFileRefactoring Seal(RefactorPlan plan, TextEditSet edits)
    {
        plan = plan with { Id = RefactorPlanJson.Identity(plan) };
        if (RefactorPlanJson.Serialize(plan).Length > RefactorPlanJson.MaximumBytes)
            throw new ContextFailure(plan.Kind == "rename" ? "rename_limit" : "result_limit", "Refactoring plan exceeds 4 MiB.");
        return new(plan, edits);
    }

    /// <summary>Prepares the same checked parameter removal used by editor clients for file publication.</summary>
    /// <param name="analysis">Checked project or solution context.</param>
    /// <param name="selection">One-based parameter selection and optional project owner.</param>
    /// <param name="token">Cancels plan preparation.</param>
    /// <param name="closedContext">Confirms that all public signature consumers are in the context.</param>
    /// <returns>The portable plan and its validated source edits.</returns>
    public static PreparedFileRefactoring RemoveParameter(WorkspaceAnalysis analysis, SourceSelection selection, CancellationToken token,
        bool closedContext = false)
        => Prepare(analysis, new("remove-parameter", selection, selection.Line, selection.Column, ClosedContext: closedContext), token);

    internal static PreparedFileRefactoring Prepare(WorkspaceAnalysis analysis, RefactoringRequest request, CancellationToken token)
    {
        var selection = request.Selection;
        selection = selection with { File = Path.GetFullPath(selection.File), Project = selection.Project is null ? null : Path.GetFullPath(selection.Project) };
        var prepared = CheckedRefactoring.Prepare(analysis, request with { Selection = selection }, token);
        var files = prepared.Edits.Documents.Select(document => new RefactorPlanFile(document.Original.Path, document.OriginalHash,
            document.ReplacementHash, document.Edits)).ToArray();
        var extraction = request.Kind is "extract-variable" or "extract-routine";
        var plan = new RefactorPlan(1, request.Kind, analysis.Context.Path, analysis.Context.Configuration, prepared.SnapshotId,
            selection, extraction ? request.Name : null, prepared.Targets, files,
            analysis.Projects.Select(project => project.Project).Order(StringComparer.Ordinal).ToArray(), prepared.BinaryBoundaries)
        {
            AddedParameter = request.Kind == "add-parameter" ? new(request.Name!, request.Type!, request.Argument!) : null,
            SelectionEnd = extraction ? new(request.EndLine, request.EndColumn) : null,
            Notes = request.Kind is "extract-variable" or "extract-routine" or "inline-variable" or "organize-imports" or "change-signature" or "implement-protocol" || prepared.Targets.Any(target => target.PublicApi)
                ? prepared.Notes : null,
            SignatureChange = request.SignatureChange is { } change ? change with
            {
                Arguments = change.Arguments.Select(argument => argument.File is null ? argument :
                    argument with { File = Path.GetFullPath(argument.File) }).ToArray()
            } : null,
            ClosedContext = request.ClosedContext,
            Protocol = request.Protocol
        };
        return Seal(plan, prepared.Edits);
    }

    internal static PreparedFileRefactoring Draft(WorkspaceContext context, IReadOnlyDictionary<string, string> replacements,
        IReadOnlyList<DraftCreation> creations, CancellationToken token)
    {
        var analyzed = DraftAnalysis.Analyze(context, replacements, creations, token);
        if (!analyzed.Success || !analyzed.IsComplete)
            throw new ContextFailure("draft_invalid", "Proposed sources fail dependency or semantic validation: " +
                string.Join("; ", analyzed.Projects.SelectMany(project => project.Result?.Diagnostics ?? []).Take(10).Select(diagnostic => diagnostic.Code + ": " + diagnostic.Message)));
        var edits = new List<DiagnosticEdit>();
        foreach (var (file, replacement) in replacements)
        {
            token.ThrowIfCancellationRequested();
            var original = context.Texts.Document(file);
            if (original.Text != replacement)
                edits.Add(new(new(original.Path, 1, 1, original.Text.Length, Offset: 0), original.Text, replacement));
        }
        var changes = TextEditSet.Create(context.Texts, edits, token);
        var files = changes.Documents.Select(document => new RefactorPlanFile(document.Original.Path, document.OriginalHash,
            document.ReplacementHash, document.Edits)).Concat(creations.Select(file => new RefactorPlanFile(file.Path, null,
                Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(TextDocumentSnapshot.Utf8.GetBytes(file.Text))), [], file.Project, file.Text)))
            .OrderBy(file => file.Path, StringComparer.Ordinal).ToArray();
        return Seal(new RefactorPlan(1, "draft", context.Path, context.Configuration, context.SnapshotId, null, null, [], files,
            analyzed.Projects.Select(project => project.Project).Order(StringComparer.Ordinal).ToArray(), [])
            { DraftSnapshotId = analyzed.Context.SnapshotId }, changes);
    }

    internal static PreparedFileRefactoring ValidateDraft(WorkspaceContext context, RefactorPlan plan, CancellationToken token)
    {
        VerifyInputs(context, plan);
        var changes = TextEditSet.Create(context.Texts, plan.Files.SelectMany(file => file.Edits), token);
        var drafts = changes.Documents.ToDictionary(document => document.Original.Path, document => document.NewText, WorkspaceTextSnapshot.Paths);
        return RequireMatchingPlan(plan, Draft(context, drafts, plan.Creations(), token));
    }

    /// <summary>Regenerates a plan and requires its context, input snapshot and identity to match exactly.</summary>
    /// <param name="analysis">Current successful complete analysis.</param>
    /// <param name="plan">Original checked refactoring plan.</param>
    /// <param name="token">Cancels semantic regeneration.</param>
    /// <returns>The matching regenerated plan and edits.</returns>
    public static PreparedFileRefactoring Validate(WorkspaceAnalysis analysis, RefactorPlan plan, CancellationToken token = default)
    {
        if (plan.Kind == "draft")
            throw new ContextFailure("invalid_plan", "Draft plans require a captured draft workspace.");
        VerifyInputs(analysis.Context, plan);
        var selection = plan.Selection!;
        var prepared = plan.Kind == "rename" ? RenameService.Prepare(analysis, selection, plan.NewName!, token)
            : Prepare(analysis, new(plan.Kind, selection,
                plan.SelectionEnd?.Line ?? selection.Line, plan.SelectionEnd?.Column ?? selection.Column,
                plan.AddedParameter?.Name ?? plan.NewName, plan.AddedParameter?.Type, plan.AddedParameter?.Argument,
                plan.SignatureChange, plan.ClosedContext, plan.Protocol), token);
        return RequireMatchingPlan(plan, prepared);
    }

    private static void VerifyInputs(WorkspaceContext context, RefactorPlan plan)
    {
        if (!plan.HasValidOperation || RefactorPlanJson.Identity(plan) != plan.Id)
            throw new ContextFailure("invalid_plan", "Invalid or modified refactoring plan.");
        ValidateContext(plan, context.Path, context.Configuration);
        if (plan.InputSnapshotId != context.SnapshotId)
            throw new ContextFailure("stale_plan", "Project inputs changed after refactoring planning. Prepare and review a new plan.");
    }

    private static PreparedFileRefactoring RequireMatchingPlan(RefactorPlan plan, PreparedFileRefactoring prepared)
    {
        if (prepared.Plan.Id != plan.Id)
            throw new ContextFailure("invalid_plan", "Plan does not match the regenerated checked refactoring.");
        return prepared;
    }

    /// <summary>Rejects applying a plan to another context or configuration.</summary>
    /// <param name="plan">Plan containing the original context.</param>
    /// <param name="context">Requested project or solution path.</param>
    /// <param name="configuration">Requested context configuration.</param>
    public static void ValidateContext(RefactorPlan plan, string context, string configuration)
    {
        if (plan.Context != Path.GetFullPath(context) || plan.Configuration != configuration)
            throw new ContextFailure("plan_context_mismatch", "Supply the plan's original context and configuration.");
    }
}

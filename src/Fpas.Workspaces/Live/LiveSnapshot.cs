using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Live;

/// <summary>Contains syntax diagnostics for an open file outside the project source set.</summary>
/// <param name="Document">Captured open buffer.</param>
/// <param name="Diagnostics">Syntax-only diagnostics for that buffer.</param>
public sealed record UnownedDocument(TextDocumentSnapshot Document, IReadOnlyList<CompilerDiagnostic> Diagnostics);

/// <summary>Detached results for a specific session revision; verify before publishing.</summary>
public sealed class LiveSnapshot
{
    internal object Owner { get; }
    internal string InputIdentity { get; }
    /// <summary>Gets the workspace revision captured by this result.</summary>
    public long Revision { get; }
    /// <summary>Gets detached project analysis, including failures and coverage.</summary>
    public WorkspaceAnalysis Analysis { get; }
    /// <summary>Gets owned source texts and unowned open buffers.</summary>
    public WorkspaceTextSnapshot Documents { get; }
    /// <summary>Gets syntax-only results for unowned open buffers.</summary>
    public IReadOnlyList<UnownedDocument> Unowned { get; }
    /// <summary>Gets the number of project analyses reused from the session cache.</summary>
    public int ReusedProjects { get; }
    /// <summary>Gets accounted bytes retained by this result and the remaining cache.</summary>
    public long RetainedBytes { get; }

    internal LiveSnapshot(object owner, long revision, string identity, WorkspaceAnalysis analysis, WorkspaceTextSnapshot documents,
        UnownedDocument[] unowned, int reused, long retained)
    { Owner = owner; Revision = revision; InputIdentity = identity; Analysis = analysis; Documents = documents; Unowned = Array.AsReadOnly(unowned); ReusedProjects = reused; RetainedBytes = retained; }

    /// <summary>Creates queries over this result's checked semantic regions.</summary>
    /// <returns>A query service tied to this analysis.</returns>
    public SourceQuery Queries() => new(Analysis);
    /// <summary>Prepares a rename without publishing source changes.</summary>
    /// <param name="selection">Checked declaration or reference position.</param>
    /// <param name="name">New unescaped simple identifier.</param>
    /// <param name="token">Cancels planning and validation.</param>
    /// <returns>The checked plan and detached edits.</returns>
    public PreparedFileRefactoring Rename(SourceSelection selection, string name, CancellationToken token = default) => RenameService.Prepare(Analysis, selection, name, token);
    /// <summary>Collects automatic corrections and review-only proposals.</summary>
    /// <param name="token">Cancels proposal validation.</param>
    /// <returns>A detached correction plan.</returns>
    public DiagnosticFixPlan Corrections(CancellationToken token = default) => DiagnosticFixPlan.Create(Analysis, token);
}

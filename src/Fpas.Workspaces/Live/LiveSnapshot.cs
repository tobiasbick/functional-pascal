using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Live;

public sealed record UnownedDocument(TextDocumentSnapshot Document, IReadOnlyList<CompilerDiagnostic> Diagnostics);

/// <summary>Detached results for a specific session revision; verify before publishing.</summary>
public sealed class LiveSnapshot
{
    internal object Owner { get; }
    internal string InputIdentity { get; }
    public long Revision { get; }
    public WorkspaceAnalysis Analysis { get; }
    public WorkspaceTextSnapshot Documents { get; }
    public IReadOnlyList<UnownedDocument> Unowned { get; }
    public int ReusedProjects { get; }
    public long RetainedBytes { get; }

    internal LiveSnapshot(object owner, long revision, string identity, WorkspaceAnalysis analysis, WorkspaceTextSnapshot documents,
        UnownedDocument[] unowned, int reused, long retained)
    { Owner = owner; Revision = revision; InputIdentity = identity; Analysis = analysis; Documents = documents; Unowned = Array.AsReadOnly(unowned); ReusedProjects = reused; RetainedBytes = retained; }

    public SourceQuery Queries() => new(Analysis);
    public PreparedRename Rename(SourceSelection selection, string name, CancellationToken token = default) => RenameService.Prepare(Analysis, selection, name, token);
    public DiagnosticFixPlan Corrections(CancellationToken token = default) => DiagnosticFixPlan.Create(Analysis, token);
}

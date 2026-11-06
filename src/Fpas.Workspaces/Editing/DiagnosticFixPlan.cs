using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Editing;

public sealed record FixSuggestion(string Project, string Code, DiagnosticFix Fix);
public sealed record FixFile(string Path, string OriginalHash, string ReplacementHash, int EditCount, bool Written = false);

public sealed class DiagnosticFixPlan
{
    public IReadOnlyList<FixSuggestion> Suggestions { get; }
    public IReadOnlyList<FixFile> Files { get; }
    public TextEditSet Edits { get; }
    public int EditCount => Edits.EditCount;

    private DiagnosticFixPlan(FixSuggestion[] suggestions, TextEditSet edits)
    {
        Suggestions = Array.AsReadOnly(suggestions); Edits = edits;
        Files = Array.AsReadOnly(edits.Documents.Select(document => new FixFile(document.Original.Path,
            document.OriginalHash, document.ReplacementHash, document.Edits.Count)).ToArray());
    }

    public static DiagnosticFixPlan Create(WorkspaceAnalysis analysis, CancellationToken token = default)
    {
        if (analysis.Context.Issues.Count != 0 || analysis.Projects.Any(project => project.Result is null || project.Result.Diagnostics.Any(diagnostic => diagnostic.Code == "FP0003")))
            throw new ContextFailure("fix_incomplete", "Automatic fixes require captured inputs for every selected project and diagnostics below the error limit.");
        var suggestions = analysis.Projects.SelectMany(project => project.Result!.Diagnostics.SelectMany(diagnostic =>
            diagnostic.Fixes.Select(fix => new FixSuggestion(project.Project, diagnostic.Code, fix)))).ToArray();
        return Create(analysis.Context.Texts, suggestions, token);
    }

    public static DiagnosticFixPlan Create(WorkspaceTextSnapshot texts, IEnumerable<FixSuggestion> suggestions, CancellationToken token = default)
    {
        token.ThrowIfCancellationRequested();
        var captured = suggestions.Select(suggestion => suggestion with
            { Fix = suggestion.Fix with { Edits = Array.AsReadOnly(suggestion.Fix.Edits.ToArray()) } }).ToArray();
        if (captured.Sum(suggestion => (long)suggestion.Fix.Edits.Count) > 1000)
            throw new ContextFailure("fix_limit", "A correction batch exceeds 1,000 proposed edits.");
        var automatic = captured.Where(suggestion => suggestion.Fix.Applicability == DiagnosticApplicability.Automatic).ToArray();
        if (automatic.Any(suggestion => suggestion.Fix.Edits.Count == 0)) throw new ContextFailure("invalid_fix", "An automatic correction contains no edits.");
        return new(captured, TextEditSet.Create(texts, automatic.SelectMany(suggestion => suggestion.Fix.Edits), token));
    }
}

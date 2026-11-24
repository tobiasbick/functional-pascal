// [Documentation](../../../docs/pascal/tools/diagnostics.md#apply-automatic-corrections)
using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Editing;

/// <summary>Associates a compiler correction proposal with its owning diagnostic.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Code">Diagnostic code.</param>
/// <param name="Fix">Automatic or review-only proposal.</param>
public sealed record FixSuggestion(string Project, string Code, DiagnosticFix Fix)
{
    /// <summary>Gets the deterministic identity of this proposal and its owning diagnostic.</summary>
    public string Id { get; init; } = "";
}
/// <summary>Describes a changed file and its publication state.</summary>
/// <param name="Path">Source path.</param>
/// <param name="OriginalHash">SHA-256 hash of original UTF-8 bytes.</param>
/// <param name="ReplacementHash">SHA-256 hash of replacement UTF-8 bytes.</param>
/// <param name="EditCount">Number of deduplicated edits.</param>
/// <param name="Written">Whether an adapter has confirmed publication.</param>
public sealed record FixFile(string Path, string OriginalHash, string ReplacementHash, int EditCount, bool Written = false);

/// <summary>Retains all correction proposals and validates only automatic edits.</summary>
public sealed class DiagnosticFixPlan
{
    /// <summary>Gets detached automatic and review-only proposals.</summary>
    public IReadOnlyList<FixSuggestion> Suggestions { get; }
    /// <summary>Gets changed-file identities, initially marked unwritten.</summary>
    public IReadOnlyList<FixFile> Files { get; }
    /// <summary>Gets validated automatic edits; review proposals are excluded.</summary>
    public TextEditSet Edits { get; }
    /// <summary>Gets the number of automatic edits in changed documents.</summary>
    public int EditCount => Edits.EditCount;

    private DiagnosticFixPlan(FixSuggestion[] suggestions, TextEditSet edits)
    {
        Suggestions = Array.AsReadOnly(suggestions); Edits = edits;
        Files = Array.AsReadOnly(edits.Documents.Select(document => new FixFile(document.Original.Path,
            document.OriginalHash, document.ReplacementHash, document.Edits.Count)).ToArray());
    }

    /// <summary>Collects proposals when every selected project has captured diagnostics below the error limit.</summary>
    /// <param name="analysis">Analysis supplying proposals and source texts.</param>
    /// <param name="token">Cancels edit validation.</param>
    /// <param name="ids">Optional proposed correction IDs to select.</param>
    /// <returns>A detached correction plan, possibly with no automatic changes.</returns>
    public static DiagnosticFixPlan Create(WorkspaceAnalysis analysis, CancellationToken token = default, IReadOnlyList<string>? ids = null)
    {
        if (analysis.Context.Issues.Count != 0 || analysis.Projects.Any(project => project.Result is null || project.Result.Diagnostics.Any(diagnostic => diagnostic.Code == "FP0003")))
            throw new ContextFailure("fix_incomplete", "Automatic fixes require captured inputs for every selected project and diagnostics below the error limit.");
        var suggestions = analysis.Projects.SelectMany(project => project.Result!.Diagnostics.SelectMany(diagnostic =>
            diagnostic.Fixes.Select(fix => new FixSuggestion(project.Project, diagnostic.Code, fix)))).ToArray();
        return Create(analysis.Context.Texts, suggestions, token, ids);
    }

    /// <summary>Validates automatic proposals against explicit captured texts.</summary>
    /// <param name="texts">Authoritative original texts.</param>
    /// <param name="suggestions">Proposals to detach and retain, including review-only proposals.</param>
    /// <param name="token">Cancels edit validation.</param>
    /// <param name="ids">Optional proposed correction IDs to select.</param>
    /// <returns>A plan with conflicting or invalid automatic edits rejected.</returns>
    public static DiagnosticFixPlan Create(WorkspaceTextSnapshot texts, IEnumerable<FixSuggestion> suggestions, CancellationToken token = default,
        IReadOnlyList<string>? ids = null)
    {
        token.ThrowIfCancellationRequested();
        var requested = ids is { Count: > 0 } ? ids.ToHashSet(StringComparer.OrdinalIgnoreCase) : null;
        var selected = new Dictionary<string, FixSuggestion>(StringComparer.Ordinal);
        long editCount = 0;
        foreach (var input in suggestions)
        {
            token.ThrowIfCancellationRequested();
            var suggestion = Identify(input);
            if (requested is not null && !requested.Contains(suggestion.Id)) continue;
            editCount += suggestion.Fix.Edits.Count;
            if (editCount > 1000) throw new ContextFailure("fix_limit", "A correction batch exceeds 1,000 proposed edits.");
            selected.TryAdd(suggestion.Id, suggestion);
        }
        var captured = selected.Values.ToArray();
        if (requested is not null)
        {
            requested.ExceptWith(selected.Keys);
            if (requested.Count != 0)
                throw new ContextFailure("invalid_argument", "Unknown correction ID. Preview the current fixes before selecting IDs.",
                    candidates: requested.Order(StringComparer.Ordinal).ToArray());
            if (captured.Any(suggestion => suggestion.Fix.Applicability != DiagnosticApplicability.Automatic))
                throw new ContextFailure("invalid_argument", "Only automatic corrections can be selected with --fix-id.");
        }
        var automatic = captured.Where(suggestion => suggestion.Fix.Applicability == DiagnosticApplicability.Automatic).ToArray();
        if (automatic.Any(suggestion => suggestion.Fix.Edits.Count == 0)) throw new ContextFailure("invalid_fix", "An automatic correction contains no edits.");
        return new(captured, TextEditSet.Create(texts, automatic.SelectMany(suggestion => suggestion.Fix.Edits), token));
    }
    private static FixSuggestion Identify(FixSuggestion suggestion)
    {
        var copy = suggestion with { Id = "", Fix = suggestion.Fix with { Edits = Array.AsReadOnly(suggestion.Fix.Edits.ToArray()) } };
        return copy with { Id = Convert.ToHexString(SHA256.HashData(JsonSerializer.SerializeToUtf8Bytes(copy, FixSuggestionJson.Default.FixSuggestion))) };
    }
}

[JsonSerializable(typeof(FixSuggestion))]
internal partial class FixSuggestionJson : JsonSerializerContext;

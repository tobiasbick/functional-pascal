namespace Fpas.Compiler.Analysis;

public sealed record SemanticRegion(SourceSpan Span, string State, string? Reason = null);

/// <summary>Partition of captured text into checked and unavailable semantic regions.</summary>
public sealed class SemanticCoverage
{
    private readonly IReadOnlyDictionary<string, int> lengths;
    public IReadOnlyList<SemanticRegion> Regions { get; }
    public bool IsComplete => Regions.All(region => region.State == "checked");

    internal SemanticCoverage(IEnumerable<SourceDocument> documents, IEnumerable<SemanticRegion> invalid)
    {
        var captured = documents.ToArray();
        lengths = captured.ToDictionary(document => document.Path, document => document.Text.Length,
            OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        var result = new List<SemanticRegion>();
        foreach (var document in captured)
        {
            var cursor = 0;
            foreach (var region in invalid.Where(region => SamePath(region.Span.Path, document.Path)).OrderBy(region => region.Span.Offset).ThenByDescending(region => region.Span.Length))
            {
                if (region.Span.Offset < cursor && (long)region.Span.Offset + region.Span.Length <= cursor) continue;
                if (region.Span.Offset > cursor) result.Add(new(document.Span(cursor, region.Span.Offset - cursor), "checked"));
                var start = Math.Max(cursor, region.Span.Offset); var end = Math.Max(start, region.Span.Offset + region.Span.Length);
                if (end > start || region.Span.Length == 0) result.Add(new(document.Span(start, end - start), "unavailable", region.Reason));
                cursor = end;
            }
            if (cursor < document.Text.Length || result.All(region => !SamePath(region.Span.Path, document.Path)))
                result.Add(new(document.Span(cursor, document.Text.Length - cursor), "checked"));
        }
        Regions = Array.AsReadOnly(result.ToArray());
    }

    public bool AvailableAt(string path, int offset) => lengths.TryGetValue(path, out var length) && offset >= 0 && offset <= length &&
        !Regions.Any(region => region.State != "checked" && SamePath(region.Span.Path, path) &&
        (region.Span.Contains(offset) || region.Span.Length == 0 && region.Span.Offset == offset));
    public bool Available(SourceSpan span) => lengths.TryGetValue(span.Path, out var length) && span.Offset >= 0 && span.Length >= 0 && (long)span.Offset + span.Length <= length &&
        (span.Length == 0 ? AvailableAt(span.Path, span.Offset) : !Regions.Any(region => region.State != "checked" && SamePath(region.Span.Path, span.Path) &&
            region.Span.Offset < (long)span.Offset + span.Length && span.Offset < (long)region.Span.Offset + region.Span.Length));
    private static bool SamePath(string a, string b) => string.Equals(a, b, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal);
}

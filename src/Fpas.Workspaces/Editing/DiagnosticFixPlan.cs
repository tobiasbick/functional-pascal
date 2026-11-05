using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Editing;

public sealed record FixSuggestion(string Project, string Code, DiagnosticFix Fix);
public sealed record FixFile(string Path, string OriginalHash, string ReplacementHash, int EditCount, bool Written = false);

public sealed class DiagnosticFixPlan
{
    private static readonly UTF8Encoding Utf8 = new(false, true);
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    public IReadOnlyList<FixSuggestion> Suggestions { get; }
    public IReadOnlyList<FixFile> Files { get; }
    public IReadOnlyList<SourceChange> Changes { get; }
    public int EditCount => Files.Sum(file => file.EditCount);

    private DiagnosticFixPlan(IReadOnlyList<FixSuggestion> suggestions, IReadOnlyList<FixFile> files, IReadOnlyList<SourceChange> changes)
    { Suggestions = suggestions; Files = files; Changes = changes; }

    public static async Task<DiagnosticFixPlan> CreateAsync(WorkspaceAnalysis analysis, CancellationToken token = default)
    {
        if (analysis.Context.Issues.Count != 0 || analysis.Projects.Any(project => project.Result is null || project.Result.Diagnostics.Any(diagnostic => diagnostic.Code == "FP0003")))
            throw new ContextFailure("fix_incomplete", "Automatic fixes require captured inputs for every selected project and diagnostics below the error limit.");
        var sources = new Dictionary<string, string>(Paths);
        foreach (var source in analysis.Context.Projects.SelectMany(project => project.Request.Sources))
        {
            if (sources.TryGetValue(source.Path, out var text) && text != source.Text)
                throw new ContextFailure("edit_conflict", "Linked source owners have different captured contents: " + source.Path);
            sources[source.Path] = source.Text;
        }
        var suggestions = analysis.Projects.SelectMany(project => project.Result!.Diagnostics.SelectMany(diagnostic =>
            diagnostic.Fixes.Select(fix => new FixSuggestion(project.Project, diagnostic.Code, fix)))).ToArray();
        return await CreateAsync(sources.Select(pair => new CompilationSource(pair.Key, pair.Value)).ToArray(), suggestions, token);
    }

    public static async Task<DiagnosticFixPlan> CreateAsync(IReadOnlyList<CompilationSource> sources,
        IReadOnlyList<FixSuggestion> suggestions, CancellationToken token = default)
    {
        token.ThrowIfCancellationRequested();
        if (suggestions.Sum(suggestion => (long)suggestion.Fix.Edits.Count) > 1000)
            throw new ContextFailure("fix_limit", "A correction batch exceeds 1,000 proposed edits.");
        var owners = sources.ToDictionary(source => Path.GetFullPath(source.Path), source => source.Text, Paths);
        var edits = new Dictionary<string, List<DiagnosticEdit>>(Paths);
        foreach (var suggestion in suggestions.Where(suggestion => suggestion.Fix.Applicability == DiagnosticApplicability.Automatic))
        {
            if (suggestion.Fix.Edits.Count == 0) throw new ContextFailure("invalid_fix", "An automatic correction contains no edits.");
            foreach (var edit in suggestion.Fix.Edits)
            {
                token.ThrowIfCancellationRequested(); var location = edit.Location;
                var path = Path.GetFullPath(location.Path);
                if (!owners.TryGetValue(path, out var text) || !Path.GetExtension(path).Equals(".fpas", StringComparison.OrdinalIgnoreCase))
                    throw new ContextFailure("invalid_fix", "A correction is outside the captured FPAS sources: " + path);
                if (location.Offset is not { } offset || offset < 0 || location.Length < 0 || (long)offset + location.Length > text.Length ||
                    location.Length != edit.ExpectedText.Length || !text.AsSpan(offset, location.Length).SequenceEqual(edit.ExpectedText))
                    throw new ContextFailure("invalid_fix", "A correction does not match its captured source range: " + path);
                if (!edits.TryGetValue(path, out var group)) edits[path] = group = [];
                if (!group.Any(previous => previous.Location.Offset == offset && previous.Location.Length == location.Length &&
                    previous.ExpectedText == edit.ExpectedText && previous.NewText == edit.NewText)) group.Add(edit);
            }
        }
        var changes = new List<SourceChange>(); var files = new List<FixFile>(); long bytes = 0;
        foreach (var (path, group) in edits.OrderBy(pair => pair.Key, StringComparer.Ordinal))
        {
            token.ThrowIfCancellationRequested(); SourcePublication.ValidateRegularPath(path);
            var ordered = group.OrderBy(edit => edit.Location.Offset).ThenBy(edit => edit.Location.Length).ToArray();
            for (var index = 1; index < ordered.Length; index++)
            {
                var previous = ordered[index - 1].Location; var current = ordered[index].Location;
                if (current.Offset == previous.Offset || current.Offset < previous.Offset + previous.Length)
                    throw new ContextFailure("edit_conflict", "Automatic corrections overlap or compete at the same position: " + path);
            }
            if (new FileInfo(path).Length > 67_108_864 - bytes) throw new ContextFailure("fix_limit", "Correction inputs exceed 64 MiB.");
            var original = await File.ReadAllBytesAsync(path, token);
            if ((bytes += original.Length) > 67_108_864) throw new ContextFailure("fix_limit", "Correction inputs exceed 64 MiB.");
            if (Utf8.GetString(original) != owners[path]) throw new ContextFailure("inputs_changed", "Source changed while preparing corrections: " + path);
            var replacementSize = original.Length + ordered.Sum(edit => (long)Utf8.GetByteCount(edit.NewText) - Utf8.GetByteCount(edit.ExpectedText));
            if (replacementSize > 67_108_864 - bytes) throw new ContextFailure("fix_limit", "Correction inputs and replacements exceed 64 MiB.");
            var updated = new StringBuilder(owners[path]);
            foreach (var edit in ordered.Reverse()) updated.Remove(edit.Location.Offset!.Value, edit.Location.Length).Insert(edit.Location.Offset.Value, edit.NewText);
            var replacement = Utf8.GetBytes(updated.ToString());
            if ((bytes += replacement.Length) > 67_108_864) throw new ContextFailure("fix_limit", "Correction inputs and replacements exceed 64 MiB.");
            if (!original.AsSpan().SequenceEqual(replacement))
            {
                changes.Add(new(path, original, replacement));
                files.Add(new(path, Convert.ToHexString(SHA256.HashData(original)), Convert.ToHexString(SHA256.HashData(replacement)), ordered.Length));
            }
        }
        return new(Array.AsReadOnly(suggestions.ToArray()), files.AsReadOnly(), changes.AsReadOnly());
    }
}

// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-baseline)
using Fpas.Compiler;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Lint;
using Fpas.Workspaces.Loading;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record LintBaselineEntry(string Project, string Rule, string Key, string[]? CycleEdges);
internal sealed record LintBaselineSummary(string Path, bool Written, int SuppressedCount, IReadOnlyList<LintBaselineEntry> Resolved);

[JsonUnmappedMemberHandling(JsonUnmappedMemberHandling.Disallow)]
internal sealed record LintBaselineDocument(string Kind, int SchemaVersion, string Context, string Configuration, LintLimits Limits, LintBaselineEntry[] Entries);

internal static class LintBaseline
{
    private const int MaximumBytes = 4_194_304;
    private const string Kind = "fpas-lint-baseline";

    internal static async Task<LintReport> ApplyAsync(string path, bool write, ContextOptions input, LintLimits limits,
        LintReport report, FileWorkspace workspace, CancellationToken token)
    {
        byte[]? original = null;
        LintBaselineDocument? document = null;
        if (!write || File.Exists(path))
        {
            SourcePublication.ValidateRegularPath(path);
            original = await BoundedFile.ReadAsync(path, MaximumBytes, token);
            document = Parse(path, original, input, limits);
        }

        if (write)
        {
            var entries = report.Findings.Where(IsError).Select(finding => Entry(input.Context, finding))
                .DistinctBy(entry => (entry.Project, entry.Rule, entry.Key))
                .OrderBy(entry => entry.Project, StringComparer.Ordinal).ThenBy(entry => entry.Rule, StringComparer.Ordinal)
                .ThenBy(entry => entry.Key, StringComparer.Ordinal).ToArray();
            document = new(Kind, 1, Path.GetFileName(input.Context), input.Configuration, limits, entries);
            var bytes = JsonSerializer.SerializeToUtf8Bytes(document, LintBaselineJson.Default.LintBaselineDocument);
            if (bytes.Length > MaximumBytes) throw new ContextFailure("result_limit", "Lint baseline exceeds 4 MiB.", inputPath: path);
            var result = Filter(path, true, input.Context, report, entries, token);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            await SourcePublication.ApplyAsync([new(path, original, bytes)], workspace.VerifyAsync, token, verifyPublished: workspace.VerifyAsync);
            return result;
        }
        else
        {
            await workspace.VerifyAsync(token);
            await SourcePublication.VerifyAsync([new(path, original, original!)], token);
        }

        return Filter(path, false, input.Context, report, document!.Entries, token);
    }

    private static LintBaselineDocument Parse(string path, byte[] bytes, ContextOptions input, LintLimits limits)
    {
        LintBaselineDocument? document;
        try { document = JsonSerializer.Deserialize(bytes, LintBaselineJson.Default.LintBaselineDocument); }
        catch (JsonException) { throw Invalid("The lint baseline is not a valid version 1 baseline."); }
        if (document is not { Kind: Kind, SchemaVersion: 1, Entries: not null, Limits: not null })
            throw Invalid("The lint baseline is not a valid version 1 baseline.");
        if (document.Context != Path.GetFileName(input.Context) || document.Configuration != input.Configuration || document.Limits != limits)
            throw Invalid("Use the baseline with the same context, configuration and lint thresholds.");
        var keys = new HashSet<(string, string, string)>();
        foreach (var entry in document.Entries)
        {
            if (entry is null || string.IsNullOrWhiteSpace(entry.Project) || Path.IsPathRooted(entry.Project) ||
                string.IsNullOrWhiteSpace(entry.Rule) || string.IsNullOrWhiteSpace(entry.Key) ||
                !keys.Add((entry.Project, entry.Rule, entry.Key)) ||
                (IsCycle(entry.Rule) ? entry.CycleEdges is not { Length: > 0 } || entry.CycleEdges.Any(string.IsNullOrWhiteSpace) : entry.CycleEdges is not null))
                throw Invalid("The lint baseline contains an invalid or duplicate entry.");
        }
        return document;

        ContextFailure Invalid(string message) => new("invalid_baseline", message, inputPath: path);
    }

    private static LintReport Filter(string path, bool written, string context, LintReport report, LintBaselineEntry[] entries, CancellationToken token)
    {
        var stillPresent = new bool[entries.Length];
        var indexed = entries.Select((entry, index) => (Entry: entry, Index: index));
        var exact = indexed.Where(item => item.Entry.CycleEdges is null)
            .ToDictionary(item => (item.Entry.Project, item.Entry.Rule, item.Entry.Key), item => item.Index);
        var cycles = indexed.Where(item => item.Entry.CycleEdges is not null)
            .ToLookup(item => (item.Entry.Project, item.Entry.Rule),
                item => (item.Index, Edges: item.Entry.CycleEdges!.ToHashSet(StringComparer.Ordinal)));
        var findings = new List<LintFinding>();
        var suppressed = 0;
        foreach (var finding in report.Findings)
        {
            token.ThrowIfCancellationRequested();
            if (!IsError(finding)) { findings.Add(finding); continue; }
            var entry = Entry(context, finding);
            var covered = false;
            if (entry.CycleEdges is { } edges)
            {
                foreach (var (index, allowed) in cycles[(entry.Project, entry.Rule)])
                {
                    if (edges.All(allowed.Contains)) stillPresent[index] = covered = true;
                    else if (edges.Any(allowed.Contains)) stillPresent[index] = true;
                }
            }
            else if (exact.TryGetValue((entry.Project, entry.Rule, entry.Key), out var index)) stillPresent[index] = covered = true;

            if (covered) suppressed++;
            else findings.Add(finding);
        }

        var success = !findings.Any(IsError);
        return report with
        {
            Success = success, Status = success ? "ok" : "findings", Findings = findings,
            Baseline = new(path, written, suppressed, entries.Where((_, index) => !stillPresent[index]).ToArray())
        };
    }

    internal static bool IsError(LintFinding finding) => finding.Severity == DiagnosticSeverity.Error;

    private static bool IsCycle(string rule) => rule is "module-cycle" or "namespace-cycle" or "tangle";

    private static LintBaselineEntry Entry(string context, LintFinding finding)
        => new(finding.Rule is "namespace-cycle" or "wide-directory" ? "." :
            Path.GetRelativePath(Path.GetDirectoryName(context)!, finding.Project).Replace(Path.DirectorySeparatorChar, '/'),
            finding.Rule, finding.Key, finding.CycleEdges?.ToArray());
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, WriteIndented = true)]
[JsonSerializable(typeof(LintBaselineDocument))]
internal partial class LintBaselineJson : JsonSerializerContext;

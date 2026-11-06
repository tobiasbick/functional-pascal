using Fpas.Compiler.Analysis;
using System.Text.Json;

namespace Fpas.Workspaces.Analysis;

internal sealed record CachedProject(AnalysisResult Analysis, MetadataImage? Output, long Bytes);

/// <summary>Session-owned bounded reuse of detached compiler facts and private dependency images.</summary>
internal sealed class ProjectAnalysisCache(long limit)
{
    private readonly Dictionary<string, CachedProject> entries = new(StringComparer.Ordinal);
    private readonly LinkedList<string> order = [];
    internal long RetainedBytes { get; private set; }
    internal int Hits { get; private set; }

    internal CachedProject? Get(string key)
    {
        if (!entries.TryGetValue(key, out var value)) return null;
        order.Remove(key); order.AddLast(key); Hits++; return value;
    }

    internal void Add(string key, CachedProject value)
    {
        if (value.Bytes > limit) throw new Loading.ContextFailure("workspace_limit", "Project analysis exceeds the retained workspace budget.");
        if (entries.Remove(key, out var previous)) { RetainedBytes -= previous.Bytes; order.Remove(key); }
        while (order.Count != 0 && (entries.Count >= 64 || RetainedBytes + value.Bytes > limit))
        {
            var oldest = order.First!.Value; order.RemoveFirst(); RetainedBytes -= entries[oldest].Bytes; entries.Remove(oldest);
        }
        entries.Add(key, value); order.AddLast(key); RetainedBytes += value.Bytes;
    }

    internal static long Measure(AnalysisResult result, MetadataImage? output)
        => 4L * JsonSerializer.SerializeToUtf8Bytes(result).Length + (output?.Length ?? 0);

    internal void Trim(long budget)
    {
        while (order.Count != 0 && RetainedBytes > budget)
        { var oldest = order.First!.Value; order.RemoveFirst(); RetainedBytes -= entries[oldest].Bytes; entries.Remove(oldest); }
    }

    internal void Clear() { entries.Clear(); order.Clear(); RetainedBytes = 0; }
}

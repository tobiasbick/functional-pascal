using Microsoft.Diagnostics.Tracing.Etlx;

namespace Fpas.Cli.Profiling;

internal sealed record FrameSamples(string Method, long Samples);
internal sealed record ThreadSamples(int Id, long Samples, int TotalMethods, IReadOnlyList<FrameSamples> Frames);
internal sealed record TraceSummary(long Events, long Samples, long Collections, long AllocationTicks,
    int LostEvents, bool Truncated, IReadOnlyList<ThreadSamples> Threads)
{
    public string Sampling => "thread-wall-time; includes waiting threads, not CPU utilization";

    internal static TraceSummary Read(string file, int maximum, CancellationToken token, string directory)
    {
        file = Path.GetFullPath(file);
        if (!File.Exists(file) || new FileInfo(file).Length is <= 0 or > 536_870_912)
            throw new IOException("Supply a nonempty .nettrace file of at most 512 MiB.");
        var etlx = TraceLog.CreateFromEventPipeDataFile(file, Path.Combine(directory, "trace.etlx"), new TraceLogOptions { MaxEventCount = 5_000_001, LocalSymbolsOnly = true });
        using var trace = new TraceLog(etlx);
        var threads = new Dictionary<int, (long Count, Dictionary<string, long> Frames)>();
        long events = 0, samples = 0, collections = 0, allocations = 0; var truncated = trace.Truncated;
        foreach (var item in trace.Events)
        {
            token.ThrowIfCancellationRequested();
            if (++events > 5_000_000) { truncated = true; break; }
            if (item.ProviderName == "Microsoft-Windows-DotNETRuntime")
            {
                if (item.EventName == "GC/Start") collections++;
                if (item.EventName == "GC/AllocationTick") allocations++;
            }
            if (item.ProviderName != "Microsoft-DotNETCore-SampleProfiler" || item.EventName != "Thread/Sample") continue;
            samples++;
            if (!threads.TryGetValue(item.ThreadID, out var thread))
            {
                if (threads.Count == 256) { truncated = true; continue; }
                thread = (0, new(StringComparer.Ordinal));
            }
            thread.Count++;
            var visited = new HashSet<string>(StringComparer.Ordinal); var depth = 0;
            for (var stack = item.CallStack(); stack is not null && depth++ < 128; stack = stack.Caller)
            {
                var name = stack.CodeAddress.FullMethodName;
                if (string.IsNullOrEmpty(name)) name = "[unresolved]";
                if (!visited.Add(name)) continue;
                if (thread.Frames.Count >= 4096 && !thread.Frames.ContainsKey(name)) { truncated = true; continue; }
                thread.Frames[name] = thread.Frames.GetValueOrDefault(name) + 1;
            }
            if (depth >= 128) truncated = true;
            threads[item.ThreadID] = thread;
        }
        if (events == 0) throw new IOException("The trace contains no decoded events.");
        return new(events, samples, collections, allocations, trace.EventsLost, truncated,
            threads.OrderByDescending(pair => pair.Value.Count).ThenBy(pair => pair.Key).Select(pair => new ThreadSamples(pair.Key, pair.Value.Count, pair.Value.Frames.Count,
                pair.Value.Frames.OrderByDescending(frame => frame.Value).ThenBy(frame => frame.Key, StringComparer.Ordinal).Take(maximum)
                    .Select(frame => new FrameSamples(frame.Key, frame.Value)).ToArray())).ToArray());
    }
}

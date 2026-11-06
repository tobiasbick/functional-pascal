using Fpas.Debugging.Core;

namespace Fpas.Debugging.Dap;

internal sealed class StopHandles<T>
{
    private readonly Dictionary<int, (long Stop, T Value)> entries = [];
    private int sequence;
    internal int Add(long stop, T value)
    {
        var existing = entries.FirstOrDefault(item => item.Value.Stop == stop && EqualityComparer<T>.Default.Equals(item.Value.Value, value));
        if (existing.Key != 0) return existing.Key;
        if (entries.Count >= 4096) throw new DebugException("limit_exceeded", "The DAP stop has more than 4096 live handles.");
        var id = checked(++sequence); entries.Add(id, (stop, value)); return id;
    }
    internal T Get(int id, long stop) => entries.TryGetValue(id, out var entry) && entry.Stop == stop ? entry.Value
        : throw new DebugException("stale_reference", "Use a handle from the current stopped context.");
    internal void Clear() => entries.Clear();
}

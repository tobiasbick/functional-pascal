namespace Fpas.Debugging.Core;

internal sealed partial class OutputCapture
{
    private readonly object gate = new();
    private readonly Queue<(CapturedOutput Item, int Bytes)> items = [];
    private readonly Dictionary<string, TextState> streams = new(StringComparer.Ordinal);
    private int bytes, accepted;
    private long sequence, totalBytes;
    private bool truncated, encodingLoss;
    internal const int MaximumBytes = 1_048_576;
    private readonly int limit;
    private readonly bool prefix;
    private readonly TaskCompletionSource reached = new(TaskCreationOptions.RunContinuationsAsynchronously);
    internal Task LimitReached => reached.Task;

    internal OutputCapture(int limit = MaximumBytes, bool retainPrefix = false)
    {
        if (limit is < 1 or > MaximumBytes) throw new ArgumentOutOfRangeException(nameof(limit));
        this.limit = limit; prefix = retainPrefix;
    }

    internal void AddBytes(string category, ReadOnlySpan<byte> data)
    {
        lock (gate)
        {
            if (!streams.TryGetValue(category, out var state)) streams.Add(category, state = new());
            totalBytes = data.Length > long.MaxValue - totalBytes ? long.MaxValue : totalBytes + data.Length;
            var id = ++sequence;
            var count = prefix ? Math.Min(data.Length, limit - accepted) : data.Length;
            if (prefix) accepted += count;
            if (count != data.Length) truncated = true;
            if (count != 0) Append(category, id, Decode(state, data[..count], flush: false));
            if (prefix && accepted == limit)
            {
                foreach (var stream in streams.Values)
                    if (stream.Pending.Length != 0 && !stream.Boundary) { stream.Boundary = true; stream.PrefixBytes = stream.Pending.Length; }
                if (state.Boundary && count < data.Length) Append(category, id, DecodeBoundary(state, data[count..], eof: false));
            }
            if (totalBytes >= limit) reached.TrySetResult();
        }
    }

    internal void Complete(string category)
    {
        lock (gate)
            if (streams.TryGetValue(category, out var state) && state.Pending.Length != 0)
                Append(category, ++sequence, state.Boundary ? DecodeBoundary(state, [], eof: true) : Decode(state, [], flush: true));
    }

    private void Append(string category, long id, Decoded decoded)
    {
        encodingLoss |= decoded.Loss;
        if (decoded.Text.Length == 0) return;
        items.Enqueue((new(id, category, decoded.Text), decoded.Bytes)); bytes += decoded.Bytes;
        if (!prefix)
            while (bytes > limit) { bytes -= items.Dequeue().Bytes; truncated = true; }
    }

    internal OutputSnapshot Read(long after = 0)
    {
        lock (gate)
        {
            if (after < 0 || after > sequence) throw new DebugException("invalid_argument", "Invalid debugger output cursor.");
            return new(items.Where(item => item.Item.Sequence > after).Select(item => item.Item).ToArray(), sequence,
                prefix ? truncated : truncated && (items.Count == 0 || after < items.Peek().Item.Sequence - 1),
                totalBytes, bytes, reached.Task.IsCompleted, encodingLoss);
        }
    }
}

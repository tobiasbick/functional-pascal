using Fpas.Debugging.Protocol;
using System.Text;

namespace Fpas.Debugging.Sessions;

internal sealed class OutputBuffer
{
    private readonly Queue<(OutputChunk Chunk, int Bytes)> chunks = [];
    private readonly object gate = new();
    private int bytes;
    private long sequence;

    internal void Add(string category, string text)
    {
        var buffer = new StringBuilder(); var size = 0;
        lock (gate)
        {
            foreach (var rune in text.EnumerateRunes())
            {
                if (size + rune.Utf8SequenceLength > 32768) { Store(); }
                buffer.Append(rune); size += rune.Utf8SequenceLength;
            }
            if (buffer.Length != 0) Store();
            void Store()
            {
                chunks.Enqueue((new(++sequence, category, buffer.ToString()), size)); bytes += size;
                while (bytes > 1_048_576) bytes -= chunks.Dequeue().Bytes;
                buffer.Clear(); size = 0;
            }
        }
    }

    internal OutputPage Read(long after, int limit)
    {
        if (limit is < 1 or > 32768) throw new DebugFailure("invalid_argument", "Output limit must be 1..32768 UTF-8 bytes.");
        lock (gate)
        {
            if (after < 0 || after > sequence) throw new DebugFailure("invalid_argument", "Invalid output cursor.");
            var expired = chunks.Count > 0 && after < chunks.Peek().Chunk.Sequence - 1;
            var selected = new List<OutputChunk>(); var used = 0; var next = after;
            foreach (var item in chunks.Where(item => item.Chunk.Sequence > after))
            {
                // A whole chunk is returned so a cursor never skips an unread suffix.
                if (used + item.Bytes > limit)
                {
                    if (selected.Count == 0) throw new DebugFailure("limit_too_small", "Use an output page limit of 32768 bytes.");
                    break;
                }
                selected.Add(item.Chunk); used += item.Bytes; next = item.Chunk.Sequence;
            }
            return new(selected, next, expired, next < sequence);
        }
    }
}

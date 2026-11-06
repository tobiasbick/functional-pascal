using System.Buffers;
using System.Text;

namespace Fpas.Debugging.Core;

internal sealed partial class OutputCapture
{
    private sealed class TextState { internal byte[] Pending = []; internal bool Boundary; internal int PrefixBytes; }
    private sealed record Decoded(string Text, int Bytes, bool Loss);

    private static Decoded DecodeBoundary(TextState state, ReadOnlySpan<byte> data, bool eof)
    {
        var take = Math.Min(data.Length, 4 - state.Pending.Length); var combined = new byte[state.Pending.Length + take];
        state.Pending.CopyTo(combined, 0); data[..take].CopyTo(combined.AsSpan(state.Pending.Length));
        var status = Rune.DecodeFromUtf8(combined, out _, out _);
        if (status == OperationStatus.NeedMoreData && !eof) { state.Pending = combined; return new("", 0, false); }
        var bytes = state.PrefixBytes; state.Pending = []; state.Boundary = false; state.PrefixBytes = 0;
        return status == OperationStatus.Done ? new("", 0, false) : new("\uFFFD", bytes, true);
    }

    private static Decoded Decode(TextState state, ReadOnlySpan<byte> data, bool flush)
    {
        byte[]? joined = null;
        if (state.Pending.Length != 0)
        { joined = new byte[state.Pending.Length + data.Length]; state.Pending.CopyTo(joined, 0); data.CopyTo(joined.AsSpan(state.Pending.Length)); data = joined; }
        state.Pending = [];
        var text = new StringBuilder(); var offset = 0; var loss = false;
        Span<char> characters = stackalloc char[2];
        while (offset < data.Length)
        {
            var status = Rune.DecodeFromUtf8(data[offset..], out var rune, out var consumed);
            if (status == OperationStatus.NeedMoreData && !flush) { state.Pending = data[offset..].ToArray(); break; }
            if (status != OperationStatus.Done)
            { rune = Rune.ReplacementChar; loss = true; consumed = status == OperationStatus.NeedMoreData ? data.Length - offset : Math.Max(1, consumed); }
            text.Append(characters[..rune.EncodeToUtf16(characters)]); offset += consumed;
        }
        return new(text.ToString(), offset, loss);
    }
}

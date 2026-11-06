using System.Text;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Values;

internal static class StoredText
{
    internal static PassiveText Read(ReadOnlySpan<char> buffer, long length)
    {
        var scalars = 0; var end = 0; var invalid = false; var clean = new StringBuilder();
        while (end < buffer.Length && scalars < 4096)
        {
            var current = buffer[end++]; scalars++;
            if (char.IsHighSurrogate(current) && end < buffer.Length && char.IsLowSurrogate(buffer[end])) { clean.Append(current); clean.Append(buffer[end++]); }
            else if (char.IsSurrogate(current)) { invalid = true; clean.Append('\uFFFD'); }
            else clean.Append(current);
        }
        var raw = invalid ? string.Concat(buffer[..end].ToArray().Select(character => ((int)character).ToString("X4", System.Globalization.CultureInfo.InvariantCulture))) : null;
        return new(clean.ToString(), end < length, raw);
    }
}

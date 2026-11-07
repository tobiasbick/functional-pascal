using System.Buffers;
using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Processes.Terminal;

internal sealed class VtInput
{
    private readonly List<byte> bytes = [];
    private bool paste;
    internal bool HasPending => bytes.Count != 0 || paste;

    internal void Append(ReadOnlySpan<byte> input)
    {
        if (bytes.Count + input.Length > 1_048_576) throw new IOException("Terminal input exceeds 1 MiB.");
        foreach (var value in input) bytes.Add(value);
    }

    internal TerminalInput? Take(bool flushEscape)
    {
        if (bytes.Count == 0) return null;
        var input = CollectionsMarshal.AsSpan(bytes);
        if (paste)
        {
            var end = input.IndexOf("\x1b[201~"u8);
            if (end < 0) return null;
            var text = Encoding.UTF8.GetString(input[..end]);
            paste = false;
            bytes.RemoveRange(0, end + 6);
            return new("paste", Text: text);
        }
        if (input[0] == 27)
        {
            if (input.Length == 1 && !flushEscape) return null;
            if (input.Length > 1 && input[1] == (byte)'O')
            {
                if (input.Length < 3 && !flushEscape) return null;
                if (input.Length >= 3)
                {
                    var last = (char)input[2];
                    bytes.RemoveRange(0, 3);
                    return DecodeCsi("", last);
                }
            }
            if (input.Length > 1 && input[1] == (byte)'[')
            {
                for (var index = 2; index < input.Length; index++)
                {
                    if (input[index] is < 64 or > 126) continue;
                    var code = Encoding.ASCII.GetString(input[2..index]);
                    var last = (char)input[index];
                    bytes.RemoveRange(0, index + 1);
                    if (code == "200" && last == '~') { paste = true; return Take(flushEscape); }
                    return DecodeCsi(code, last);
                }
                if (!flushEscape && input.Length < 64) return null;
            }
            if (input.Length > 1 && input[1] is (>= (byte)'a' and <= (byte)'z') or (>= (byte)'A' and <= (byte)'Z'))
            {
                var letter = char.ToUpperInvariant((char)input[1]).ToString();
                bytes.RemoveRange(0, 2);
                return new("key", letter, 2);
            }
            bytes.RemoveAt(0);
            return new("key", "Escape");
        }
        if (input[0] < 32 || input[0] == 127)
        {
            var code = input[0];
            bytes.RemoveAt(0);
            return code switch
            {
                8 or 127 => new("key", "Backspace"), 9 => new("key", "Tab"), 10 or 13 => new("key", "Enter"),
                >= 1 and <= 26 => new("key", ((char)(code + 64)).ToString(), 4), _ => null
            };
        }
        var decoded = Rune.DecodeFromUtf8(input, out var rune, out var consumed);
        if (decoded == OperationStatus.NeedMoreData) return null;
        bytes.RemoveRange(0, consumed);
        return new("text", Text: rune.ToString());
    }

    private static TerminalInput? DecodeCsi(string code, char last)
    {
        if (code.StartsWith('<'))
        {
            var coordinates = code[1..].Split(';');
            if (coordinates.Length == 3 && int.TryParse(coordinates[0], out var button) &&
                int.TryParse(coordinates[1], out var x) && int.TryParse(coordinates[2], out var y) &&
                x is >= 1 and <= 500 && y is >= 1 and <= 300 && last is 'M' or 'm')
            {
                if (button is 64 or 65 && last == 'M') return new("scroll", Column: x - 1, Row: y - 1, Delta: button == 64 ? -3 : 3);
                if ((button & 32) != 0) return new("move", Column: x - 1, Row: y - 1);
                if ((button & 0b0110_0011) == 0) return new(last == 'M' ? "press" : "release", Column: x - 1, Row: y - 1);
            }
            return null;
        }
        var parts = code.Split(';');
        var mods = parts.Length == 2 && int.TryParse(parts[1], out var value) ? Math.Clamp(value - 1, 0, 7) : 0;
        var key = last switch
        {
            'A' => "Up", 'B' => "Down", 'C' => "Right", 'D' => "Left", 'H' => "Home", 'F' => "End", 'Z' => "Tab",
            'P' => "F1", 'Q' => "F2", 'R' => "F3", 'S' => "F4",
            '~' => parts[0] switch
            {
                "1" or "7" => "Home", "2" => "Insert", "3" => "Delete", "4" or "8" => "End",
                "5" => "PageUp", "6" => "PageDown", "15" => "F5", "17" => "F6", "18" => "F7",
                "19" => "F8", "20" => "F9", "21" => "F10", "23" => "F11", "24" => "F12", _ => null
            },
            _ => null
        };
        return key is null ? null : new("key", key, last == 'Z' ? 1 : mods);
    }
}

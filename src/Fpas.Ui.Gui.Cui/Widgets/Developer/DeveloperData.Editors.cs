// [Documentation](../../../../docs/ui/gui/developer-tools/README.md)
using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;
using System.Buffers;
using System.Text;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static unsafe NativeDevField* PackQueryFields(ChartMemory memory, CuiDevField[] fields)
    {
        ArgumentNullException.ThrowIfNull(fields);
        var schema = new NativeDevField[fields.Length];
        for (int i = 0; i < schema.Length; i++)
        {
            CuiDevField field = fields[i];
            ArgumentNullException.ThrowIfNull(field);
            Utf8.Validate(field.Name, nameof(fields));
            if (field.Kind is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(fields));
            string[] choices = field.Kind == 2 ? field.Choices : [];
            ArgumentNullException.ThrowIfNull(choices);
            foreach (string choice in choices)
            {
                Utf8.Validate(choice, nameof(fields));
                if (Utf8.Encoding.GetByteCount(choice) > 63)
                    throw new ArgumentOutOfRangeException(nameof(fields), "A query choice must fit in 63 UTF-8 bytes.");
            }
            schema[i] = new() { Name = memory.Text(field.Name), Kind = field.Kind,
                Choices = memory.Labels(choices), Count = choices.Length };
        }
        return memory.Copy<NativeDevField>(schema);
    }

    private static unsafe void PackQueryRules(CuiDevRule[] rules, NativeDevRule* rows)
    {
        for (int i = 0; i < rules.Length; i++)
        {
            CuiDevRule rule = rules[i];
            ArgumentNullException.ThrowIfNull(rule);
            if (rule.Depth is < 0 or > 64) throw new ArgumentOutOfRangeException(nameof(rules));
            rows[i].Depth = rule.Depth;
            rows[i].Group = rule.Group ? (byte)1 : (byte)0;
            rows[i].Any = rule.Any ? (byte)1 : (byte)0;
            rows[i].Field = rule.Field;
            rows[i].Operator = rule.Operator;
            WriteDevText(rule.Value, new Span<byte>(rows[i].Value, 64));
        }
    }

    private static unsafe NativeDevPair PackPair(CuiDevPair pair)
    {
        NativeDevPair row = default;
        WriteDevText(pair.Key, new Span<byte>(row.Key, 64));
        WriteDevText(pair.Value, new Span<byte>(row.Value, 256));
        row.Enabled = pair.Enabled ? (byte)1 : (byte)0;
        row.Secret = pair.Secret ? (byte)1 : (byte)0;
        return row;
    }

    private static unsafe CuiDevPair[] ReadPairs(NativeDevPair* native, int count)
    {
        var result = new CuiDevPair[count];
        for (int i = 0; i < count; i++)
            result[i] = new(ReadDevText(new ReadOnlySpan<byte>(native[i].Key, 64)),
                ReadDevText(new ReadOnlySpan<byte>(native[i].Value, 256)), native[i].Enabled != 0, native[i].Secret != 0);
        return result;
    }

    private static byte[] DevBuffer(string value, int capacity)
    {
        if (capacity is < 1 or > 1_048_576) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = new byte[capacity + 1];
        WriteDevText(value, buffer);
        return buffer;
    }

    private static void WriteDevText(string text, Span<byte> destination)
    {
        Utf8.Validate(text, nameof(text));
        if (Utf8.Encoding.GetByteCount(text) >= destination.Length)
            throw new ArgumentException("The text exceeds the native UTF-8 buffer.", nameof(text));
        destination[Utf8.Encoding.GetBytes(text, destination)] = 0;
    }

    private static string ReadDevText(ReadOnlySpan<byte> value)
        => Utf8.Encoding.GetString(value[..value.IndexOf((byte)0)]);

    /// <summary>Decodes the query parameters of a URL draft.</summary>
    /// <param name="url">URL text to parse.</param>
    /// <param name="capacity">Maximum number of pairs, from 0 to 4096.</param>
    /// <returns>The decoded pairs.</returns>
    public static unsafe CuiDevPair[] UrlParameters(string url, int capacity)
    {
        if (capacity is < 0 or > 4096) throw new ArgumentOutOfRangeException(nameof(capacity));
        Utf8.Validate(url, nameof(url));
        int query = url.IndexOf('?');
        int fragment = url.IndexOf('#');
        // CUI-137 (docs/future/cui-upstream-bugs.md#cui-137): '?' inside a fragment is not a query delimiter.
        if (capacity == 0 || query < 0 || (fragment >= 0 && fragment < query)) return [];
        var rows = new NativeDevPair[capacity];
        fixed (NativeDevPair* data = rows)
        {
            int count = NativeMethods.UrlParameters(url, data, capacity);
            var result = new CuiDevPair[count];
            for (int i = 0; i < count; ++i)
            {
                result[i] = new(ReadUrlText(new ReadOnlySpan<byte>(data[i].Key, 64)),
                    ReadUrlText(new ReadOnlySpan<byte>(data[i].Value, 256)), data[i].Enabled != 0, data[i].Secret != 0);
            }
            return result;
        }
    }

    private static string ReadUrlText(ReadOnlySpan<byte> data)
    {
        var text = data[..data.IndexOf((byte)0)];
        Span<char> chars = stackalloc char[text.Length];
        int written = 0;
        int bytes = 0;
        while (!text.IsEmpty)
        {
            if (Rune.DecodeFromUtf8(text, out Rune rune, out int consumed) == OperationStatus.NeedMoreData) break;
            bytes += rune.Utf8SequenceLength;
            if (bytes >= data.Length) break;
            written += rune.EncodeToUtf16(chars[written..]);
            text = text[consumed..];
        }
        return new string(chars[..written]);
    }

    /// <summary>Replaces a URL draft's query with enabled pairs that have a key.</summary>
    /// <param name="url">Original URL text.</param>
    /// <param name="pairs">Pairs to encode.</param>
    /// <param name="capacity">Maximum URL byte count excluding the terminator.</param>
    /// <returns>The encoded URL or null if it does not fit.</returns>
    public static unsafe string? SetUrlParameters(string url, CuiDevPair[] pairs, int capacity)
    {
        if (capacity is < 1 or > 1_048_576) throw new ArgumentOutOfRangeException(nameof(capacity));
        ArgumentNullException.ThrowIfNull(pairs);
        Utf8.Validate(url, nameof(url));
        // CUI-137 and CUI-138 (docs/future/cui-upstream-bugs.md): preserve the complete fragment separately.
        int fragment = url.IndexOf('#');
        ReadOnlySpan<char> suffix = fragment < 0 ? [] : url.AsSpan(fragment);
        ReadOnlySpan<char> path = fragment < 0 ? url : url.AsSpan(0, fragment);
        int query = path.IndexOf('?');
        if (query >= 0) path = path[..query];
        int available = capacity - Utf8.Encoding.GetByteCount(suffix);
        if (available < Utf8.Encoding.GetByteCount(path)) return null;

        // CUI-138 (docs/future/cui-upstream-bugs.md#cui-138): CUI reserves three bytes for every encoded byte.
        byte[] output = new byte[available + 3];
        Utf8.Encoding.GetBytes(path, output);
        var rows = new NativeDevPair[pairs.Length];
        for (int i = 0; i < rows.Length; ++i)
        {
            ArgumentNullException.ThrowIfNull(pairs[i], nameof(pairs));
            rows[i] = PackPair(pairs[i]);
        }
        fixed (byte* text = output)
        fixed (NativeDevPair* data = rows)
        {
            if (NativeMethods.SetUrlParameters(text, (nuint)output.Length, data, rows.Length) == 0) return null;
            int length = output.AsSpan().IndexOf((byte)0);
            return length <= available ? string.Concat(Utf8.Encoding.GetString(output, 0, length), suffix) : null;
        }
    }

    /// <summary>Formats the query using the native query builder's vocabulary.</summary>
    /// <param name="fields">Native field schema.</param>
    /// <param name="rules">Current conditions and groups.</param>
    /// <returns>The complete native description, preserving Unicode field names.</returns>
    public static unsafe string DescribeQuery(CuiDevField[] fields, CuiDevRule[] rules)
    {
        ArgumentNullException.ThrowIfNull(rules);
        using var memory = new ChartMemory();
        var schema = PackQueryFields(memory, fields);
        var rows = new NativeDevRule[rules.Length];
        fixed (NativeDevRule* data = rows)
        {
            PackQueryRules(rules, data);
            nuint length = NativeMethods.DescribeQuery(schema, fields.Length, data, rules.Length, null, 0);
            byte[] output = new byte[checked((int)length + 1)];
            fixed (byte* text = output)
                NativeMethods.DescribeQuery(schema, fields.Length, data, rules.Length, text, (nuint)output.Length);
            return ReadDevText(output);
        }
    }
}

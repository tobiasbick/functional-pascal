using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Editable key and value pair.</summary>
/// <param name="Key">Key supplied to the widget.</param>
/// <param name="Value">Value supplied to the widget.</param>
/// <param name="Enabled">Enabled supplied to the widget.</param>
/// <param name="Secret">Secret supplied to the widget.</param>
public sealed record CuiDevPair(string Key, string Value, bool Enabled, bool Secret);

/// <summary>Allowed field in a native query.</summary>
/// <param name="Name">Name supplied to the widget.</param>
/// <param name="Kind">Kind supplied to the widget.</param>
/// <param name="Choices">Choices supplied to the widget.</param>
public sealed record CuiDevField(string Name, int Kind, string[] Choices);

/// <summary>One group or condition in a query.</summary>
/// <param name="Depth">Depth supplied to the widget.</param>
/// <param name="Group">Group supplied to the widget.</param>
/// <param name="Any">Any supplied to the widget.</param>
/// <param name="Field">Field supplied to the widget.</param>
/// <param name="Operator">Operator supplied to the widget.</param>
/// <param name="Value">Value supplied to the widget.</param>
public sealed record CuiDevRule(int Depth, bool Group, bool Any, int Field, int Operator, string Value);

/// <summary>Feature flag state across environments.</summary>
/// <param name="Key">Key supplied to the widget.</param>
/// <param name="Description">Description supplied to the widget.</param>
/// <param name="On">On supplied to the widget.</param>
/// <param name="Rollout">Rollout supplied to the widget.</param>
public sealed record CuiDevFlag(string Key, string? Description, bool[] On, int[] Rollout);

/// <summary>Schedule expression and native validation.</summary>
/// <param name="Expression">Expression supplied to the widget.</param>
/// <param name="Valid">Valid supplied to the widget.</param>
/// <param name="Next">Next supplied to the widget.</param>
public sealed record CuiScheduleProposal(string Expression, bool Valid, long Next);

/// <summary>Current regex text and match summary.</summary>
/// <param name="Pattern">Pattern supplied to the widget.</param>
/// <param name="Sample">Sample supplied to the widget.</param>
/// <param name="Modifiers">Modifiers supplied to the widget.</param>
/// <param name="Valid">Valid supplied to the widget.</param>
/// <param name="Matches">Matches supplied to the widget.</param>
public sealed record CuiRegexProposal(string Pattern, string Sample, int Modifiers, bool Valid, int Matches);

/// <summary>Current HTTP form and dispatch intents.</summary>
/// <param name="Method">Method supplied to the widget.</param>
/// <param name="Url">Url supplied to the widget.</param>
/// <param name="Send">Send supplied to the widget.</param>
/// <param name="Cancel">Cancel supplied to the widget.</param>
public sealed record CuiRequestProposal(int Method, string Url, bool Send, bool Cancel);

/// <summary>Changed feature flag cell.</summary>
/// <param name="Index">Index supplied to the widget.</param>
/// <param name="Environment">Environment supplied to the widget.</param>
/// <param name="On">On supplied to the widget.</param>
/// <param name="Rollout">Rollout supplied to the widget.</param>
public sealed record CuiFlagProposal(int Index, int Environment, bool On, int Rollout);

public sealed partial class CuiWindow
{
    private static unsafe NativeDevField* PackQueryFields(ChartMemory memory, CuiDevField[] fields)
    {
        var schema = new NativeDevField[fields.Length];
        for (int i = 0; i < schema.Length; i++)
            schema[i] = new() { Name = memory.Text(fields[i].Name), Kind = fields[i].Kind,
                Choices = memory.Labels(fields[i].Choices), Count = fields[i].Choices.Length };
        return memory.Copy<NativeDevField>(schema);
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

    /// <summary>Decodes the query parameters of an example URL.</summary>
    /// <param name="url">URL to parse.</param>
    /// <param name="capacity">Maximum number of pairs.</param>
    /// <returns>The decoded pairs.</returns>
    public static unsafe CuiDevPair[] UrlParameters(string url, int capacity)
    {
        if (capacity is < 1 or > 4096) throw new ArgumentOutOfRangeException(nameof(capacity));
        Utf8.Validate(url, nameof(url));
        var rows = new NativeDevPair[capacity];
        fixed (NativeDevPair* data = rows)
            return ReadPairs(data, NativeMethods.UrlParameters(url, data, capacity));
    }

    /// <summary>Replaces an example URL's query string with the enabled pairs.</summary>
    /// <param name="url">Base URL.</param>
    /// <param name="pairs">Pairs to encode.</param>
    /// <param name="capacity">Maximum URL byte count excluding the terminator.</param>
    /// <returns>The encoded URL or null if it does not fit.</returns>
    public static unsafe string? SetUrlParameters(string url, CuiDevPair[] pairs, int capacity)
    {
        byte[] output = DevBuffer(url, capacity);
        var rows = new NativeDevPair[pairs.Length];
        for (int i = 0; i < rows.Length; ++i) rows[i] = PackPair(pairs[i]);
        fixed (byte* text = output)
        fixed (NativeDevPair* data = rows)
            return NativeMethods.SetUrlParameters(text, (nuint)output.Length, data, rows.Length) != 0 ? ReadDevText(output) : null;
    }

    /// <summary>Formats the query using the native query builder's vocabulary.</summary>
    /// <param name="fields">Native field schema.</param>
    /// <param name="rules">Current conditions and groups.</param>
    /// <returns>The native description.</returns>
    public static unsafe string DescribeQuery(CuiDevField[] fields, CuiDevRule[] rules)
    {
        using var memory = new ChartMemory();
        var schema = PackQueryFields(memory, fields);
        var rows = new NativeDevRule[rules.Length];
        byte[] output = new byte[checked(1 + 256 * rules.Length)];
        fixed (NativeDevRule* data = rows)
        fixed (byte* text = output)
        {
            for (int i = 0; i < rows.Length; i++)
            {
                var rule = rules[i];
                data[i].Depth = rule.Depth;
                data[i].Group = rule.Group ? (byte)1 : (byte)0;
                data[i].Any = rule.Any ? (byte)1 : (byte)0;
                data[i].Field = rule.Field;
                data[i].Operator = rule.Operator;
                WriteDevText(rule.Value, new Span<byte>(data[i].Value, 64));
            }
            NativeMethods.DescribeQuery(schema, fields.Length, data, rules.Length, text, (nuint)output.Length);
            return ReadDevText(output);
        }
    }
}

using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One native inspector row.</summary>
/// <param name="Label">Row label.</param>
/// <param name="Kind">Native value kind.</param>
/// <param name="Integer">Integer, boolean or selection.</param>
/// <param name="Values">Number or vector data.</param>
/// <param name="Text">Text value.</param>
/// <param name="Color">Packed color.</param>
/// <param name="Choices">Enum choices.</param>
/// <param name="Minimum">Numeric minimum.</param>
/// <param name="Maximum">Numeric maximum.</param>
/// <param name="Step">Scrub step.</param>
/// <param name="Digits">Decimal precision.</param>
/// <param name="Suffix">Optional unit.</param>
/// <param name="Placeholder">Empty text hint.</param>
/// <param name="Tooltip">Row tooltip.</param>
/// <param name="Capacity">Text capacity.</param>
/// <param name="Flags">Input and checkbox flags.</param>
public sealed record CuiProperty(string Label, int Kind, int Integer, float[] Values, string? Text, uint Color, string[] Choices, float Minimum, float Maximum, float Step, int Digits, string Suffix, string Placeholder, string Tooltip, int Capacity, int Flags);

/// <summary>One labeled group in the native inspector.</summary>
/// <param name="Title">Group title.</param>
/// <param name="Properties">Property rows.</param>
/// <param name="Open">Expansion state.</param>
/// <param name="Collapsible">Allows collapsing.</param>
public sealed record CuiPropertyGroup(string Title, CuiProperty[] Properties, bool Open, bool Collapsible);

/// <summary>A native inspector proposal.</summary>
/// <param name="Group">Group index.</param>
/// <param name="Row">Property index, or minus one for expansion.</param>
/// <param name="Open">Proposed expansion.</param>
/// <param name="Integer">Proposed scalar integer.</param>
/// <param name="Values">Proposed numeric values.</param>
/// <param name="Text">Proposed text.</param>
/// <param name="Color">Proposed color.</param>
public sealed record CuiPropertyEdit(int Group, int Row, bool Open, int Integer, float[] Values, string? Text, uint Color);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeProperty
{
    internal byte* Label, Suffix, Placeholder, Tooltip, Text;
    internal byte** Choices;
    internal int Kind, Integer, Count, Capacity, Digits, Flags, Changed;
    internal float Minimum, Maximum, Step;
    internal fixed float Values[4];
    internal uint Rgba;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePropertyGroup
{
    internal byte* Title;
    internal NativeProperty* Properties;
    internal int Count, Open, Collapsible, Changed;
}

public sealed partial class CuiWindow
{
    /// <summary>Draws a native PropertyInspector.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="groups">Ordered groups and typed property rows.</param>
    /// <param name="labelWidth">Initial label column width.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="enabled">Accept pointer and key input.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiPropertyEdit[] PropertyInspector(string id, CuiPropertyGroup[] groups, float labelWidth, int widthKind, float width, bool enabled)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        using var memory = new ChartMemory();
        var native = new NativePropertyGroup[groups.Length];
        for (int index = 0; index < native.Length; index++)
        {
            var item = groups[index];
            var rows = new NativeProperty[item.Properties.Length];
            for (int row = 0; row < rows.Length; row++)
                rows[row] = PackProperty(item.Properties[row], memory);
            native[index] = new() { Title = memory.Text(item.Title), Properties = memory.Copy<NativeProperty>(rows),
                Count = rows.Length, Open = item.Open ? 1 : 0, Collapsible = item.Collapsible ? 1 : 0 };
        }

        fixed (NativePropertyGroup* data = native)
            NativeMethods.PropertyInspector(context, Stable(id), data, native.Length, labelWidth, widthKind, width, enabled ? 1 : 0);
        if (!enabled) return [];

        var events = new List<CuiPropertyEdit>();
        for (int index = 0; index < native.Length; index++)
        {
            var group = native[index];
            if (group.Changed != 0) events.Add(new(index, -1, group.Open != 0, 0, [], null, 0));
            for (int row = 0; row < group.Count; row++)
            {
                var value = group.Properties[row];
                if (value.Changed == 0) continue;
                int components = value.Kind == 6 ? value.Count : value.Kind == 0 ? 1 : 0;
                events.Add(new(index, row, group.Open != 0, value.Integer,
                    new ReadOnlySpan<float>(value.Values, components).ToArray(),
                    Marshal.PtrToStringUTF8((nint)value.Text), value.Rgba));
            }
        }
        return events.ToArray();
    }

    private static unsafe NativeProperty PackProperty(CuiProperty value, ChartMemory memory)
    {
        var native = new NativeProperty
        {
            Label = memory.Text(value.Label), Suffix = memory.Text(OptionalAi(value.Suffix)),
            Placeholder = memory.Text(OptionalAi(value.Placeholder)), Tooltip = memory.Text(OptionalAi(value.Tooltip)),
            Choices = memory.Labels(value.Choices), Kind = value.Kind, Integer = value.Integer,
            Count = value.Kind == 4 ? value.Choices.Length : value.Values.Length, Capacity = value.Capacity,
            Digits = value.Digits, Flags = value.Flags, Minimum = value.Minimum, Maximum = value.Maximum,
            Step = value.Step, Rgba = value.Color
        };
        for (int index = 0; index < value.Values.Length; index++)
            native.Values[index] = value.Values[index];
        if (value.Kind == 2)
        {
            var buffer = new byte[value.Capacity];
            Utf8.Encoding.GetBytes(value.Text ?? "", buffer.AsSpan(0, buffer.Length - 1));
            native.Text = memory.Copy<byte>(buffer);
        }
        return native;
    }
}

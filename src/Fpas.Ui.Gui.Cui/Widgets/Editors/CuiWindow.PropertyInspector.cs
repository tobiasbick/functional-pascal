using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

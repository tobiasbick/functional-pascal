// [Documentation](../../../../docs/ui/gui/editing-tools/property-inspector.md)
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
        if (!float.IsFinite(labelWidth) || labelWidth < 0)
            throw new ArgumentOutOfRangeException(nameof(labelWidth));
        ArgumentNullException.ThrowIfNull(groups);
        using var memory = new ChartMemory();
        var native = new NativePropertyGroup[groups.Length];
        for (int index = 0; index < native.Length; index++)
        {
            var item = groups[index];
            ArgumentNullException.ThrowIfNull(item, nameof(groups));
            Utf8.Validate(item.Title, nameof(groups));
            ArgumentNullException.ThrowIfNull(item.Properties, nameof(groups));
            var rows = new NativeProperty[item.Properties.Length];
            for (int row = 0; row < rows.Length; row++)
            {
                var property = item.Properties[row];
                CheckProperty(property, nameof(groups));
                rows[row] = PackProperty(property, memory);
            }
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

    private static void CheckProperty(CuiProperty property, string parameter)
    {
        ArgumentNullException.ThrowIfNull(property, parameter);
        Utf8.Validate(property.Label, parameter);
        Utf8.Validate(property.Suffix, parameter);
        Utf8.Validate(property.Placeholder, parameter);
        Utf8.Validate(property.Tooltip, parameter);
        ArgumentNullException.ThrowIfNull(property.Values, parameter);
        ArgumentNullException.ThrowIfNull(property.Choices, parameter);
        if (property.Kind is < 0 or > 6 || property.Values.Length > NativeProperty.MaxComponents ||
            property.Kind == 0 && property.Values.Length != 1 ||
            property.Kind == 6 && property.Values.Length == 0)
            throw new ArgumentException("Invalid property kind or numeric component count.", parameter);
        if (property.Digits is < -1 or > 9 || !float.IsFinite(property.Minimum) ||
            !float.IsFinite(property.Maximum) || !float.IsFinite(property.Step) || property.Step < 0 ||
            property.Values.Any(value => !float.IsFinite(value)))
            throw new ArgumentException("Property bounds, values and precision must be valid.", parameter);
        if (property.Kind == 4 && (property.Choices.Length == 0 ||
            property.Integer < -1 || property.Integer >= property.Choices.Length))
            throw new ArgumentException("Expected a nonempty choice list and a valid selection or minus one.", parameter);
        foreach (string choice in property.Choices) Utf8.Validate(choice, parameter);
        if (property.Kind == 2)
        {
            string content = property.Text ?? "";
            Utf8.Validate(content, parameter);
            if (property.Capacity is < 1 or > 1_048_576 || Utf8.Encoding.GetByteCount(content) >= property.Capacity)
                throw new ArgumentException("Property text must fit its native UTF-8 buffer.", parameter);
        }
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
        value.Values.AsSpan().CopyTo(new Span<float>(native.Values, NativeProperty.MaxComponents));
        if (value.Kind == 2)
        {
            var buffer = new byte[value.Capacity];
            Utf8.Encoding.GetBytes(value.Text ?? "", buffer.AsSpan(0, buffer.Length - 1));
            native.Text = memory.Copy<byte>(buffer);
        }
        return native;
    }
}

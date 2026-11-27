// [Documentation](../../../../docs/ui/gui/inputs/select.md)
using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native select with model-owned selection.</summary>
    /// <param name="id">Stable select identity.</param>
    /// <param name="items">Nonempty choice captions.</param>
    /// <param name="icons">One glyph or empty string per choice.</param>
    /// <param name="selected">Current index, or minus one.</param>
    /// <param name="label">Optional caption above the trigger.</param>
    /// <param name="placeholder">Unselected caption. Null uses CUI's Select placeholder.</param>
    /// <param name="searchable">Whether the popup includes a filter field.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackOption">Maps original indices to tracking IDs, or null to skip tracking.</param>
    /// <param name="widthKind">Native width kind. Zero uses the default for the label.</param>
    /// <param name="width">Fixed width in logical units or percentage of available width.</param>
    /// <returns>Current native selection and popup state.</returns>
    public unsafe CuiSelectResult Select(string id, string[] items, string[] icons, int selected,
        string label, string? placeholder, bool searchable, bool enabled, bool focus, Func<int, string>? trackOption,
        int widthKind = 0, float width = 0)
    {
        CheckFrame();
        string stable = Stable(id);
        ArgumentNullException.ThrowIfNull(items);
        ArgumentNullException.ThrowIfNull(icons);
        Utf8.Validate(label, nameof(label));
        if (placeholder is not null) Utf8.Validate(placeholder, nameof(placeholder));
        CheckExtent(widthKind, width, nameof(width));
        if (items.Length == 0 || icons.Length != items.Length)
            throw new ArgumentException("Supply a nonempty list of choices with one glyph per choice.");
        if (selected < -1 || selected >= items.Length) throw new ArgumentOutOfRangeException(nameof(selected));
        for (int index = 0; index < items.Length; ++index)
        {
            Utf8.Validate(items[index], nameof(items));
            Utf8.Validate(icons[index], nameof(icons));
            if (string.IsNullOrWhiteSpace(items[index]))
                throw new ArgumentException("Each select option needs a visible label.", nameof(items));
        }

        Span<int> nodes = items.Length < 128 ? stackalloc int[items.Length + 1] : new int[items.Length + 1];
        bool open;
        fixed (int* parts = nodes)
            open = NativeMethods.Select(context, stable, items, icons, items.Length, ref selected,
                Optional(label), placeholder, searchable ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0,
                widthKind, width, parts) != 0;
        if (trackOption is not null) TrackControl(id, nodes[0]);
        if (!open) return new(selected, false, []);

        List<int> visible = [];
        for (int index = 0; index < items.Length; ++index)
        {
            if (nodes[index + 1] < 0) continue;
            if (trackOption is not null) TrackControl(trackOption(index), nodes[index + 1]);
            visible.Add(index);
        }
        return new(selected, true, visible.ToArray());
    }
}

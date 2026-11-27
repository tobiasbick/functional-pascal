// [Documentation](../../../../docs/ui/gui/pickers/multi-select.md)
using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's multiple-selection picker with model-owned flags.</summary>
    /// <param name="id">Stable field identity.</param>
    /// <param name="labels">Nonempty list of choice captions.</param>
    /// <param name="icons">One glyph or empty string per choice.</param>
    /// <param name="selected">One flag per choice.</param>
    /// <param name="label">Caption above the trigger, or null to omit the label wrapper.</param>
    /// <param name="placeholder">Empty-selection caption, or null for CUI's Select caption.</param>
    /// <param name="maxChips">Visible chip limit, or zero for CUI's default.</param>
    /// <param name="searchable">Whether the list includes a search field.</param>
    /// <param name="enabled">Whether the picker accepts input.</param>
    /// <param name="focus">Whether to focus the trigger.</param>
    /// <param name="trackOption">Maps option indices to tracking identities, or null.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <returns>The selection proposal and native popup state.</returns>
    public unsafe CuiMultiSelectResult MultiSelect(string id, string[] labels, string[] icons, bool[] selected,
        string? label, string? placeholder, int maxChips, bool searchable, bool enabled, bool focus,
        Func<int, string>? trackOption, int widthKind = 0, float width = 0)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        ArgumentNullException.ThrowIfNull(selected);
        if (labels.Length == 0 || labels.Length != icons.Length || labels.Length != selected.Length)
            throw new ArgumentException("Supply one selection flag and glyph per choice.");
        ArgumentOutOfRangeException.ThrowIfNegative(maxChips);
        CheckExtent(widthKind, width, nameof(width));
        if (label is not null) Utf8.Validate(label, nameof(label));
        if (placeholder is not null) Utf8.Validate(placeholder, nameof(placeholder));
        int[] flags = new int[labels.Length], nodes = new int[labels.Length + 1];
        for (int i = 0; i < labels.Length; i++)
        {
            Utf8.Validate(labels[i], nameof(labels));
            Utf8.Validate(icons[i], nameof(icons));
            flags[i] = selected[i] ? 1 : 0;
        }
        bool changed;
        int open;
        fixed (int* values = flags, parts = nodes)
            changed = NativeMethods.MultiSelect(context, Stable(id), labels, icons, labels.Length, values,
                label, placeholder, maxChips, searchable ? 1 : 0, widthKind, width, enabled ? 1 : 0,
                focus ? 1 : 0, parts, out open) != 0;
        bool[]? proposal = changed ? Array.ConvertAll(flags, value => value != 0) : null;
        if (trackOption is null) return new(proposal, open != 0, []);
        TrackControl(id, nodes[0]);
        List<int> visible = [];
        for (int i = 0; i < labels.Length; i++)
        {
            if (nodes[i + 1] < 0) continue;
            TrackControl(trackOption(i), nodes[i + 1]);
            visible.Add(i);
        }
        return new(proposal, open != 0, visible.ToArray());
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>The native picker proposal and visible options.</summary>
/// <param name="Selection">Fresh selection flags on a change, or null.</param>
/// <param name="Open">Whether the popup accepts input.</param>
/// <param name="VisibleOptions">Original indices of tracked option rows.</param>
public readonly record struct CuiMultiSelectResult(bool[]? Selection, bool Open, int[] VisibleOptions);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's multiple-selection picker with model-owned flags.</summary>
    /// <param name="id">Stable field identity.</param>
    /// <param name="labels">Nonempty list of choice captions.</param>
    /// <param name="icons">One glyph or empty string per choice.</param>
    /// <param name="selected">One flag per choice.</param>
    /// <param name="label">Optional caption above the trigger.</param>
    /// <param name="placeholder">Optional empty-selection caption.</param>
    /// <param name="maxChips">Visible chip limit, or zero for CUI's default.</param>
    /// <param name="searchable">Whether the list includes a search field.</param>
    /// <param name="enabled">Whether the picker accepts input.</param>
    /// <param name="focus">Whether to focus the trigger.</param>
    /// <param name="trackOption">Maps option indices to tracking identities, or null.</param>
    /// <returns>The selection proposal and native popup state.</returns>
    public unsafe CuiMultiSelectResult MultiSelect(string id, string[] labels, string[] icons, bool[] selected,
        string label, string placeholder, int maxChips, bool searchable, bool enabled, bool focus,
        Func<int, string>? trackOption)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        ArgumentNullException.ThrowIfNull(selected);
        if (labels.Length == 0 || labels.Length != icons.Length || labels.Length != selected.Length)
            throw new ArgumentException("Supply one selection flag and glyph per choice.");
        ArgumentOutOfRangeException.ThrowIfNegative(maxChips);
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(placeholder, nameof(placeholder));
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
                Optional(label), Optional(placeholder), maxChips, searchable ? 1 : 0, enabled ? 1 : 0,
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

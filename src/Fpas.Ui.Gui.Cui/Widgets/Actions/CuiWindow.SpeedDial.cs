// [Documentation](../../../../docs/ui/gui/actions/speed-dial.md)
using Fpas.Ui.Gui.Cui.Widgets.Actions;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's floating action trigger and native action rows.</summary>
    /// <param name="id">Stable trigger identity.</param>
    /// <param name="labels">Ordered action labels. Null entries omit the label bubble.</param>
    /// <param name="icons">Glyphs in action order. Null entries omit the glyph.</param>
    /// <param name="icon">Trigger glyph. Null uses the native plus icon.</param>
    /// <param name="tooltip">Optional trigger hint.</param>
    /// <param name="placement">CUI placement ordinal.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Zero or one plus the requested node slot.</param>
    /// <param name="trackAction">Maps action indices to tracking identities, or null.</param>
    /// <returns>The native choice, open state and rendered actions.</returns>
    public unsafe CuiSpeedDialResult SpeedDial(string id, string?[] labels, string?[] icons, string? icon,
        string? tooltip, int placement, bool enabled, int focus, Func<int, string>? trackAction)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        if (labels.Length is < 1 or > 16 || icons.Length != labels.Length)
            throw new ArgumentException("Supply one through sixteen action slots.");
        ArgumentOutOfRangeException.ThrowIfNegative(focus);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(focus, labels.Length + 1);
        ArgumentOutOfRangeException.ThrowIfNegative(placement);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(placement, 9);
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        if (tooltip is not null) Utf8.Validate(tooltip, nameof(tooltip));
        for (int i = 0; i < labels.Length; i++)
        {
            if (labels[i] is { } label) Utf8.Validate(label, nameof(labels));
            if (icons[i] is { } glyph) Utf8.Validate(glyph, nameof(icons));
        }
        Span<int> nodes = stackalloc int[labels.Length + 1];
        int choice, open;
        fixed (int* parts = nodes)
            choice = NativeMethods.SpeedDial(context, Stable(id), labels, icons, labels.Length,
                icon, tooltip, placement, enabled ? 1 : 0, focus, parts, out open);
        if (choice == -2) throw new InvalidOperationException("CUI's disabled-scope stack is full.");
        if (trackAction is null) return new(choice, open != 0, []);
        TrackControl(id, nodes[0]);
        List<int> visible = [];
        for (int i = 0; i < labels.Length; i++)
        {
            if (nodes[i + 1] < 0) continue;
            TrackControl(trackAction(i), nodes[i + 1]);
            visible.Add(i);
        }
        return new(choice, open != 0, visible.ToArray());
    }
}

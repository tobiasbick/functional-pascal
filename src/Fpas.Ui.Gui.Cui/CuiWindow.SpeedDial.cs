using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native speed dial's choice and visible action rows.</summary>
/// <param name="Choice">Chosen action index, or minus one.</param>
/// <param name="Open">Whether the dial accepts action input.</param>
/// <param name="VisibleActions">Original indices of tracked action rows.</param>
public readonly record struct CuiSpeedDialResult(int Choice, bool Open, int[] VisibleActions);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's floating action trigger and native action rows.</summary>
    /// <param name="id">Stable trigger identity.</param>
    /// <param name="labels">Ordered action labels.</param>
    /// <param name="icons">One glyph per action.</param>
    /// <param name="icon">Optional trigger glyph.</param>
    /// <param name="tooltip">Optional trigger hint.</param>
    /// <param name="placement">CUI placement ordinal.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Zero or one plus the requested node slot.</param>
    /// <param name="trackAction">Maps action indices to tracking identities, or null.</param>
    /// <returns>The native choice, open state and rendered actions.</returns>
    public unsafe CuiSpeedDialResult SpeedDial(string id, string[] labels, string[] icons, string icon,
        string tooltip, int placement, bool enabled, int focus, Func<int, string>? trackAction)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        if (labels.Length is < 1 or > 16 || icons.Length != labels.Length)
            throw new ArgumentException("Supply one through sixteen labels and glyphs.");
        ArgumentOutOfRangeException.ThrowIfNegative(focus);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(focus, labels.Length + 1);
        ArgumentOutOfRangeException.ThrowIfNegative(placement);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(placement, 9);
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(tooltip, nameof(tooltip));
        for (int i = 0; i < labels.Length; i++)
        {
            Utf8.Validate(labels[i], nameof(labels));
            Utf8.Validate(icons[i], nameof(icons));
        }
        Span<int> nodes = stackalloc int[labels.Length + 1];
        int choice, open;
        fixed (int* parts = nodes)
            choice = NativeMethods.SpeedDial(context, Stable(id), labels, icons, labels.Length,
                Optional(icon), Optional(tooltip), placement, enabled ? 1 : 0, focus, parts, out open);
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

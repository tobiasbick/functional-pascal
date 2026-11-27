// [Documentation](../../../../docs/ui/gui/pickers/wheel-picker.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native wheel of choices.</summary>
    /// <param name="id">Stable wheel identity.</param>
    /// <param name="items">Nonempty array of captions in selection order.</param>
    /// <param name="selected">Model-owned zero-based selection.</param>
    /// <param name="widthKind">Native width extent kind.</param>
    /// <param name="width">Logical width or percentage for the selected extent.</param>
    /// <param name="rows">Visible rows. Nonpositive uses five. CUI rounds even counts up.</param>
    /// <param name="loop">Whether to wrap around at the ends.</param>
    /// <param name="joinLeft">Whether the selection band joins the previous wheel.</param>
    /// <param name="joinRight">Whether the selection band joins the next wheel.</param>
    /// <param name="enabled">Whether the wheel accepts input.</param>
    /// <param name="focus">Whether to request wheel focus.</param>
    /// <param name="track">Whether to publish the wheel's native bounds and focus.</param>
    /// <returns>The proposed zero-based selection.</returns>
    public int WheelPicker(string id, string[] items, int selected, int widthKind, float width, int rows,
        bool loop, bool joinLeft, bool joinRight, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(items);
        if (selected < 0 || selected >= items.Length) throw new ArgumentOutOfRangeException(nameof(selected));
        CheckExtent(widthKind, width, nameof(width));
        foreach (string item in items) Utf8.Validate(item, nameof(items));

        int proposed = NativeMethods.WheelPicker(context, Stable(id), items, items.Length, selected, widthKind, width, rows,
            loop ? 1 : 0, joinLeft ? 1 : 0, joinRight ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return proposed;
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A color proposal and the native popup's visibility.</summary>
/// <param name="Value">Straight-alpha RGBA with red in the low byte.</param>
/// <param name="Open">Whether the native palette accepts input.</param>
public readonly record struct CuiColorPickerResult(uint Value, bool Open);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native color field and its palette.</summary>
    /// <param name="id">Stable color field identity.</param>
    /// <param name="value">Current straight-alpha color with red in the low byte.</param>
    /// <param name="enabled">Whether the field accepts input.</param>
    /// <param name="focus">Whether to request focus on the field.</param>
    /// <param name="trackPart">Maps palette part indices to tracking IDs, or null.</param>
    /// <returns>The proposed RGB channels with the model's alpha and the popup state.</returns>
    public unsafe CuiColorPickerResult ColorPicker(string id, uint value, bool enabled, bool focus,
        Func<int, string>? trackPart)
    {
        CheckFrame();
        int* nodes = stackalloc int[11];
        uint proposed = NativeMethods.ColorPicker(context, Stable(id), value, enabled ? 1 : 0, focus ? 1 : 0,
            trackPart is null ? null : nodes, out int open);
        if (trackPart is not null)
        {
            TrackControl(id, nodes[0]);
            if (open != 0)
                for (int i = 0; i < 10; ++i) TrackControl(trackPart(i), nodes[i + 1]);
        }
        return new(proposed, open != 0);
    }
}

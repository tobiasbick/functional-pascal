using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's legacy voice orb and reports activation.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="activity">Native voice activity.</param>
    /// <param name="level">Current loudness.</param>
    /// <param name="size">Orb diameter or zero.</param>
    /// <param name="color">Optional ink override.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    public unsafe bool VoiceOrb(string id, int activity, float level, float size, CuiChartColor color, int flags)
    {
        CheckFrame();
        return NativeMethods.VoiceOrb(context, Stable(id), activity, level, size, color.Palette, color.Rgba, flags) != 0;
    }
}

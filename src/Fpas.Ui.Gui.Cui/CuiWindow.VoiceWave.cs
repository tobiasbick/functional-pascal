using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a voice timeline and returns the proposed progress, or zero for a passive strip.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="levels">Measured levels, newest last.</param>
    /// <param name="owners">Optional speaker tags.</param>
    /// <param name="progress">Played fraction, or null.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="pitch">Dot spacing or zero.</param>
    /// <param name="step">Seek fraction or zero.</param>
    /// <param name="owner">Default native speaker.</param>
    /// <param name="color">Optional ink override.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    public unsafe float VoiceWave(string id, float[] levels, byte[]? owners, float? progress, int widthKind, float width,
        float pitch, float step, int owner, CuiChartColor color, int flags)
    {
        CheckFrame();
        CheckVoiceSignal(levels, owners);
        CheckExtent(widthKind, width, nameof(width));
        float next = progress.GetValueOrDefault();
        fixed (float* signal = levels)
        fixed (byte* tags = owners)
            NativeMethods.VoiceWave(context, Stable(id), signal, tags, levels.Length, progress.HasValue ? &next : null,
                widthKind, width, pitch, step, owner, color.Palette, color.Rgba, flags);
        return next;
    }
}

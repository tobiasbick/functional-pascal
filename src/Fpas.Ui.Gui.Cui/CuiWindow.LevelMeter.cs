using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native audio level meter.</summary>
    /// <param name="id">Stable meter identity.</param>
    /// <param name="levels">One to eight nonnegative linear peaks.</param>
    /// <param name="minimumDb">Negative lower bound in dBFS.</param>
    /// <param name="maximumDb">Upper bound in dBFS, or zero for the native default.</param>
    /// <param name="length">Logical bar length, or zero for the native default.</param>
    /// <param name="thickness">Logical channel thickness, or zero for the native default.</param>
    /// <param name="horizontal">Whether bars run from left to right.</param>
    /// <param name="scale">Whether to label the dB scale.</param>
    /// <param name="segmented">Whether to draw LED segments.</param>
    /// <param name="hold">Peak hold in seconds. Negative disables hold, zero uses the native default.</param>
    /// <param name="decay">Fall rate in dB per second, or zero for the native default.</param>
    /// <param name="enabled">Whether the clip indicator accepts a reset.</param>
    /// <param name="track">Whether to publish the meter bounds.</param>
    /// <returns>Whether the clip indicator was clicked.</returns>
    public unsafe bool LevelMeter(string id, float[] levels, float minimumDb, float maximumDb, float length,
        float thickness, bool horizontal, bool scale, bool segmented, float hold, float decay,
        bool enabled, bool track)
    {
        CheckFrame();
        ValidateAudioLevels(levels, minimumChannels: 1);
        if (!float.IsFinite(minimumDb) || minimumDb >= 0)
            throw new ArgumentOutOfRangeException(nameof(minimumDb));
        if (!float.IsFinite(maximumDb - minimumDb) || maximumDb <= minimumDb)
            throw new ArgumentOutOfRangeException(nameof(maximumDb));
        if (!ValidAudioSize(length)) throw new ArgumentOutOfRangeException(nameof(length));
        if (!ValidAudioSize(thickness)) throw new ArgumentOutOfRangeException(nameof(thickness));
        if (!float.IsFinite(hold)) throw new ArgumentOutOfRangeException(nameof(hold));
        if (!ValidAudioSize(decay)) throw new ArgumentOutOfRangeException(nameof(decay));

        fixed (float* peaks = levels)
        {
            int cleared = NativeMethods.LevelMeter(context, Stable(id), peaks, levels.Length, minimumDb, maximumDb,
                length, thickness, horizontal ? 1 : 0, scale ? 1 : 0, segmented ? 1 : 0,
                hold, decay, enabled ? 1 : 0, out int node);
            if (track) TrackControl(id, node);
            return cleared != 0;
        }
    }
}

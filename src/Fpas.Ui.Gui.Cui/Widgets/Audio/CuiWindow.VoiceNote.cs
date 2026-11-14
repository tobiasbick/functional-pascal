using Fpas.Ui.Gui.Cui.Widgets.Audio;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Resamples recorded levels using CUI's native peak buckets.</summary>
    /// <param name="levels">Original recording levels.</param>
    /// <param name="columns">Number of samples to retain.</param>
    /// <returns>Resampled peaks.</returns>
    public static unsafe float[] ResampleVoice(float[] levels, int columns)
    {
        ArgumentNullException.ThrowIfNull(levels);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(columns);
        var output = new float[columns];
        fixed (float* source = levels, destination = output)
            NativeMethods.ResampleVoice(source, levels.Length, destination, columns);
        return output;
    }

    /// <summary>Draws a recorded message and returns native transport proposals.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="levels">Measured levels, newest last.</param>
    /// <param name="duration">Clip duration in seconds.</param>
    /// <param name="position">Played seconds.</param>
    /// <param name="speed">Playback multiplier.</param>
    /// <param name="playing">Current transport state.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="focus">Requested native part index or minus one.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe CuiVoiceNoteResult VoiceNote(string id, float[] levels, float duration, float position, float speed, bool playing,
        int widthKind, float width, int flags, int focus, Func<int, string>? track)
    {
        CheckFrame();
        CheckVoiceSignal(levels, null);
        CheckExtent(widthKind, width, nameof(width));
        int active = playing ? 1 : 0;
        int* nodes = stackalloc int[3];
        int events;
        fixed (float* signal = levels)
            events = NativeMethods.VoiceNote(context, Stable(id), signal, levels.Length, duration, &position, &speed,
                &active, widthKind, width, flags, focus, nodes);
        return new(events, position, speed, active != 0, TrackAiParts(nodes, 3, track));
    }
}

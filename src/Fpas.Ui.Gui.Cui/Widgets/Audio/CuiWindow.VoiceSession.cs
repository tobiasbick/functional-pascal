// [Documentation](../../../../docs/ui/gui/voice/voice-session.md)
using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Audio;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native call and returns any mute or end action.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="activity">Native voice activity from idle (0) through muted (4).</param>
    /// <param name="levels">Measured levels, oldest first. Negative values display silence.</param>
    /// <param name="owners">Speaker tags, one per level. Null uses the caller color.</param>
    /// <param name="text">Transcript and labels.</param>
    /// <param name="muted">Mute state, or null to omit the button.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Bit 1 enables input. Bit 2 hides End.</param>
    /// <param name="focus">Request focus for Mute (0) or End (1). Use minus one for no request.</param>
    /// <param name="track">Optional tracking callback for the visible Mute and End parts.</param>
    /// <returns>The action flags, proposed mute state and presence of each button.</returns>
    public unsafe CuiVoiceSessionResult VoiceSession(string id, int activity, float[] levels, byte[]? owners,
        CuiVoiceSessionText text, bool? muted, int widthKind, float width, int flags, int focus, Func<int, string>? track)
    {
        CheckFrame();
        CheckVoiceSignal(levels, owners);
        CheckExtent(widthKind, width, nameof(width));
        if (text.Transcript is not null) Utf8.Validate(text.Transcript, nameof(text));
        if (text.Upcoming is not null) Utf8.Validate(text.Upcoming, nameof(text));
        int value = muted.GetValueOrDefault() ? 1 : 0;
        int* nodes = stackalloc int[2];
        int events;
        fixed (float* signal = levels)
        fixed (byte* tags = owners)
            events = NativeMethods.VoiceSession(context, Stable(id), activity, signal, tags, levels.Length,
                OptionalAi(text.Label), text.Transcript, text.Upcoming, text.Speaker,
                OptionalAi(text.You), OptionalAi(text.Assistant), muted.HasValue ? &value : null,
                widthKind, width, flags, focus, nodes);
        return new(events, value != 0, TrackAiParts(nodes, 2, track));
    }
}

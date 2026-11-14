using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Audio;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native call and returns any mute or end action.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="activity">Native voice activity.</param>
    /// <param name="levels">Measured levels, newest last.</param>
    /// <param name="owners">Optional speaker tags.</param>
    /// <param name="text">Transcript and labels.</param>
    /// <param name="muted">Mute state, or null to omit the button.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="focus">Requested native part index or minus one.</param>
    /// <param name="track">Optional part tracking callback.</param>
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

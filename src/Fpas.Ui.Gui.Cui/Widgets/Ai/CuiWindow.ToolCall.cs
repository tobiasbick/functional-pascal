using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a tool call and reports whether its details are open.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="name">Tool caption.</param>
    /// <param name="status">Native task state.</param>
    /// <param name="icon">Optional glyph.</param>
    /// <param name="detail">Supporting text.</param>
    /// <param name="input">Input payload.</param>
    /// <param name="output">Output or error payload.</param>
    /// <param name="seconds">Elapsed seconds.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe bool ToolCall(string id, string name, int status, string icon, string detail, string input, string output,
        float seconds, int flags, string? track)
    {
        CheckFrame();
        Utf8.Validate(name, nameof(name));
        int node;
        int open = NativeMethods.ToolCall(context, Stable(id), name, status, OptionalAi(icon), OptionalAi(detail),
            OptionalAi(input), OptionalAi(output), seconds, flags, &node);
        if (track is not null) TrackControl(track, node);
        return open != 0;
    }
}

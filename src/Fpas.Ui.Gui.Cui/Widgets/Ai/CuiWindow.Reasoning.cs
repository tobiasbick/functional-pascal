using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a reasoning header and opens the animated body if present.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="thinking">Whether reasoning is in progress.</param>
    /// <param name="seconds">Elapsed seconds.</param>
    /// <param name="title">Optional heading.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="header">Optional tracked header identity.</param>
    public unsafe int BeginReasoning(string id, bool thinking, float seconds, string title, int flags, string? header)
    {
        CheckFrame();
        int* nodes = stackalloc int[2];
        int state = NativeMethods.ReasoningBegin(context, Stable(id), thinking ? 1 : 0, seconds, OptionalAi(title), flags, nodes);
        if (header is not null) TrackControl(header, nodes[1]);
        if (state != 0) layouts.Add(new(Layout.Reasoning, Node: nodes[0], Data: nodes[1]));
        return state;
    }

    /// <summary>Closes the current reasoning body.</summary>
    public void EndReasoning()
    {
        var scope = RequireLayout(Layout.Reasoning);
        NativeMethods.ReasoningEnd(context, scope.Node, (int)scope.Data!);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

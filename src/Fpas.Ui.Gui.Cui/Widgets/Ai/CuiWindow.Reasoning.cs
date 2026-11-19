// [Documentation](../../../../docs/ui/gui/ai/reasoning.md)
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
    /// <returns>Bit 1 when the body must be drawn and closed with <see cref="EndReasoning"/>;
    /// bit 2 when the body is open. A closing animation can return only bit 1.</returns>
    public unsafe int BeginReasoning(string id, bool thinking, float seconds, string title, int flags, string? header)
    {
        CheckFrame();
        int* nodes = stackalloc int[2];
        int state = NativeMethods.ReasoningBegin(context, Stable(id), thinking ? 1 : 0, seconds, OptionalAi(title), flags, nodes);
        if (header is not null) TrackControl(header, nodes[1]);
        if (state != 0) layouts.Add(new(Layout.Reasoning, Node: nodes[0], Data: nodes[1]));
        return state;
    }

    /// <summary>Closes a body drawn when <see cref="BeginReasoning"/> returned bit 1, including its closing animation.</summary>
    public void EndReasoning()
    {
        var scope = RequireLayout(Layout.Reasoning);
        NativeMethods.ReasoningEnd(context, scope.Node, (int)scope.Data!);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native response toolbar proposal.</summary>
/// <param name="Events">Native event mask.</param>
/// <param name="Vote">Proposed vote.</param>
/// <param name="Parts">Visible action controls.</param>
public sealed record CuiResponseResult(int Events, int Vote, bool[] Parts);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native response toolbar and returns its events.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="copyText">Clipboard text, or null.</param>
    /// <param name="vote">Model vote, or null.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe CuiResponseResult ResponseActions(string id, string? copyText, int? vote, int flags, Func<int, string>? track)
    {
        CheckFrame();
        if (copyText is not null) Utf8.Validate(copyText, nameof(copyText));
        int next = vote.GetValueOrDefault();
        int* nodes = stackalloc int[5];
        int events = NativeMethods.ResponseActions(context, Stable(id), copyText, vote.HasValue ? &next : null, flags, nodes);
        return new(events, next, TrackAiParts(nodes, 5, track));
    }
}

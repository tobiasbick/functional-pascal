using Fpas.Ui.Gui.Cui.Interop.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native source cards and returns the clicked index or minus one.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="sources">Cited documents.</param>
    /// <param name="enabled">Accepts input.</param>
    /// <param name="focus">Requested native part index or minus one.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe int SourceList(string id, CuiSourceItem[] sources, bool enabled, int focus, Func<int, string>? track)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var data = new NativeSource[sources.Length];
        for (int i = 0; i < data.Length; i++) data[i] = NativeCitation(memory, sources[i]);
        int[] nodes = new int[data.Length];
        int picked;
        fixed (NativeSource* items = data)
        fixed (int* parts = nodes)
        {
            picked = NativeMethods.SourceList(context, Stable(id), items, data.Length, enabled ? 1 : 0, focus, parts);
            TrackAiParts(parts, nodes.Length, track);
        }
        return picked;
    }
}

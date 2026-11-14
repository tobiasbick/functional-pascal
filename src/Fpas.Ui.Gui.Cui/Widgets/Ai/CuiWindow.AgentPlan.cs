using Fpas.Ui.Gui.Cui.Interop.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native plan and returns the clicked step or minus one.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="steps">Plan steps.</param>
    /// <param name="title">Optional heading.</param>
    /// <param name="enabled">Accepts input.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe int AgentPlan(string id, CuiPlanStep[] steps, string title, bool enabled, Func<int, string>? track)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var data = new NativePlanStep[steps.Length];
        for (int i = 0; i < data.Length; i++)
            data[i] = new() { Title = AiText(memory, steps[i].Title), Detail = AiText(memory, steps[i].Detail, true), Status = steps[i].Status };
        int[] nodes = new int[steps.Length];
        int picked;
        fixed (NativePlanStep* items = data)
        fixed (int* parts = nodes)
        {
            picked = NativeMethods.AgentPlan(context, Stable(id), items, steps.Length, OptionalAi(title), enabled ? 1 : 0, parts);
            TrackAiParts(parts, nodes.Length, track);
        }
        return picked;
    }
}

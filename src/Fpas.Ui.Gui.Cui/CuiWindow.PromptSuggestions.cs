using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native suggestions and returns the selected index or minus one.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="suggestions">Prompt suggestion cards.</param>
    /// <param name="columns">Grid column count or zero.</param>
    /// <param name="enabled">Accepts input.</param>
    /// <param name="focus">Requested native part index or minus one.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe int PromptSuggestions(string id, CuiSuggestion[] suggestions, int columns, bool enabled, int focus, Func<int, string>? track)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var data = new NativeSuggestion[suggestions.Length];
        for (int i = 0; i < data.Length; i++)
            data[i] = new() { Title = AiText(memory, suggestions[i].Title), Detail = AiText(memory, suggestions[i].Detail, true),
                Icon = AiText(memory, suggestions[i].Icon, true) };
        int[] nodes = new int[data.Length];
        int picked;
        fixed (NativeSuggestion* items = data)
        fixed (int* parts = nodes)
        {
            picked = NativeMethods.PromptSuggestions(context, Stable(id), items, data.Length, columns, enabled ? 1 : 0, focus, parts);
            TrackAiParts(parts, nodes.Length, track);
        }
        return picked;
    }
}

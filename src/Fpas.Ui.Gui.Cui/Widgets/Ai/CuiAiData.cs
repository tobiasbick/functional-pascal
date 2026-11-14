using Fpas.Ui.Gui.Cui.Interop.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static string? OptionalAi(string? value)
    {
        if (string.IsNullOrEmpty(value)) return null;
        Utf8.Validate(value, nameof(value));
        return value;
    }

    private static unsafe byte* AiText(ChartMemory memory, string value, bool optional = false)
    {
        Utf8.Validate(value, nameof(value));
        return memory.Text(optional && value.Length == 0 ? null : value);
    }

    private static unsafe NativeSource NativeCitation(ChartMemory memory, CuiSourceItem source) => new()
    {
        Title = AiText(memory, source.Title, true), Url = AiText(memory, source.Url),
        Snippet = AiText(memory, source.Snippet, true)
    };

    private static unsafe NativeModel* NativeModels(ChartMemory memory, CuiModelItem[] models)
    {
        var data = new NativeModel[models.Length];
        for (int i = 0; i < data.Length; ++i)
            data[i] = new() { Name = AiText(memory, models[i].Name), Detail = AiText(memory, models[i].Detail, true),
                Badge = AiText(memory, models[i].Badge, true), Icon = AiText(memory, models[i].Icon, true) };
        return memory.Copy<NativeModel>(data);
    }

    private unsafe bool[] TrackAiParts(int* nodes, int count, Func<int, string>? track)
    {
        var present = new bool[count];
        for (int i = 0; i < count; ++i)
        {
            present[i] = nodes[i] >= 0;
            if (present[i] && track is not null) TrackControl(track(i), nodes[i]);
        }
        return present;
    }

    private static void CheckVoiceSignal(float[] levels, byte[]? owners)
    {
        ArgumentNullException.ThrowIfNull(levels);
        if (owners is not null && owners.Length != levels.Length)
            throw new ArgumentException("Each level needs a speaker tag.", nameof(owners));
    }
}

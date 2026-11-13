using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Text supplied to the voice session.</summary>
/// <param name="Label">Optional activity caption.</param>
/// <param name="Transcript">Spoken transcript.</param>
/// <param name="Upcoming">Upcoming words.</param>
/// <param name="Speaker">Native speaker index.</param>
/// <param name="You">Local speaker name.</param>
/// <param name="Assistant">Remote speaker name.</param>
public sealed record CuiVoiceSessionText(string Label, string? Transcript, string? Upcoming, int Speaker, string You, string Assistant);

/// <summary>Native composer layout and recording options.</summary>
/// <param name="Placeholder">Text when empty.</param>
/// <param name="MaxUtf8Bytes">Capacity excluding the terminator.</param>
/// <param name="MaxLines">Visible lines or zero.</param>
/// <param name="WidthKind">Encoded native width.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Seconds">Recorded seconds.</param>
public sealed record CuiComposerOptions(string Placeholder, int MaxUtf8Bytes, int MaxLines, int WidthKind, float Width, float Seconds);

/// <summary>A native model choice.</summary>
/// <param name="Name">Visible model name.</param>
/// <param name="Detail">Optional description.</param>
/// <param name="Badge">Optional badge.</param>
/// <param name="Icon">Optional icon.</param>
public sealed record CuiModelItem(string Name, string Detail, string Badge, string Icon);

/// <summary>A native source card.</summary>
/// <param name="Url">Source address.</param>
/// <param name="Title">Optional title.</param>
/// <param name="Snippet">Optional excerpt.</param>
public sealed record CuiSourceItem(string Url, string Title, string Snippet);

/// <summary>A native plan step.</summary>
/// <param name="Title">Step caption.</param>
/// <param name="Detail">Optional step detail.</param>
/// <param name="Status">Native task status.</param>
public sealed record CuiPlanStep(string Title, string Detail, int Status);

/// <summary>A native suggestion.</summary>
/// <param name="Title">Suggestion title.</param>
/// <param name="Detail">Optional description.</param>
/// <param name="Icon">Optional icon.</param>
public sealed record CuiSuggestion(string Title, string Detail, string Icon);

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

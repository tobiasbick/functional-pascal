using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native editor or recorder and returns its proposed changes.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="draft">Current draft.</param>
    /// <param name="attachments">Attached file names.</param>
    /// <param name="models">Available model choices.</param>
    /// <param name="selected">Current model index.</param>
    /// <param name="levels">Measured levels, newest last.</param>
    /// <param name="options">Native layout and recording values.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    /// <param name="focus">Requested native part index or minus one.</param>
    /// <param name="track">Optional part tracking callback.</param>
    public unsafe CuiComposerResult Composer(string id, string draft, string[] attachments, CuiModelItem[] models, int selected,
        float[]? levels, CuiComposerOptions options, int flags, int focus, Func<int, string>? track)
    {
        CheckFrame();
        Utf8.Validate(draft, nameof(draft));
        int length = Utf8.Encoding.GetByteCount(draft);
        if (options.MaxUtf8Bytes is < 1 or > 1_048_576 || length > options.MaxUtf8Bytes || draft.Contains('\r'))
            throw new ArgumentException("The draft must fit its UTF-8 capacity and use LF line endings.", nameof(draft));
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        if (levels is not null) CheckVoiceSignal(levels, null);
        byte[] buffer = new byte[options.MaxUtf8Bytes + 1];
        Utf8.Encoding.GetBytes(draft, buffer);
        using var memory = new ChartMemory();
        foreach (string attachment in attachments) Utf8.Validate(attachment, nameof(attachments));
        int removed = -1;
        int* nodes = stackalloc int[4];
        int events;
        fixed (byte* text = buffer)
        fixed (float* samples = levels)
            events = NativeMethods.Composer(context, Stable(id), text, (nuint)buffer.Length, OptionalAi(options.Placeholder),
                memory.Labels(attachments), attachments.Length, NativeModels(memory, models), models.Length, &selected,
                samples, levels?.Length ?? 0, options.Seconds, options.MaxLines, options.WidthKind, options.Width,
                flags, focus, &removed, nodes);
        int end = Array.IndexOf(buffer, (byte)0);
        return new(Utf8.Encoding.GetString(buffer.AsSpan(0, end)), events, selected, removed, TrackAiParts(nodes, 4, track));
    }
}

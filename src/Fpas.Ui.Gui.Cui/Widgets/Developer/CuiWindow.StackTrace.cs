// [Documentation](../../../../docs/ui/gui/more-developer-tools/stack-trace.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native StackTrace widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Frames">Frames in display order, with optional source previews.</param>
    /// <param name="Title">Error heading, or empty to omit it.</param>
    /// <param name="Message">Error message, or empty to omit it.</param>
    /// <param name="Language">Syntax language for the source preview.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>The activated frame index, or minus one.</returns>
    public unsafe CuiDevResult StackTrace(string Id, CuiDevStackFrame[] Frames, string Title, string Message, string Language, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        Utf8.Validate(Title, nameof(Title));
        Utf8.Validate(Message, nameof(Message));
        Utf8.Validate(Language, nameof(Language));
        foreach (var frame in Frames)
        {
            if (frame.Line < 0 || frame.Column < 0 || frame.SourceFirst < 0 ||
                frame.SourceFirst > int.MaxValue - frame.Source.Length)
                throw new ArgumentOutOfRangeException(nameof(Frames));
            if (frame.FunctionName is { } function) Utf8.Validate(function, nameof(Frames));
            if (frame.File is { } file) Utf8.Validate(file, nameof(Frames));
            foreach (var line in frame.Source) Utf8.Validate(line, nameof(Frames));
        }
        using var memory = new ChartMemory();
        NativeMethods.DevStackTrace(context, Stable(Id), PackStackFrame(memory, Frames), Frames.Length, OptionalAi(Title), OptionalAi(Message), OptionalAi(Language), WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native StackTrace widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Frames">Frames supplied to CUI.</param>
    /// <param name="Title">Title supplied to CUI.</param>
    /// <param name="Message">Message supplied to CUI.</param>
    /// <param name="Language">Language supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult StackTrace(string Id, CuiDevStackFrame[] Frames, string Title, string Message, string Language, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevStackTrace(context, Stable(Id), PackStackFrame(memory, Frames), Frames.Length, OptionalAi(Title), OptionalAi(Message), OptionalAi(Language), WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}

// [Documentation](../../../../docs/ui/gui/more-developer-tools/http-response-status.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native response status.</summary>
    /// <param name="Id">Stable widget identity.</param>
    /// <param name="Status">HTTP status code, or a nonpositive value before a response.</param>
    /// <param name="Seconds">Elapsed time in seconds, or a negative value to omit it.</param>
    /// <param name="Bytes">Response size in bytes, or a negative value to omit it.</param>
    /// <param name="WidthKind">Native width extent kind.</param>
    /// <param name="Width">Value for the native width extent.</param>
    /// <param name="Enabled">Whether the host presents the widget as enabled.</param>
    public void ResponseStatus(string Id, int Status, double Seconds, double Bytes, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!double.IsFinite(Seconds)) throw new ArgumentOutOfRangeException(nameof(Seconds));
        if (!double.IsFinite(Bytes)) throw new ArgumentOutOfRangeException(nameof(Bytes));
        NativeMethods.DevResponseStatus(context, Stable(Id), Status, Seconds, Bytes, WidthKind, Width, Enabled ? 1 : 0);
    }
}

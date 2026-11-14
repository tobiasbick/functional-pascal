using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native ResponseStatus widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Status">Status supplied to CUI.</param>
    /// <param name="Seconds">Seconds supplied to CUI.</param>
    /// <param name="Bytes">Bytes supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult ResponseStatus(string Id, int Status, double Seconds, double Bytes, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        NativeMethods.DevResponseStatus(context, Stable(Id), Status, Seconds, Bytes, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}

using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native RequestBar widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Method">Method supplied to the widget.</param>
    /// <param name="Url">Url supplied to the widget.</param>
    /// <param name="Capacity">Capacity supplied to the widget.</param>
    /// <param name="Loading">Loading supplied to the widget.</param>
    /// <param name="Placeholder">Placeholder supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiRequestProposal RequestBar(string Id, int Method, string Url, int Capacity, bool Loading, string Placeholder, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        byte[] value = DevBuffer(Url, Capacity);
        fixed (byte* text = value)
        {
            NativeMethods.DevRequestBar(context, Stable(Id), Method, text, (nuint)value.Length, Loading ? 1 : 0,
                OptionalAi(Placeholder), WidthKind, Width, Enabled ? 1 : 0, out var result);
            return Enabled ? new(result.Index, ReadDevText(value), (result.Flags & 2) != 0, (result.Flags & 4) != 0)
                : new(Method, Url, false, false);
        }
    }
}

// [Documentation](../../../../docs/ui/gui/more-developer-tools/http-request-bar.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native RequestBar widget.</summary>
    /// <param name="Id">Stable widget identity.</param>
    /// <param name="Method">HTTP method index from 0 (GET) to 6 (OPTIONS).</param>
    /// <param name="Url">Current URL draft.</param>
    /// <param name="Capacity">Maximum URL length in UTF-8 bytes.</param>
    /// <param name="Loading">Whether the application has a request in progress.</param>
    /// <param name="Placeholder">Caption for an empty URL, or null for the native default.</param>
    /// <param name="WidthKind">Native width extent kind.</param>
    /// <param name="Width">Value for the native width extent.</param>
    /// <param name="Enabled">Whether the bar accepts input.</param>
    /// <param name="Focus">Whether to request keyboard focus for the URL field.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiRequestProposal RequestBar(string Id, int Method, string Url, int Capacity, bool Loading, string? Placeholder, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Method is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(Method));
        if (Placeholder is not null) Utf8.Validate(Placeholder, nameof(Placeholder));
        byte[] value = DevBuffer(Url, Capacity);
        fixed (byte* text = value)
        {
            NativeMethods.DevRequestBar(context, Stable(Id), Method, text, (nuint)value.Length, Loading ? 1 : 0,
                Placeholder, WidthKind, Width, (Enabled ? 1 : 0) | (Focus ? 2 : 0), out var result);
            return Enabled
                ? new(result.Index, ReadDevText(value), (result.Flags & 2) != 0, (result.Flags & 4) != 0, (result.Flags & 1) != 0)
                : new(Method, Url, false, false, false);
        }
    }
}

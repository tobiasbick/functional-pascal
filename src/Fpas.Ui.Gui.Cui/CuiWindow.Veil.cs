using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native VeilBegin operation.</summary>
    /// <param name="id">id for the native widget.</param>
    /// <param name="revealed">revealed for the native widget.</param>
    /// <param name="label">label for the native widget.</param>
    /// <param name="hint">hint for the native widget.</param>
    /// <param name="radius">radius for the native widget.</param>
    /// <param name="color">color for the native widget.</param>
    /// <param name="widthKind">widthKind for the native widget.</param>
    /// <param name="width">width for the native widget.</param>
    /// <param name="flags">flags for the native widget.</param>
    /// <returns>The native widget result.</returns>
    public unsafe int VeilBegin(string id, bool revealed, string? label, string? hint, float radius, uint color, int widthKind, float width, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        var state = NativeMethods.VeilBegin(context, Stable(id), revealed ? 1 : 0, label, hint, radius, color, widthKind, width, flags);
        layouts.Add(new(Layout.Veil));
        return state;
    }

    /// <summary>Invokes the native VeilEnd operation.</summary>
    public unsafe void VeilEnd()
    {
        CheckFrame();
        RequireLayout(Layout.Veil);
        NativeMethods.VeilEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

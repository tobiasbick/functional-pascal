// [Documentation](../../../../docs/ui/gui/disclosure/veil.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Charts;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native VeilBegin operation.</summary>
    /// <param name="id">id for the native widget.</param>
    /// <param name="revealed">revealed for the native widget.</param>
    /// <param name="label">label for the native widget.</param>
    /// <param name="hint">hint for the native widget.</param>
    /// <param name="radius">radius for the native widget.</param>
    /// <param name="color">Automatic, theme color, or explicit cover color.</param>
    /// <param name="widthKind">widthKind for the native widget.</param>
    /// <param name="width">width for the native widget.</param>
    /// <param name="flags">Native cover input/appearance flags. Composed bodies supply their own input permission.</param>
    /// <returns>The native widget result.</returns>
    public unsafe int VeilBegin(string id, bool revealed, string? label, string? hint, float radius,
        CuiChartColor color, int widthKind, float width, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        ValidateDistance(radius, nameof(radius));
        if (label is not null) Utf8.Validate(label, nameof(label));
        if (hint is not null) Utf8.Validate(hint, nameof(hint));
        if (!color.IsValid) throw new ArgumentOutOfRangeException(nameof(color));
        var state = NativeMethods.VeilBegin(context, Stable(id), revealed ? 1 : 0, label, hint,
            radius, color.Palette, color.Rgba, widthKind, width, flags);
        bool lifted = (state & 1) != 0;
        // A covered native veil retains an additional disabled scope above the
        // adapter's scope. Leave both intact until native VeilEnd balances them.
        if (lifted) NativeMethods.PopDisabled(context);
        layouts.Add(new(Layout.Veil, Node: lifted ? 1 : 0, Data: (flags & 1) != 0));
        return state;
    }

    /// <summary>Invokes the native VeilEnd operation.</summary>
    public unsafe void VeilEnd()
    {
        CheckFrame();
        var scope = RequireLayout(Layout.Veil);
        if (scope.Node != 0) RestoreContainerInput(Layout.Veil);
        NativeMethods.VeilEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

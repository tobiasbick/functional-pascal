// [Documentation](../../../../docs/ui/gui/canvas-and-bars/selection-ring.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Outlines the previous component with a collaborator's selection ring.</summary>
    /// <param name="tone">Native semantic tone ordinal; zero uses CUI's default when no explicit color is supplied.</param>
    /// <param name="rgba">Packed straight RGBA override, or zero to use the tone.</param>
    /// <param name="name">Optional collaborator flag at the component's top end corner.</param>
    /// <param name="radius">Corner radius in logical points, or zero to use the component's top-left corner radius.</param>
    public unsafe void SelectionRing(int tone, uint rgba, string? name, float radius)
    {
        CheckFrame();
        NativeMethods.SelectionRing(context, tone, rgba, name, radius);
    }
}

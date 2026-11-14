using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native ResultCard operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="image">image supplied to the native operation.</param>
    /// <param name="title">title supplied to the native operation.</param>
    /// <param name="caption">caption supplied to the native operation.</param>
    /// <param name="badge">badge supplied to the native operation.</param>
    /// <param name="tone">tone supplied to the native operation.</param>
    /// <param name="widthKind">widthKind supplied to the native operation.</param>
    /// <param name="width">width supplied to the native operation.</param>
    /// <param name="aspect">aspect supplied to the native operation.</param>
    /// <param name="enabled">enabled supplied to the native operation.</param>
    /// <param name="focus">focus supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe bool ResultCard(string id, CuiMediaImage image, string title, string? caption, string? badge, int tone, int widthKind, float width, float aspect, bool enabled, bool focus)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        fixed (byte* pixels = image.Pixels)
        {
            var source = MediaSource(image, memory, pixels);
            return NativeMethods.ResultCard(context, Stable(id), in source, title, caption, badge, tone,
                widthKind, width, aspect, enabled ? 1 : 0, focus ? 1 : 0) != 0;
        }
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Applies color and corner radius to the current container.</summary>
    /// <param name="rgba">Straight-alpha RGBA channels packed low to high.</param>
    /// <param name="radius">Corner radius in logical units.</param>
    public void SurfaceLast(uint rgba, float radius)
    {
        CheckFrame();
        if (!float.IsFinite(radius) || radius < 0) throw new ArgumentOutOfRangeException(nameof(radius));
        NativeMethods.Surface(context, NativeMethods.LastNode(context), rgba, radius);
    }

    /// <summary>Draws static text with an explicit typeface, size and color.</summary>
    /// <param name="text">Text to draw.</param>
    /// <param name="font">Native body, medium, display or mono face.</param>
    /// <param name="size">Logical size or zero for the theme default.</param>
    /// <param name="rgba">Straight-alpha RGBA channels packed low to high.</param>
    public void FormattedText(string text, int font, float size, uint rgba)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        if (font is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(font));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        NativeMethods.FormattedText(context, text, font, size, rgba);
    }

    /// <summary>Opens a native stack whose children paint over one another.</summary>
    /// <param name="alignment">Horizontal default/start/center/end/stretch, zero through four.</param>
    /// <param name="justification">Vertical start/center/end/between/around, zero through four.</param>
    /// <param name="clip">Whether to clip children to the stack.</param>
    public void BeginLayers(int alignment = 0, int justification = 0, bool clip = false)
    {
        CheckFrame();
        if (alignment is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(alignment));
        if (justification is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(justification));
        int node = NativeMethods.GroupBegin(context, 2, 0, 0, alignment, 0, 0, clip ? 1 : 0, justification);
        layouts.Add(new(Layout.Group, Node: node));
    }

    /// <summary>Sets native insets and reading direction immediately after opening a container.</summary>
    /// <param name="top">Nonnegative logical top inset.</param>
    /// <param name="end">Nonnegative logical end inset, mirrored by native RTL layout.</param>
    /// <param name="bottom">Nonnegative logical bottom inset.</param>
    /// <param name="start">Nonnegative logical start inset, mirrored by native RTL layout.</param>
    /// <param name="direction">Inherit zero, left-to-right one, right-to-left two.</param>
    public void LayoutLast(float top, float end, float bottom, float start, int direction = 0)
    {
        CheckFrame();
        foreach (float value in new[] { top, end, bottom, start })
            if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(nameof(top));
        if (direction is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(direction));
        int node = NativeMethods.LastNode(context);
        if (node < 0) throw new InvalidOperationException("Open a container before configuring its layout.");
        NativeMethods.BoxDetails(context, node, top, end, bottom, start, direction);
    }

    /// <summary>Inserts CUI's native flexible space along the containing axis.</summary>
    public void Spacer() { CheckFrame(); NativeMethods.Spacer(context); }

    /// <summary>Inserts CUI's native fixed space along the containing axis.</summary>
    /// <param name="distance">Finite nonnegative logical distance.</param>
    public void Gap(float distance)
    {
        CheckFrame();
        if (!float.IsFinite(distance) || distance < 0) throw new ArgumentOutOfRangeException(nameof(distance));
        NativeMethods.Gap(context, distance);
    }
}

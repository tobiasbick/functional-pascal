// [Documentation](../../../docs/ui/gui/cui.md#layout-and-control-inspection)
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

    /// <summary>Sets the native paint options for the open group.</summary>
    /// <param name="background">Packed RGBA background when no token is supplied.</param>
    /// <param name="backgroundToken">Optional CUI theme token, or zero for RGBA.</param>
    /// <param name="topLeft">Top left corner radius.</param>
    /// <param name="topRight">Top right corner radius.</param>
    /// <param name="bottomRight">Bottom right corner radius.</param>
    /// <param name="bottomLeft">Bottom left corner radius.</param>
    /// <param name="border">Stroke width in logical units.</param>
    /// <param name="borderColor">Optional packed RGBA stroke color.</param>
    /// <param name="borderToken">Theme token or zero for the default theme border.</param>
    /// <param name="elevation">Native shadow elevation, zero through three.</param>
    /// <param name="opacity">Paint opacity of the group and its content.</param>
    public void StyleGroup(uint background, int backgroundToken, float topLeft, float topRight,
        float bottomRight, float bottomLeft, float border, uint? borderColor, int borderToken,
        int elevation, float opacity)
    {
        var group = RequireLayout(Layout.Group);
        if (backgroundToken is < 0 or > ThemeColorCount)
            throw new ArgumentOutOfRangeException(nameof(backgroundToken));
        if (borderToken is < 0 or > ThemeColorCount || (borderToken != 0 && borderColor.HasValue))
            throw new ArgumentOutOfRangeException(nameof(borderToken));
        if (elevation is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(elevation));
        if (!float.IsFinite(opacity) || opacity is < 0 or > 1)
            throw new ArgumentOutOfRangeException(nameof(opacity));
        ValidateDistance(topLeft, nameof(topLeft));
        ValidateDistance(topRight, nameof(topRight));
        ValidateDistance(bottomRight, nameof(bottomRight));
        ValidateDistance(bottomLeft, nameof(bottomLeft));
        ValidateDistance(border, nameof(border));
        NativeMethods.GroupStyle(context, group.Node, background, backgroundToken, topLeft, topRight,
            bottomRight, bottomLeft, border, borderColor.GetValueOrDefault(),
            borderColor.HasValue ? -1 : borderToken, elevation, opacity);
    }

    private static void ValidateDistance(float distance, string name)
    {
        if (!float.IsFinite(distance) || distance < 0) throw new ArgumentOutOfRangeException(name);
    }

    /// <summary>Draws static text with an explicit typeface, size and color.</summary>
    /// <param name="text">Text to draw.</param>
    /// <param name="font">Native body, medium, display or mono face.</param>
    /// <param name="size">Logical size or zero for the theme default.</param>
    /// <param name="rgba">Straight-alpha RGBA channels packed low to high.</param>
    /// <param name="direction">Inherit, left-to-right, right-to-left, or automatic, in native order.</param>
    /// <param name="alignment">Start, center, right, left, or end, in native order.</param>
    /// <param name="noWrap">Truncate to a single line and show the full text on hover.</param>
    public void FormattedText(string text, int font, float size, uint rgba, int direction = 0, int alignment = 0,
        bool noWrap = false)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        if (font is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(font));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        if (direction is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(direction));
        if (alignment is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(alignment));
        NativeMethods.FormattedText(context, text, font, size, rgba, direction, alignment, noWrap ? 1 : 0);
    }

    /// <summary>Opens a native stack whose children paint over one another.</summary>
    /// <param name="alignment">Horizontal default/start/center/end/stretch, zero through four.</param>
    /// <param name="justification">Vertical start/center/end/between/around, zero through four.</param>
    /// <param name="clip">Whether to clip children to the stack.</param>
    /// <param name="id">Optional stable container ID.</param>
    /// <param name="scroll">None, vertical or both axes, in that order.</param>
    public void BeginLayers(int alignment = 0, int justification = 0, bool clip = false,
        string? id = null, int scroll = 0)
    {
        CheckFrame();
        if (alignment is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(alignment));
        if (justification is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(justification));
        OpenGroup(2, 0, 0, alignment, false, false, clip, justification, id, scroll);
    }

    /// <summary>Sets native insets and reading direction immediately after opening a container.</summary>
    /// <param name="top">Nonnegative logical top inset.</param>
    /// <param name="end">Nonnegative logical end inset, mirrored by native RTL layout.</param>
    /// <param name="bottom">Nonnegative logical bottom inset.</param>
    /// <param name="start">Nonnegative logical start inset, mirrored by native RTL layout.</param>
    /// <param name="direction">Inherit zero, left-to-right one, right-to-left two, or native automatic direction three.</param>
    public void LayoutLast(float top, float end, float bottom, float start, int direction = 0)
    {
        CheckFrame();
        foreach (float value in new[] { top, end, bottom, start })
            if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(nameof(top));
        if (direction is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(direction));
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

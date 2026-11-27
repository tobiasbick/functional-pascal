// [Documentation](../../../../docs/ui/gui/canvas-and-bars/painting.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>A canvas-local point in logical units.</summary>
/// <param name="X">Horizontal coordinate.</param>
/// <param name="Y">Vertical coordinate.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiPoint(float X, float Y);

/// <summary>Four independent corner radii in logical units.</summary>
/// <param name="TopLeft">Top-left radius.</param>
/// <param name="TopRight">Top-right radius.</param>
/// <param name="BottomRight">Bottom-right radius.</param>
/// <param name="BottomLeft">Bottom-left radius.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiCorners(float TopLeft, float TopRight, float BottomRight, float BottomLeft)
{
    /// <summary>Sets all four corners to the same radius.</summary>
    /// <param name="radius">Radius in logical units.</param>
    public CuiCorners(float radius) : this(radius, radius, radius, radius) { }
}

/// <summary>A texture-coordinate region. Zero through one spans each image axis.</summary>
/// <param name="U">Coordinate at the left edge of the destination.</param>
/// <param name="V">Coordinate at the top edge of the destination.</param>
/// <param name="Width">Signed horizontal span.</param>
/// <param name="Height">Signed vertical span.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiImageRegion(float U, float V, float Width, float Height);

/// <summary>Native text metrics in logical units.</summary>
/// <param name="Width">Advance width.</param>
/// <param name="Height">Native measured block height.</param>
/// <param name="Ascent">Distance above the baseline.</param>
/// <param name="Descent">Signed distance below the baseline.</param>
/// <param name="LineHeight">Recommended line advance.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiTextMetrics(float Width, float Height, float Ascent, float Descent, float LineHeight);

/// <summary>Current canvas pointer state; emitted only when state or active coordinates change.</summary>
/// <param name="Hovered">Whether native hit testing hovers this enabled canvas.</param>
/// <param name="Pressed">Whether CUI reports an active pointer press.</param>
/// <param name="X">Canvas-local horizontal position.</param>
/// <param name="Y">Canvas-local vertical position.</param>
/// <param name="Width">Previously laid-out canvas width.</param>
/// <param name="Height">Previously laid-out canvas height.</param>
public readonly record struct CuiCanvasPointer(bool Hovered, bool Pressed, float X, float Y, float Width, float Height);

/// <summary>Direct CUI drawing access, valid only during one owner-thread canvas callback.</summary>
public sealed class CuiCanvasPainter
{
    private readonly int thread = Environment.CurrentManagedThreadId;
    private readonly nint context;
    private readonly CuiRectangle area;
    private readonly uint images, wheel;
    private readonly int state;
    private bool active = true;
    private int clips, imageIndex;

    internal CuiCanvasPainter(nint context, CuiRectangle area, uint images, int state, uint wheel)
        => (this.context, this.area, this.images, this.state, this.wheel) = (context, area, images, state, wheel);

    internal bool HasWheelRegion { get; private set; }

    /// <summary>Moves a canvas-local value toward a target with CUI's standard spring.</summary>
    /// <param name="id">Stable animation ID within this canvas. Sample it once per frame.</param>
    /// <param name="target">Finite destination value.</param>
    /// <returns>The current value, with a 0.4-second spring and no bounce.</returns>
    public float Animate(string id, float target) => AnimateValue(id, target, 0, 0);

    /// <summary>Moves a canvas-local value toward a target with an adjustable spring.</summary>
    /// <param name="id">Stable animation ID within this canvas. Sample it once per frame.</param>
    /// <param name="target">Finite destination value.</param>
    /// <param name="duration">Positive, finite duration in seconds.</param>
    /// <param name="bounce">Bounce from zero through one.</param>
    /// <returns>The current value. Reduced motion returns the target immediately.</returns>
    public float AnimateSpring(string id, float target, float duration, float bounce)
    {
        if (!float.IsFinite(duration) || duration <= 0) throw new ArgumentOutOfRangeException(nameof(duration));
        if (!float.IsFinite(bounce) || bounce is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(bounce));
        return AnimateValue(id, target, duration, bounce);
    }

    private float AnimateValue(string id, float target, float duration, float bounce)
    {
        Check();
        ArgumentException.ThrowIfNullOrEmpty(id);
        Utf8.Validate(id, nameof(id));
        if (!float.IsFinite(target)) throw new ArgumentOutOfRangeException(nameof(target));
        return NativeMethods.CanvasAnimate(context, images, id, target, duration, bounce);
    }

    /// <summary>Gets the final laid-out logical width.</summary>
    public float Width { get { Check(); return area.Width; } }
    /// <summary>Gets the final laid-out logical height.</summary>
    public float Height { get { Check(); return area.Height; } }
    /// <summary>Gets the disabled scope captured by CUI when constructing the canvas.</summary>
    public bool Disabled { get { Check(); return (state & 1) != 0; } }
    /// <summary>Gets whether this canvas has native focus.</summary>
    public bool Focused { get { Check(); return (state & 2) != 0; } }
    /// <summary>Gets whether CUI requests a visible keyboard-focus indicator.</summary>
    public bool FocusVisible { get { Check(); return (state & 4) != 0; } }

    /// <summary>Requests the window cursor for this frame. The last CUI cursor request takes priority.</summary>
    /// <param name="cursor">Native cursor ordinal from zero through six.</param>
    public void SetCursor(int cursor)
    {
        Check();
        if (cursor is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(cursor));
        NativeMethods.CanvasCursor(context, cursor);
    }

    /// <summary>Adds a clipped rectangle for the canvas wheel callback, replacing the default full region.</summary>
    /// <param name="x">Canvas-local left edge.</param>
    /// <param name="y">Canvas-local top edge.</param>
    /// <param name="width">Nonnegative logical width.</param>
    /// <param name="height">Nonnegative logical height.</param>
    public void AddWheelRegion(float x, float y, float width, float height)
    {
        Check();
        if (!float.IsFinite(area.X + x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(area.Y + y)) throw new ArgumentOutOfRangeException(nameof(y));
        Distance(width);
        Distance(height);
        HasWheelRegion = true;
        if (wheel != 0 && width > 0 && height > 0)
            NativeMethods.CanvasWheelRegion(context, wheel,
                new CuiRectangle { X = area.X + x, Y = area.Y + y, Width = width, Height = height });
    }

    /// <summary>Gets CUI's user text multiplier. Explicit drawing sizes are logical points; apply this multiplier when desired.</summary>
    public float FontScale { get { Check(); return NativeMethods.CanvasFontScale(context); } }
    /// <summary>Gets CUI's monotonic frame time in seconds.</summary>
    public double Time { get { Check(); return NativeMethods.CanvasTime(context); } }
    /// <summary>Gets the native frame duration in seconds, capped at 0.25.</summary>
    public float DeltaTime { get { Check(); return NativeMethods.CanvasDeltaTime(context); } }

    /// <summary>Resolves a semantic token from the current interpolated palette.</summary>
    /// <param name="token">CUI color token ordinal.</param>
    /// <returns>Packed straight RGBA.</returns>
    public uint Color(int token)
    {
        Check();
        if (token is < 1 or > CuiWindow.ThemeColorCount) throw new ArgumentOutOfRangeException(nameof(token));
        return NativeMethods.CanvasColor(context, token);
    }

    private void Check()
    {
        if (!active) throw new InvalidOperationException("The canvas paint callback has ended.");
        if (Environment.CurrentManagedThreadId != thread)
            throw new InvalidOperationException("Paint only on the canvas callback's owner thread.");
    }

    private unsafe void Primitive(int kind, uint color, ReadOnlySpan<float> values, uint second = 0,
        string? text = null, CuiCorners radii = default)
    {
        Check();
        foreach (float value in values)
            if (!float.IsFinite(value)) throw new ArgumentException("Drawing coordinates must be finite.", nameof(values));
        fixed (float* data = values)
            NativeMethods.CanvasPrimitive(context, kind, area.X, area.Y, data, color, second, text, radii);
    }

    private static void Distance(float value)
    {
        if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(nameof(value));
    }

    private static void RoundedSize(float width, float height, CuiCorners radii)
    {
        Distance(width); Distance(height);
        Distance(radii.TopLeft); Distance(radii.TopRight); Distance(radii.BottomRight); Distance(radii.BottomLeft);
    }

    /// <summary>Fills a rounded rectangle using canvas-local coordinates.</summary>
    /// <param name="x">Left edge.</param><param name="y">Top edge.</param>
    /// <param name="width">Nonnegative width.</param><param name="height">Nonnegative height.</param>
    /// <param name="radius">Nonnegative corner radius.</param><param name="color">Packed straight RGBA.</param>
    public void Rectangle(float x, float y, float width, float height, float radius, uint color)
        => Rectangle(x, y, width, height, new CuiCorners(radius), color);

    /// <summary>Fills a rectangle with independent corner radii.</summary>
    /// <param name="x">Left edge.</param><param name="y">Top edge.</param>
    /// <param name="width">Nonnegative width.</param><param name="height">Nonnegative height.</param>
    /// <param name="radii">Nonnegative corner radii.</param><param name="color">Packed straight RGBA.</param>
    public void Rectangle(float x, float y, float width, float height, CuiCorners radii, uint color)
    { RoundedSize(width, height, radii); Primitive(0, color, [x, y, width, height], radii: radii); }

    /// <summary>Fills a circle in canvas-local coordinates.</summary>
    /// <param name="x">Center X.</param><param name="y">Center Y.</param>
    /// <param name="radius">Nonnegative radius.</param><param name="color">Packed straight RGBA.</param>
    public void Circle(float x, float y, float radius, uint color)
    { Distance(radius); Primitive(1, color, [x, y, radius]); }

    /// <summary>Draws a straight line in canvas-local coordinates.</summary>
    /// <param name="x">Start X.</param><param name="y">Start Y.</param>
    /// <param name="endX">End X.</param><param name="endY">End Y.</param>
    /// <param name="thickness">Nonnegative stroke width.</param><param name="color">Packed straight RGBA.</param>
    public void Line(float x, float y, float endX, float endY, float thickness, uint color)
    { Distance(thickness); Primitive(2, color, [x, y, endX, endY, thickness]); }

    /// <summary>Fills and outlines a rounded rectangle.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radius">Corner radius.</param><param name="fill">Packed fill RGBA.</param>
    /// <param name="border">Packed border RGBA.</param><param name="thickness">Nonnegative border width.</param>
    public void Border(float x, float y, float width, float height, float radius, uint fill, uint border, float thickness)
        => Border(x, y, width, height, new CuiCorners(radius), fill, border, thickness);

    /// <summary>Fills and outlines a rectangle with independent corner radii.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radii">Corner radii.</param><param name="fill">Packed fill RGBA.</param>
    /// <param name="border">Packed border RGBA.</param><param name="thickness">Nonnegative border width.</param>
    public void Border(float x, float y, float width, float height, CuiCorners radii, uint fill, uint border, float thickness)
    { RoundedSize(width, height, radii); Distance(thickness); Primitive(3, fill, [x, y, width, height, thickness], border, radii: radii); }

    /// <summary>Fills a rounded rectangle with CUI's vertical gradient.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radius">Corner radius.</param><param name="top">Packed top RGBA.</param><param name="bottom">Packed bottom RGBA.</param>
    public void Gradient(float x, float y, float width, float height, float radius, uint top, uint bottom)
        => Gradient(x, y, width, height, new CuiCorners(radius), top, bottom);

    /// <summary>Fills a rectangle with a vertical gradient and independent corner radii.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radii">Corner radii.</param><param name="top">Packed top RGBA.</param><param name="bottom">Packed bottom RGBA.</param>
    public void Gradient(float x, float y, float width, float height, CuiCorners radii, uint top, uint bottom)
    { RoundedSize(width, height, radii); Primitive(4, top, [x, y, width, height], bottom, radii: radii); }

    /// <summary>Draws a native shadow, clipped to the canvas and current clip scopes.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radius">Corner radius.</param><param name="blur">Nonnegative blur.</param><param name="spread">Signed expansion.</param>
    /// <param name="offsetX">Horizontal offset.</param><param name="offsetY">Vertical offset.</param><param name="color">Packed RGBA.</param>
    public void Shadow(float x, float y, float width, float height, float radius, float blur, float spread, float offsetX, float offsetY, uint color)
        => Shadow(x, y, width, height, new CuiCorners(radius), blur, spread, offsetX, offsetY, color);

    /// <summary>Draws a clipped shadow with independent corner radii.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="radii">Corner radii.</param><param name="blur">Nonnegative blur.</param><param name="spread">Signed expansion.</param>
    /// <param name="offsetX">Horizontal offset.</param><param name="offsetY">Vertical offset.</param><param name="color">Packed RGBA.</param>
    public void Shadow(float x, float y, float width, float height, CuiCorners radii, float blur, float spread, float offsetX, float offsetY, uint color)
    { RoundedSize(width, height, radii); Distance(blur); Primitive(5, color, [x, y, width, height, blur, spread, offsetX, offsetY], radii: radii); }

    /// <summary>Draws a circular arc; angles are in radians.</summary>
    /// <param name="x">Center X.</param><param name="y">Center Y.</param><param name="radius">Radius.</param>
    /// <param name="start">Start angle.</param><param name="end">End angle.</param><param name="thickness">Stroke width.</param><param name="color">Packed RGBA.</param>
    public void Arc(float x, float y, float radius, float start, float end, float thickness, uint color)
    { Distance(radius); Distance(thickness); Primitive(6, color, [x, y, radius, start, end, thickness]); }

    /// <summary>Draws Unicode text through one of the window's current font roles.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="text">Literal text.</param>
    /// <param name="font">Body=0 through icons=4.</param><param name="size">Positive logical text size.</param><param name="color">Packed RGBA.</param>
    public void Text(float x, float y, string text, int font, float size, uint color)
    { CheckFont(font, size); Utf8.Validate(text, nameof(text)); Primitive(7, color, [x, y, font, size], text: text); }

    /// <summary>Measures Unicode text with the same explicit logical size used to draw it.</summary>
    /// <param name="text">Literal text.</param><param name="font">Body=0 through icons=4.</param><param name="size">Positive logical text size.</param>
    /// <returns>Native advance and line metrics.</returns>
    public CuiTextMetrics Measure(string text, int font, float size)
    { Check(); CheckFont(font, size); Utf8.Validate(text, nameof(text)); return NativeMethods.CanvasMeasure(context, font, size, text); }

    private static void CheckFont(int font, float size)
    {
        if (font is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(font));
        if (!float.IsFinite(size) || size <= 0) throw new ArgumentOutOfRangeException(nameof(size));
    }

    /// <summary>Draws a CUI icon inside a canvas-local rectangle.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    /// <param name="icon">CUI icon glyph string.</param><param name="size">Positive icon size.</param><param name="color">Packed RGBA.</param>
    public void Icon(float x, float y, float width, float height, string icon, float size, uint color)
    { Distance(width); Distance(height); CheckFont(4, size); Utf8.Validate(icon, nameof(icon)); Primitive(8, color, [x, y, width, height, size], text: icon); }

    /// <summary>Draws a color emoji sequence with CUI's current emoji resource.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param><param name="text">Emoji sequence.</param>
    public void Emoji(float x, float y, float width, float height, string text)
    { Distance(width); Distance(height); Utf8.Validate(text, nameof(text)); Primitive(9, 0, [x, y, width, height], text: text); }

    /// <summary>Draws a closed or open polyline with CUI's native stroker.</summary>
    /// <param name="points">Canvas-local points.</param><param name="thickness">Nonnegative stroke width.</param>
    /// <param name="color">Packed RGBA.</param><param name="closed">Whether to join the last point to the first.</param>
    public void Polyline(CuiPoint[] points, float thickness, uint color, bool closed = false)
    { Distance(thickness); Points(points, 2, thickness, color, closed ? 1 : 0); }

    /// <summary>Fills a convex polygon using CUI's native tessellator.</summary>
    /// <param name="points">At least three canvas-local points in winding order.</param><param name="color">Packed RGBA.</param>
    public void Polygon(CuiPoint[] points, uint color) => Points(points, 3, 0, color, 2);

    private unsafe void Points(CuiPoint[] points, int minimum, float thickness, uint color, int kind)
    {
        Check(); ArgumentNullException.ThrowIfNull(points);
        if (points.Length < minimum || points.Any(p => !float.IsFinite(p.X) || !float.IsFinite(p.Y)))
            throw new ArgumentException("Provide enough finite polygon points.", nameof(points));
        fixed (CuiPoint* data = points) NativeMethods.CanvasPoints(context, area.X, area.Y, data, points.Length, thickness, color, kind);
    }

    /// <summary>Draws an image with a per-canvas native cache, retired when unused or removed.</summary>
    /// <param name="source">Immutable picture source.</param><param name="x">Left.</param><param name="y">Top.</param>
    /// <param name="width">Width.</param><param name="height">Height.</param><param name="radius">Corner radius.</param><param name="tint">Packed RGBA tint.</param>
    /// <param name="imageRegion">Texture-coordinate origin and signed spans. Null uses the whole image.</param>
    public void Image(CuiMediaImage source, float x, float y, float width, float height, float radius, uint tint,
        CuiImageRegion? imageRegion = null)
        => Image(source, x, y, width, height, new CuiCorners(radius), tint, imageRegion);

    /// <summary>Draws a cached image with independent corner radii and an optional texture region.</summary>
    /// <param name="source">Immutable picture source.</param><param name="x">Left.</param><param name="y">Top.</param>
    /// <param name="width">Width.</param><param name="height">Height.</param><param name="radii">Corner radii.</param><param name="tint">Packed RGBA tint.</param>
    /// <param name="imageRegion">Texture-coordinate origin and signed spans. Null uses the whole image.</param>
    public void Image(CuiMediaImage source, float x, float y, float width, float height, CuiCorners radii, uint tint,
        CuiImageRegion? imageRegion = null)
    {
        Check(); RoundedSize(width, height, radii);
        if (!float.IsFinite(x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(y)) throw new ArgumentOutOfRangeException(nameof(y));
        CuiImageRegion uv = imageRegion ?? new(0, 0, 1, 1);
        if (!float.IsFinite(uv.U) || !float.IsFinite(uv.V) ||
            !float.IsFinite(uv.U + uv.Width) || !float.IsFinite(uv.V + uv.Height))
            throw new ArgumentOutOfRangeException(nameof(imageRegion));
        CuiWindow.PaintImage(context, images, imageIndex, source, area.X + x, area.Y + y, width, height, radii, uv, tint);
        imageIndex++;
    }

    /// <summary>Intersects the current native clip with a canvas-local rectangle.</summary>
    /// <param name="x">Left.</param><param name="y">Top.</param><param name="width">Width.</param><param name="height">Height.</param>
    public void PushClip(float x, float y, float width, float height)
    {
        Check(); Distance(width); Distance(height);
        if (!float.IsFinite(x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(y)) throw new ArgumentOutOfRangeException(nameof(y));
        if (NativeMethods.CanvasPushClip(context, area.X + x, area.Y + y, width, height) == 0)
            throw new InvalidOperationException("The native clip stack is full.");
        clips++;
    }

    /// <summary>Restores a clip pushed by this callback, without escaping the native canvas clip.</summary>
    public void PopClip()
    {
        Check();
        if (clips == 0) throw new InvalidOperationException("No callback-local clip to pop.");
        NativeMethods.CanvasPopClip(context); clips--;
    }

    internal void Complete()
    {
        while (clips > 0) PopClip();
        NativeMethods.CanvasImagesFinish(context, images, imageIndex);
        active = false;
    }
}

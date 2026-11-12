using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Verifies native ring geometry, literal captions and managed ABI validation.</summary>
public sealed class CuiProgressRingTests
{
    /// <summary>Default and explicit diameters survive narrow-wide-narrow window changes.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void RingKeepsNativeDiameterAndDoesNotAcceptFocus(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Progress ring", 360, 220, renderer, continuous: true);
        window.SetReducedMotion(true);
        foreach (int width in new[] { 360, 240, 600, 240 })
        {
            window.Resize(width, 220);
            foreach (float size in new[] { 0, 64, 104 })
            {
                Draw(size); Draw(size);
                var box = window.GetControlRectangle("ring");
                Assert.Equal(size == 0 ? 96 : size, box.Width);
                Assert.Equal(box.Width, box.Height);
                Assert.False(window.ControlFocused("ring"));
            }
        }

        void Draw(float size)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12);
            window.ProgressRing("ring", "Quota", .35f, size);
            window.TrackLastControl("ring");
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>CUI identity syntax in a caption remains visible text in both renderers.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CaptionIsLiteral(CuiRenderer renderer)
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Literal ring", 300, 220, renderer, continuous: true);
        window.SetReducedMotion(true);
        string literal = directory.File("literal.png"), plain = directory.File("plain.png");
        Draw("Goal##key"); Draw("Goal##key"); window.CapturePng(literal);
        Draw("Goal"); Draw("Goal"); window.CapturePng(plain);
        Assert.False(PngAssertions.ReadPixels(literal, 300, 220)
            .SequenceEqual(PngAssertions.ReadPixels(plain, 300, 220)));

        void Draw(string label)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.ProgressRing("ring", label, .72f, 160);
            window.TrackLastControl("ring");
            window.Present();
        }
    }

    /// <summary>Rejected fractions, diameters and text leave the native frame usable.</summary>
    [GuiFact]
    public void InvalidRingValuesDoNotCorruptTheFrame()
    {
        using var window = new CuiWindow("Ring validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (float value in new[] { -.01f, 1.01f, float.NaN, float.PositiveInfinity })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.ProgressRing("bad", "Quota", value));
        foreach (float size in new[] { -1f, float.NaN, float.PositiveInfinity })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.ProgressRing("bad", "Quota", .5f, size));
        Assert.Throws<ArgumentException>(() => window.ProgressRing("bad", "zero\0", .5f));
        Assert.ThrowsAny<ArgumentException>(() => window.ProgressRing("", "Quota", .5f));
        window.ProgressRing("valid", "Quota", .5f);
        window.Present();
    }
}

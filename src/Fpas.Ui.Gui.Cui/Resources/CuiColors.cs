using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Resources;

/// <summary>CUI's native color-space helpers, independent of a window.</summary>
public static class CuiColors
{
    /// <summary>Converts OKLCH to a packed straight-alpha RGBA color.</summary>
    /// <param name="lightness">Zero through one.</param><param name="chroma">Nonnegative chroma.</param>
    /// <param name="hue">Finite angle in degrees.</param><param name="alpha">Zero through one.</param>
    /// <returns>CUI's converted RGBA bytes.</returns>
    public static uint Oklch(float lightness, float chroma, float hue, float alpha = 1)
    {
        Unit(lightness); Unit(alpha);
        if (!float.IsFinite(chroma) || chroma < 0 || !float.IsFinite(hue)) throw new ArgumentOutOfRangeException(nameof(chroma));
        return NativeMethods.Color(0, lightness, chroma, hue, alpha, 0, 0);
    }
    /// <summary>Mixes colors in CUI's OKLab space.</summary>
    /// <param name="left">Packed RGBA start.</param><param name="right">Packed RGBA end.</param>
    /// <param name="amount">Interpolation from zero through one.</param><returns>The mixed RGBA color.</returns>
    public static uint Mix(uint left, uint right, float amount)
    { Unit(amount); return NativeMethods.Color(1, amount, 0, 0, 0, left, right); }
    /// <summary>Multiplies alpha through CUI, clamped to its byte range.</summary>
    /// <param name="color">Packed straight RGBA.</param><param name="multiplier">Finite nonnegative multiplier.</param>
    /// <returns>The color with multiplied alpha.</returns>
    public static uint Alpha(uint color, float multiplier)
    {
        if (!float.IsFinite(multiplier) || multiplier < 0) throw new ArgumentOutOfRangeException(nameof(multiplier));
        return NativeMethods.Color(2, multiplier, 0, 0, 0, color, 0);
    }
    private static void Unit(float value)
    { if (!float.IsFinite(value) || value < 0 || value > 1) throw new ArgumentOutOfRangeException(nameof(value)); }
}

// [Documentation](../../../docs/ui/gui/cui.md#theme-and-resources)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    internal const int ThemeColorCount = 34;

    /// <summary>Gets the platform's app appearance, or null if CUI cannot determine it.</summary>
    public bool? SystemDark
    {
        get
        {
            CheckOpen();
            return NativeMethods.SystemDark() switch { 0 => false, 1 => true, _ => null };
        }
    }

    /// <summary>Applies the shared theme's semantic palette.</summary>
    /// <param name="dark">Whether to use dark typography and elevation defaults.</param>
    /// <param name="accent">CUI accent family: neutral=0 through rose=7.</param>
    /// <param name="colors">Exactly 34 packed RGBA colors in CUI semantic token order.</param>
    /// <param name="metrics">Optional 40-value typography/geometry/elevation contract; see ReadThemeMetrics.</param>
    /// <param name="animate">Use CUI's palette transition when the appearance or accent changes.</param>
    public unsafe void SetTheme(bool dark, int accent, uint[] colors, float[]? metrics = null, bool animate = false)
    {
        CheckOpen(); ArgumentNullException.ThrowIfNull(colors);
        ArgumentOutOfRangeException.ThrowIfNegative(accent);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(accent, 7);
        if (colors.Length != ThemeColorCount)
            throw new ArgumentException($"Expected {ThemeColorCount} CUI semantic colors.", nameof(colors));
        if (metrics is not null) ValidateThemeMetrics(metrics);
        fixed (uint* values = colors)
        fixed (float* settings = metrics)
            NativeMethods.SetTheme(context, dark ? 1 : 0, accent, values, settings, animate ? 1 : 0);
    }

    /// <summary>Copies the active palette in the same semantic order accepted by SetTheme.</summary>
    /// <returns>34 packed RGBA colors detached from native storage.</returns>
    public unsafe uint[] ReadTheme()
    {
        CheckOpen();
        var colors = new uint[ThemeColorCount];
        fixed (uint* values = colors) NativeMethods.ReadTheme(context, values);
        return colors;
    }

    /// <summary>Enables native appearance notifications and immediately adopts the available system mode.</summary>
    /// <param name="enabled">Whether CUI should follow platform appearance changes.</param>
    public void FollowSystemTheme(bool enabled) { CheckOpen(); NativeMethods.FollowTheme(app, enabled ? 1 : 0); }

    /// <summary>Copies native typography, geometry and elevation values.</summary>
    /// <returns>Radii (3), control heights (3), text sizes (4), headings (4), motion, font scale,
    /// then six shadow layers (y, blur, spread, alpha) in resting/raised/floating order.</returns>
    public unsafe float[] ReadThemeMetrics()
    {
        CheckOpen();
        var result = new float[40];
        fixed (float* values = result) NativeMethods.ThemeMetrics(context, values, 0);
        return result;
    }

    private static void ValidateThemeMetrics(float[] values)
    {
        if (values.Length != 40) throw new ArgumentException("Expected 40 CUI metric values.", nameof(values));
        for (int i = 0; i < values.Length; i++)
        {
            float value = values[i];
            if (!float.IsFinite(value)) throw new ArgumentException("CUI metrics must be finite.", nameof(values));
            if (i < 16 && (value < 0 && !(i == 14 && value == -1) || (i >= 3 && i != 14 && value == 0)))
                throw new ArgumentException("Sizes and scale must be positive; radii and motion nonnegative (-1 preserves motion).", nameof(values));
            if (i >= 16 && ((i - 16) % 4 == 1 && value < 0 && value != -1 || (i - 16) % 4 == 3 && (value < 0 || value > 1)))
                throw new ArgumentException("Shadow blur must be nonnegative (-1 preserves the native layer), and opacity between zero and one.", nameof(values));
        }
    }
}

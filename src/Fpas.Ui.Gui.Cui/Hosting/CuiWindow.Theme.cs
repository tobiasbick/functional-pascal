using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Gets the platform's app appearance, or null if CUI cannot determine it.</summary>
    public bool? SystemDark
    {
        get
        {
            CheckOpen();
            return NativeMethods.SystemDark() switch { 0 => false, 1 => true, _ => null };
        }
    }

    /// <summary>Applies the shared theme's semantic palette atomically.</summary>
    /// <param name="dark">Whether to use dark typography and elevation defaults.</param>
    /// <param name="accent">CUI accent family: neutral=0 through rose=7.</param>
    /// <param name="colors">Exactly 34 packed RGBA colors in CUI semantic token order.</param>
    public unsafe void SetTheme(bool dark, int accent, uint[] colors)
    {
        CheckOpen(); ArgumentNullException.ThrowIfNull(colors);
        ArgumentOutOfRangeException.ThrowIfNegative(accent);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(accent, 7);
        if (colors.Length != 34) throw new ArgumentException("Expected 34 CUI semantic colors.", nameof(colors));
        fixed (uint* values = colors) NativeMethods.SetTheme(context, dark ? 1 : 0, accent, values);
    }

    /// <summary>Copies the active palette in the same semantic order accepted by SetTheme.</summary>
    /// <returns>34 packed RGBA colors detached from native storage.</returns>
    public unsafe uint[] ReadTheme()
    {
        CheckOpen();
        var colors = new uint[34];
        fixed (uint* values = colors) NativeMethods.ReadTheme(context, values);
        return colors;
    }
}

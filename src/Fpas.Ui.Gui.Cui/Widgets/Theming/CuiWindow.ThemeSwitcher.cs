namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    // Lucide glyphs from the pinned CUI header, shared by appearance compositions.
    private const string SunIcon = "\uE178", MoonIcon = "\uE11E", MonitorIcon = "\uE11D";

    /// <summary>Draws CUI's appearance selector and returns a proposal without replacing the host palette.</summary>
    /// <param name="id">Stable group identity.</param>
    /// <param name="selected">Model-owned light, dark or system ordinal.</param>
    /// <param name="withSystem">Whether System is available.</param>
    /// <param name="enabled">Whether segments accept input and focus.</param>
    /// <param name="focus">Zero for no request; one through three select a focus segment.</param>
    /// <param name="trackingIds">Group and ordered segment inspection keys, or null.</param>
    /// <returns>Minus one without a change; otherwise the proposed choice ordinal.</returns>
    public int ThemeSwitcher(string id, int selected, bool withSystem, bool enabled, int focus, string[]? trackingIds)
    {
        return Segmented(id, withSystem ? ["", "", ""] : ["", ""],
            withSystem ? [SunIcon, MoonIcon, MonitorIcon] : [SunIcon, MoonIcon],
            selected, 1, false, enabled, focus, trackingIds);
    }
}

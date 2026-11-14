namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a controlled sun/moon action without changing the host palette.</summary>
    /// <param name="id">Stable identity independent of appearance and tooltip.</param>
    /// <param name="dark">Whether the model's resolved appearance is dark.</param>
    /// <param name="enabled">Whether input and focus are permitted.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether the user activated the action.</returns>
    public bool ThemeToggle(string id, bool dark, bool enabled, bool focus)
    {
        return IconButton(id, dark ? SunIcon : MoonIcon, dark ? "Switch to light" : "Switch to dark",
            "", 0, 0, enabled, false, false, focus);
    }
}

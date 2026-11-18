using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Launches targets through the platform's registered desktop handlers.</summary>
public static class CuiDesktop
{
    /// <summary>Asks the desktop to open a trusted URL or path with its default handler.</summary>
    /// <param name="target">A valid Unicode URL or absolute path.</param>
    /// <returns>Whether CUI could dispatch the request. The handler may finish asynchronously.</returns>
    public static bool TryOpen(string target)
    {
        Utf8.Validate(target, nameof(target));
        return NativeMethods.OpenUrl(target) != 0;
    }
}

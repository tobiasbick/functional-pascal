using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the CUI password strength meter.</summary>
    /// <param name="id">Stable meter identity.</param>
    /// <param name="password">Current model password.</param>
    /// <returns>Native visual strength score from zero to four.</returns>
    public int PasswordStrength(string id, string password)
    {
        CheckFrame();
        Utf8.Validate(password, nameof(password));
        return NativeMethods.PasswordStrength(context, Stable(id), password);
    }
}

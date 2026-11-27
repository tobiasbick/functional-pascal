// [Documentation](../../../../docs/ui/gui/feedback/avatar-group.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's overlapping avatars and its overflow count.</summary>
    /// <param name="names">One or more names in display order.</param>
    /// <param name="maximum">Visible avatar limit. Nonpositive values use four.</param>
    /// <param name="size">Logical diameter. Nonpositive values use 32.</param>
    public void AvatarGroup(string[] names, int maximum, float size)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(names);
        if (names.Length == 0) throw new ArgumentException("The group needs a name.", nameof(names));
        if (!float.IsFinite(size)) throw new ArgumentOutOfRangeException(nameof(size));
        foreach (string name in names) Utf8.Validate(name, nameof(names));
        NativeMethods.AvatarGroup(context, names, names.Length, maximum, size);
    }
}

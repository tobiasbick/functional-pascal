using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's overlapping avatars and its overflow count.</summary>
    /// <param name="names">One or more names in display order.</param>
    /// <param name="maximum">Maximum number of avatars before the overflow count.</param>
    /// <param name="size">Logical diameter of an avatar.</param>
    public void AvatarGroup(string[] names, int maximum, float size)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(names);
        if (names.Length == 0) throw new ArgumentException("The group needs a name.", nameof(names));
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(maximum);
        if (!float.IsFinite(size) || size <= 0) throw new ArgumentOutOfRangeException(nameof(size));
        foreach (string name in names) Utf8.Validate(name, nameof(names));
        NativeMethods.AvatarGroup(context, names, names.Length, maximum, size);
    }
}

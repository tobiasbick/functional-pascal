using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using System.Globalization;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static void CheckSmartWidth(int kind, float width)
    {
        if (kind is < 0 or > 3 || !float.IsFinite(width) || width < 0 || (kind == 3 && width > 100))
            throw new ArgumentOutOfRangeException(nameof(width));
    }

    private static void CheckMoneyValue(double value, int decimals)
    {
        if (decimals is < 0 or > 15 || !double.IsFinite(value))
            throw new ArgumentOutOfRangeException(nameof(value));
        Span<char> text = stackalloc char[64];
        if (!value.TryFormat(text, out int length, "F" + decimals.ToString(CultureInfo.InvariantCulture),
            CultureInfo.InvariantCulture))
            throw new ArgumentOutOfRangeException(nameof(value), "The amount exceeds CUI's display capacity.");
        int dot = text[..length].IndexOf('.');
        int integers = (dot < 0 ? length : dot) - (value < 0 ? 1 : 0);
        if (length + Math.Max(0, (integers - 1) / 3) > 63)
            throw new ArgumentOutOfRangeException(nameof(value), "The grouped amount exceeds CUI's display capacity.");
    }

    private static void CheckKeyCombination(CuiKeyCombination value)
    {
        if (value.Key is < 0 or >= 68 || value.Modifiers is < 0 or > 15 ||
            (value.Key == 0 && value.Modifiers != 0))
            throw new ArgumentOutOfRangeException(nameof(value));
    }
}

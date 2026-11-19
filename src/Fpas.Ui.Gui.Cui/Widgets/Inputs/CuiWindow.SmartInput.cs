// [Documentation](../../../../docs/ui/gui/smart-inputs/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static void CheckSmartWidth(int kind, float width)
    {
        if (kind is < 0 or > 3 || !float.IsFinite(width) || width < 0 || (kind == 3 && width > 100))
            throw new ArgumentOutOfRangeException(nameof(width));
    }

    private static void CheckKeyCombination(CuiKeyCombination value)
    {
        if (value.Key is < 0 or >= 68 || value.Modifiers is < 0 or > 15 ||
            (value.Key == 0 && value.Modifiers != 0))
            throw new ArgumentOutOfRangeException(nameof(value));
    }
}

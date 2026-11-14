using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native group whose buttons share an outline.</summary>
    public void BeginButtonGroup()
    {
        CheckFrame();
        layouts.Add(new(Layout.ButtonGroup, Node: NativeMethods.ButtonGroupBegin(context)));
    }

    /// <summary>Closes the innermost button group and makes it the last layout node.</summary>
    public void EndButtonGroup()
    {
        var group = RequireLayout(Layout.ButtonGroup);
        NativeMethods.ButtonGroupEnd(context, group.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

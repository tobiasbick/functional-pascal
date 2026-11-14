using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins CUI's right-aligned action footer.</summary>
    public void BeginActionFooter()
    {
        CheckFrame();
        layouts.Add(new(Layout.ActionFooter, Node: NativeMethods.ActionFooterBegin(context)));
    }

    /// <summary>Ends the footer and makes its row the last layout target.</summary>
    public void EndActionFooter()
    {
        var scope = RequireLayout(Layout.ActionFooter);
        NativeMethods.ActionFooterEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}

using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private void ResolveHostLayout()
    {
        if (scrollRequests.Count == 0) return;
        NativeMethods.ResolveLayout(context);
        foreach (var (id, request) in scrollRequests)
            if (scrollNodes.TryGetValue(id, out int node)) NativeMethods.ScrollLayout(context, node, request.Mode, request.Offset);
        NativeMethods.ResolveLayout(context);
    }
}

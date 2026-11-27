// [Documentation](../../../../docs/ui/gui/data-grid/tree-table.md#context-menu)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Offers a tree table's context menu after <see cref="EndTreeTable"/>; call it in every frame that offers one.</summary>
    /// <param name="id">The identity passed to <see cref="BeginTreeTable"/>.</param>
    /// <param name="enabled">Whether the menu may open and accept input.</param>
    /// <param name="node">The model node the open menu is for, or minus one.</param>
    /// <returns>Whether the menu is open. Always close the scope with <see cref="EndTreeTableMenu"/>.</returns>
    /// <remarks>A right click on a row opens it at the pointer and Shift+F10 opens it for the cursor row.
    /// A row outside the selection becomes the selection first.</remarks>
    public bool BeginTreeTableMenu(string id, bool enabled, out int node)
    {
        CheckFrame();
        ArgumentException.ThrowIfNullOrEmpty(id);
        bool visible = NativeMethods.TreeTableMenuBegin(context, Stable(id), enabled ? 1 : 0, out node, out uint popup, out int table) != 0;
        layouts.Add(new(Layout.TreeTableMenu, table, (id, popup, visible)));
        return visible;
    }

    /// <summary>Fills the open context menu and closes its scope.</summary>
    /// <param name="entries">Commands, labels and separators for the menu's node; ignored while it is closed.</param>
    /// <param name="trackingIds">The menu, measured by its table, followed by one identity per entry, or null.</param>
    /// <returns>The chosen entry, visibility and highlight.</returns>
    public unsafe CuiMenuResult EndTreeTableMenu(CuiMenuEntry[] entries, string[]? trackingIds)
    {
        var scope = RequireLayout(Layout.TreeTableMenu);
        var (id, popup, visible) = ((string, uint, bool))scope.Data!;
        var (keys, labels, icons, shortcuts, rowOptions) = PackMenuEntries(entries, trackingIds);
        int[] nodes = new int[entries.Length + 1];
        int choice, open, highlighted;
        uint owner;
        fixed (NativeMenuRow* rows = rowOptions)
        fixed (int* parts = nodes)
            choice = NativeMethods.TreeTableMenuEnd(context, popup, scope.Node, visible ? 1 : 0, keys, labels, icons, shortcuts,
                rows, entries.Length, parts, out open, out highlighted, out owner);
        layouts.RemoveAt(layouts.Count - 1);
        TrackModal(id + "/menu", owner);
        if (trackingIds is not null)
            for (int i = 0; i < nodes.Length; i++)
                if (nodes[i] >= 0)
                {
                    if (i == 0) trackedNodes.Add(trackingIds[i], nodes[i]);
                    else TrackControl(trackingIds[i], nodes[i]);
                }
        return new CuiMenuResult(choice, open != 0, highlighted);
    }
}

// [Documentation](../../../../docs/ui/gui/data-grid/tree-table.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>Frame output from the native tree table.</summary>
/// <param name="Activated">Activated node, or minus one.</param>
/// <param name="Load">Node indices requesting children.</param>
/// <param name="Expanded">Opened node, or minus one.</param>
/// <param name="Collapsed">Closed node, or minus one.</param>
/// <param name="SelectionChanged">Whether CUI reports a selection change.</param>
/// <param name="SortChanged">Whether CUI reports a sort change.</param>
/// <param name="Dropped">Ordered indices of released nodes.</param>
/// <param name="DropTarget">Target node, or minus one.</param>
/// <param name="DropPlacement">Before, inside, or after, or minus one.</param>
public readonly record struct CuiTreeTableEvents(int Activated, int[] Load, int Expanded,
    int Collapsed, bool SelectionChanged, bool SortChanged, int[] Dropped, int DropTarget, int DropPlacement);

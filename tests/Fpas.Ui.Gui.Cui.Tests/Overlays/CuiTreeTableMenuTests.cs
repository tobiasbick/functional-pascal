// [Documentation](../../../docs/ui/gui/data-grid/tree-table.md#context-menu)
using Fpas.Ui.Gui.Cui.Widgets.Collections;
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests.Overlays;

/// <summary>Checks the native per-node context menu of the virtual tree table.</summary>
public sealed class CuiTreeTableMenuTests
{
    /// <summary>Opens for the row under a right click, selects it, chooses by keyboard and reopens with Shift+F10.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void RowMenuFollowsPointerKeyboardAndSelection(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Tree table menu", 600, 400, renderer, continuous: true);
        window.SetReducedMotion(true);
        string[] labels = ["src", "main.c", "README.md"];
        int[] parents = [-1, 0, -1];
        uint[] flags = [2, 0, 0];
        int menuNode = -1;
        Draw();
        var row = window.GetControlRectangle("row/2");
        var point = new CuiRectangle { X = row.X + 24, Y = row.Y + row.Height / 2 };

        // CUI resolves hover at the start of a frame, so the pointer arrives a frame early.
        Draw(point);
        Draw(point, right: true);
        var opened = Draw(point, right: false);
        Assert.True(opened.Open);
        Assert.Equal(2, menuNode);
        Assert.Equal(4u, flags[2] & 4u);
        var table = window.GetControlRectangle("menu");
        Assert.True(table.Y <= row.Y && table.Y + table.Height >= row.Y + row.Height);
        Assert.True(window.GetControlRectangle("menu/open").Width > 0);

        Draw(key: 10);
        Assert.Equal(0, Draw(key: 2).Choice);
        Assert.False(Draw().Open);

        Draw(key: 65, modifiers: 1);
        Assert.True(Draw().Open);
        Assert.Equal(2, menuNode);
        Draw(key: 3);
        Assert.False(Draw().Open);

        Draw(point, enabled: false);
        Assert.False(Draw(point, right: true, enabled: false).Open);
        Assert.False(Draw(point, right: false, enabled: false).Open);

        CuiMenuResult Draw(CuiRectangle? pointer = null, bool? right = null, int key = 0, int modifiers = 0, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(pointer?.X ?? -1, pointer?.Y ?? -1);
            if (right.HasValue) window.SetRightButton(right.Value);
            if (key != 0) { window.SetKey(key, true, modifiers); window.SetKey(key, false, modifiers); }
            window.BeginColumn();
            int[] state = [-2, 0, -1];
            window.BeginTreeTable("tree", [new CuiGridColumn("Name", 300, 0, false, false, 0)], labels, parents,
                flags, state, 0, 0, 32, enabled ? 1 : 0);
            for (int index; (index = window.NextTreeTableRow(i => $"row/{i}")) >= 0;)
                window.TreeTableName(labels[index], "", null);
            window.EndTreeTable();
            bool open = window.BeginTreeTableMenu("tree", enabled, out menuNode);
            CuiMenuEntry[] entries = open ? [new(CuiMenuEntryKind.Action, "open", "Open")] : [];
            var result = window.EndTreeTableMenu(entries, open ? ["menu", "menu/open"] : ["menu"]);
            window.EndColumn();
            window.Present();
            return result;
        }
    }
}

// [Documentation](../../../docs/ui/gui/data-display/reorderable-list.md)
namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks managed input and ordering for native reorder lists.</summary>
public sealed class CuiReorderableListTests
{
    /// <summary>Rejects positions outside the permutation and keeps the list open.</summary>
    [GuiFact]
    public void ReorderListValidatesRowPositions()
    {
        using var window = new CuiWindow("Reorder positions", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        int[] order = window.BeginReorder("list", [0], enabled: true);
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BeginReorderItem(-1, true));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BeginReorderItem(order.Length, true));
        window.BeginReorderItem(0, enabled: true);
        window.Text("Row");
        window.EndReorderItem();
        Assert.False(window.EndReorder());
        window.Present();
    }

    /// <summary>Ignores a row press that starts while its list is disabled.</summary>
    [GuiFact]
    public void ReorderListCannotStartADragWhileDisabled()
    {
        using var window = new CuiWindow("Reorder", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw(true, -1, -1).Changed);
        var first = window.GetControlRectangle("row/0");
        var second = window.GetControlRectangle("row/1");
        float x = first.X + first.Width / 2;
        float top = first.Y + first.Height / 2;
        float bottom = second.Y + second.Height / 2 + 2;
        Assert.False(Draw(false, x, top, true).Changed);
        Assert.False(Draw(true, x, bottom).Changed);
        var rejected = Draw(true, x, bottom, false);
        Assert.False(rejected.Changed);
        Assert.Equal([0, 1], rejected.Order);

        Assert.False(Draw(true, x, top, true).Changed);
        Assert.False(Draw(true, x, bottom).Changed);
        var accepted = Draw(true, x, bottom, false);
        Assert.True(accepted.Changed);
        Assert.Equal([1, 0], accepted.Order);

        (int[] Order, bool Changed) Draw(bool enabled, float x, float y, bool? down = null)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(x, y);
            if (down is bool pressed) window.SetLeftButton(pressed);
            int[] order = window.BeginReorder("list", [0, 1], enabled);
            for (int position = 0; position < order.Length; position++)
            {
                window.BeginReorderItem(position, enabled: true);
                window.Text("Row " + order[position]);
                window.EndReorderItem();
                window.TrackLastControl("row/" + order[position]);
            }
            bool changed = window.EndReorder();
            window.Present();
            return (order, changed);
        }
    }
}

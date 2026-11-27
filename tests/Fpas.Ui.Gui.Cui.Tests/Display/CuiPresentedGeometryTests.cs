// [Documentation](../../../docs/ui/gui/cui.md#layout-and-control-inspection)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

public sealed class CuiPresentedGeometryTests
{
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void DiffRowsFollowPagingFoldingAndLayout(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Diff rows", 820, 440, renderer, continuous: true);
        string unchanged = string.Join('\n', Enumerable.Range(1, 40));
        string before = "old\n" + unchanged + "\nend old";
        string after = "new\n" + unchanged + "\nend new";
        Assert.Throws<InvalidOperationException>(() => window.ReadDiffRows("diff", 0, 1));
        Draw(0);
        var split = window.ReadDiffRows("diff", 0, 100);
        Assert.Equal(split.Total, split.Rows.Length);
        Assert.Equal(CuiDiffRowKind.Changed, split.Rows[0].Kind);
        Assert.Contains(split.Rows, row => row.Kind == CuiDiffRowKind.Fold && row.Hidden > 0);
        Assert.Equal(split.Rows[1], Assert.Single(window.ReadDiffRows("diff", 1, 1).Rows));
        Assert.Equal(split.Total, window.ReadDiffRows("diff", 0, 0).Total);
        Assert.Empty(window.ReadDiffRows("diff", int.MaxValue, int.MaxValue).Rows);
        Draw(2 | 4);
        var inline = window.ReadDiffRows("diff", 0, 100);
        Assert.NotEqual(split.Total, inline.Total);
        Assert.Contains(inline.Rows, row => row.Kind == CuiDiffRowKind.Added && row.OldLine == -1);
        Assert.Contains(inline.Rows, row => row.Kind == CuiDiffRowKind.Removed && row.NewLine == -1);
        Assert.DoesNotContain(inline.Rows, row => row.Kind == CuiDiffRowKind.Fold);
        Assert.Equal(CuiDiffRowKind.Changed, split.Rows[0].Kind);
        Assert.True(window.NextFrame()); window.Text("Discarded"); window.DiscardFrame();
        Assert.Throws<InvalidOperationException>(() => window.ReadDiffRows("diff", 0, 100));
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Assert.Equal(0, window.ReadDiffRows("diff", 0, 100).Total);

        void Draw(int flags)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn();
            window.DiffView("diff", before, after, -1, "", "", "", 2, 1, 0, 240, 0, 0, 1 | flags);
            window.EndColumn();
            window.Present();
        }
    }

    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void FlameBoundsFollowPresentedPanAndDepth(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Flame span geometry", 620, 340, renderer, continuous: true);
        var root = new CuiFlameSpan("root", 0, 10, 0, 0);
        var child = new CuiFlameSpan("child", 4, 6, 1, 0);
        Assert.Throws<InvalidOperationException>(() => window.FlameSpanRectangle("graph", child));
        Draw(0, 10);
        var full = window.FlameSpanRectangle("graph", child);
        Assert.True(full.Width > 0 && full.Height > 0);
        Assert.True(full.Y > window.FlameSpanRectangle("graph", root).Y);
        Assert.Equal(default, window.FlameSpanRectangle("graph", child with { Start = 20, End = 21 }));
        Draw(3, 7);
        Assert.True(window.FlameSpanRectangle("graph", child).Width > full.Width);
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Assert.Equal(default, window.FlameSpanRectangle("graph", child));

        void Draw(double start, double end)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn();
            window.FlameGraph("graph", [root, child], new(start, end, 0, -1), .01, 180, 22, 1, 0, 1);
            window.EndColumn();
            window.Present();
        }
    }
}

// [Documentation](../../../docs/ui/gui/data-display/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Collections;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks collection parts, selection output and frame scopes.</summary>
public sealed class CuiCollectionTrackingTests
{
    /// <summary>Rejects an extra virtual list while leaving enclosing lists and their rows usable.</summary>
    [GuiFact]
    public void VirtualListNestingLimitLeavesExistingScopesUsable()
    {
        using var window = new CuiWindow("Virtual list nesting", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        int count = 0;
        for (; count < 32; count++)
        {
            var failure = Record.Exception(() => window.BeginVirtualList($"list-{count}", 1, 50, 2, 50, true));
            if (failure is not null)
            {
                Assert.Contains("nesting limit", Assert.IsType<InvalidOperationException>(failure).Message);
                break;
            }
            window.BeginVirtualRow(0);
        }
        Assert.InRange(count, 1, 31);

        window.EndVirtualRow();
        window.EndVirtualList();
        Assert.Equal(new CuiVirtualRange(0, 1), window.BeginVirtualList("replacement", 1, 50, 2, 50, true));
        window.BeginVirtualRow(0);
        window.Skeleton("row", 2, 48, 50);
        window.TrackLastControl("replacement-row");
        window.EndVirtualRow();
        window.EndVirtualList();

        for (int index = 1; index < count; index++)
        {
            window.EndVirtualRow();
            window.EndVirtualList();
        }
        window.Present();
        Assert.Equal(48, window.GetControlRectangle("replacement-row").Width);
    }

    /// <summary>Reports realized grid headers without registering inspection keys.</summary>
    [GuiFact]
    public void DataGridReportsHeadersWithoutTracking()
    {
        using var window = new CuiWindow("Grid headers", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var frame = window.BeginDataGrid("grid", [new CuiGridColumn("Name", 0, 0, false, false, 0)],
            0, null, [-2, 0, -1, -1], 2, 160, 28, 1, headerId: null);
        window.EndDataGrid();
        window.Present();

        Assert.Equal([true], frame.Headers);
    }

    /// <summary>Returns one selection byte per row even when the selectable grid is empty.</summary>
    [GuiFact]
    public void EmptyDataGridReturnsAnEmptySelection()
    {
        using var window = new CuiWindow("Empty selection", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var frame = window.BeginDataGrid("grid", [new CuiGridColumn("Name", 0, 0, false, false, 0)],
            0, [], [-2, 0, -1, -1], 2, 160, 28, 1, headerId: null);
        window.EndDataGrid();
        window.Present();

        Assert.NotNull(frame.Selected);
        Assert.Empty(frame.Selected);
    }

    /// <summary>Allows the caller to close the grid after an inspection mapper throws.</summary>
    [GuiFact]
    public void DataGridCanCloseAfterHeaderTrackingFails()
    {
        using var window = new CuiWindow("Grid tracking", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var failure = new InvalidOperationException("Header unavailable.");
        Assert.Same(failure, Assert.Throws<InvalidOperationException>(() =>
            window.BeginDataGrid("grid", [new CuiGridColumn("Name", 0, 0, false, false, 0)],
                0, null, [-2, 0, -1, -1], 2, 160, 28, 1, headerId: _ => throw failure)));
        window.EndDataGrid();
        window.Present();
    }

    /// <summary>Reports visible search and add controls without registering inspection keys.</summary>
    [GuiFact]
    public void FilterBarReportsPartsWithoutTracking()
    {
        using var window = new CuiWindow("Filter parts", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var result = window.FilterBar("filters", [new CuiFilterField("Status", null, ["Open"])], [0],
            "", 64, enabled: true, focus: false, partId: null);
        window.Present();

        Assert.Equal([true, true, false, false], result.Parts);
    }

    /// <summary>Preserves a blank icon and focuses the chip when the search and add controls are absent.</summary>
    [GuiFact]
    public void FilterBarPreservesChipGlyphSpaceAndFallbackFocus()
    {
        using var window = new CuiWindow("Filter chip", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        var omitted = Draw(null, true);
        Assert.True(window.ControlFocused("part-3"));
        var blank = Draw("", false);
        Assert.True(blank.Width > omitted.Width);

        CuiRectangle Draw(string? icon, bool focus)
        {
            Assert.True(window.NextFrame());
            var result = window.FilterBar("filters", [new CuiFilterField("Status", icon, ["Open"])], [1u],
                null, 0, enabled: true, focus: focus, partId: index => $"part-{index}");
            window.Present();
            Assert.False(result.Changed);
            Assert.Equal([false, false, true, true], result.Parts);
            return window.GetControlRectangle("part-3");
        }
    }
}

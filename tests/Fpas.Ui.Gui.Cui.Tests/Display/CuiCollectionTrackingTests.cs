using Fpas.Ui.Gui.Cui.Widgets.Collections;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks collection parts, selection output and frame scopes.</summary>
public sealed class CuiCollectionTrackingTests
{
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
        var result = window.FilterBar("filters", [new CuiFilterField("Status", "", ["Open"])], [0],
            "", 64, enabled: true, focus: false, partId: null);
        window.Present();

        Assert.Equal([true, true, false, false], result.Parts);
    }
}

using Fpas.Ui.Gui.Cui.Widgets.Collections;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks that optional tracking does not change which widget parts are reported.</summary>
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

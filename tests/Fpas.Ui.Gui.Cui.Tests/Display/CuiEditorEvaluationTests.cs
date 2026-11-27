// [Documentation](../../../docs/ui/gui/editing-tools/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

public sealed class CuiEditorEvaluationTests
{
    [Theory]
    [InlineData((byte)0, 0.15625f)]
    [InlineData((byte)1, 0.015625f)]
    [InlineData((byte)2, 0.25f)]
    [InlineData((byte)3, 0f)]
    public void CurveRespectsNativeInterpolationModes(byte mode, float expected)
    {
        CuiCurveKey[] keys = [new(0, 0, 0, 0, 1f / 3, 0, mode), new(1, 1, -1f / 3, -1, 0, 0, mode)];
        Assert.Equal(expected, CuiCurve.Evaluate(keys, .25f), 4);
        Assert.Equal(0, CuiCurve.Evaluate([], .5f));
        Assert.Equal(0, CuiCurve.Evaluate(keys, float.NaN));
        Assert.Equal(0, CuiCurve.Evaluate(keys, -10));
        Assert.Equal(1, CuiCurve.Evaluate(keys, 10));
        Assert.Throws<ArgumentException>(() => CuiCurve.Evaluate([keys[1], keys[0]], .5f));
    }

    [Fact]
    public void GradientSamplesUnorderedStopsIncludingAlphaAndEndpoints()
    {
        CuiGradientStop[] stops = [new(1, 0xff00ff00), new(0, 0x000000ff)];
        Assert.Equal(0x80008080u, CuiGradient.Evaluate(stops, .5f));
        Assert.Equal(stops[0].Rgba, CuiGradient.Evaluate(stops, 2));
        Assert.Equal(stops[1].Rgba, CuiGradient.Evaluate(stops, -1));
        Assert.Equal(stops[0].Rgba, CuiGradient.Evaluate(stops, float.NaN));
        Assert.Equal(0u, CuiGradient.Evaluate([], .5f));
        Assert.Throws<ArgumentException>(() => CuiGradient.Evaluate([new(float.NaN, 0)], .5f));
    }

    [Fact]
    public void QueryUsesNestedGroupsAndNativeFieldOperators()
    {
        CuiDevField[] fields = [new("Title", 0, []), new("Score", 1, []),
            new("Status", 2, ["Open", "Closed"]), new("Active", 3, [])];
        CuiDevRule[] rules = [
            new(0, true, false, 0, 0, ""),
            new(1, false, false, 0, 2, "BUG"),
            new(1, true, true, 0, 0, ""),
            new(2, false, false, 1, 3, "10"),
            new(2, false, false, 2, 0, "Open"),
            new(1, false, false, 3, 0, "")
        ];
        Assert.True(CuiWindow.MatchQuery(fields, rules, ["Some bug", "10", "Closed", "yes"]));
        Assert.True(CuiWindow.MatchQuery(fields, rules, ["Some BUG", "7", "Open", "TRUE"]));
        Assert.False(CuiWindow.MatchQuery(fields, rules, ["Some bug", "7", "open", "true"]));
        Assert.False(CuiWindow.MatchQuery(fields, rules, ["Some bug", "10", "Open", "false"]));
        Assert.True(CuiWindow.MatchQuery(fields, [], ["Some bug", "0", "Closed", "false"]));
        Assert.True(CuiWindow.MatchQuery(fields, [new(0, true, true, 0, 0, "")], ["", "", "", ""]));
        Assert.Throws<ArgumentException>(() => CuiWindow.MatchQuery(fields, rules, ["Some bug"]));
        Assert.Throws<ArgumentException>(() => CuiWindow.MatchQuery(fields, [rules[1]], ["", "", "", ""]));
    }
}

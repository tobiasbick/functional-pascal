// [Documentation](../../../docs/ui/gui/developer-tools/query-builder.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;

namespace Fpas.Ui.Gui.Cui.Tests.Developer;

/// <summary>Checks complete native query descriptions without creating a window.</summary>
public sealed class CuiDescribeQueryTests
{
    /// <summary>Preserves long field names, including Unicode scalars crossing the old byte limit.</summary>
    /// <param name="character">Text repeated after an ASCII prefix at the former truncation boundary.</param>
    [Theory]
    [InlineData("a")]
    [InlineData("é")]
    [InlineData("😀")]
    public void LongFieldNamesPreserveTheCompleteCondition(string character)
    {
        string name = new string('a', 255) + string.Concat(Enumerable.Repeat(character, 100));
        CuiDevField[] fields = [new(name, 0, [])];
        CuiDevRule[] rules = [new(0, false, false, 0, 0, "value")];

        Assert.Equal(name + " is \"value\"", CuiWindow.DescribeQuery(fields, rules));
    }

    /// <summary>Preserves nested grouping and each condition when a long field name is repeated.</summary>
    [Fact]
    public void NestedGroupsPreserveAllConditions()
    {
        string name = string.Concat(Enumerable.Repeat("条件😀", 100));
        CuiDevField[] fields = [new(name, 0, [])];
        CuiDevRule[] rules = [
            new(0, true, false, 0, 0, ""),
            new(1, true, true, 0, 0, ""),
            new(2, false, false, 0, 0, "x"),
            new(2, false, false, 0, 0, "y"),
            new(1, false, false, 0, 0, "z")
        ];

        Assert.Equal($"({name} is \"x\" or {name} is \"y\") and {name} is \"z\"",
            CuiWindow.DescribeQuery(fields, rules));
    }

    /// <summary>Returns an empty description when the query has no rules.</summary>
    [Fact]
    public void EmptyQueryHasAnEmptyDescription()
    {
        Assert.Equal("", CuiWindow.DescribeQuery([], []));
    }

    /// <summary>Rejects a choice that cannot round-trip through a rule's native buffer.</summary>
    [Fact]
    public void ChoiceValuesMustFitInNativeRule()
    {
        string choice = new string('t', 61) + "é";
        Assert.Equal("Status is " + choice,
            CuiWindow.DescribeQuery([new("Status", 2, [choice])], [new(0, false, false, 0, 0, choice)]));
        Assert.Throws<ArgumentOutOfRangeException>(() =>
            CuiWindow.DescribeQuery([new("Status", 2, [choice + "x"])], []));
    }

    /// <summary>An empty first choice does not send an edit on an idle frame.</summary>
    /// <param name="renderer">Native renderer used for presentation.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void EmptyChoiceDoesNotProposeUnchangedRules(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Query builder", 640, 300, renderer, continuous: true);
        CuiDevField[] fields = [new("Status", 2, ["", "Open"])];
        CuiDevRule[] rules = [new(0, true, false, 0, 0, ""), new(1, false, false, 0, 0, "")];
        for (int i = 0; i < 2; ++i)
        {
            Assert.True(window.NextFrame());
            Assert.Same(rules, window.QueryBuilder("query", fields, rules, 8, 0, false, 1, 0, true));
            window.Present();
        }
    }
}

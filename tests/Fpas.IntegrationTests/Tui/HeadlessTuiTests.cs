// [Documentation](../../coverage.md#terminal-ui)
using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Checks standalone terminal host contracts.</summary>
public sealed class HeadlessTuiTests
{
    /// <summary>Exercises the explorer with keyboard, pointer and resize input through the terminal host.</summary>
    [Fact]
    public async Task Fractal_explorer_renders_and_navigates_all_families()
    {
        using var program = new Debuggee("fractal-explorer");
        await TuiFixture.CompileExample(program, "fractals", "fractal-explorer.fpas");
        Assert.Equal("Fractal explorer verified\n", await program.Run());
    }

    /// <summary>Builds and executes each terminal contract through the real FPAS compiler.</summary>
    /// <param name="fixture">Contract source copied into the test fixtures.</param>
    /// <param name="expected">Successful contract output.</param>
    [Theory]
    [InlineData("headless.fpas", "headless contracts verified\n")]
    [InlineData("composition.fpas", "Composition contracts verified\n")]
    [InlineData("effects.fpas", "effect contracts verified\n")]
    [InlineData("cui.fpas", "CUI terminal contracts verified\n")]
    [InlineData("rendering.fpas", "Rendering contracts verified\n")]
    [InlineData("checkbox.fpas", "Checkbox contracts verified\n")]
    [InlineData("radio-group.fpas", "Radio group contracts verified\n")]
    [InlineData("selection-list.fpas", "Selection list contracts verified\n")]
    [InlineData("terminal-widgets.fpas", "Terminal widget contracts verified\n")]
    [InlineData("filter-bar.fpas", "Filter bar contracts verified\n")]
    [InlineData("select.fpas", "Select contracts verified\n")]
    [InlineData("dropdown.fpas", "Dropdown contracts verified\n")]
    [InlineData("expandable-card.fpas", "Expandable card contracts verified\n")]
    [InlineData("tabs.fpas", "Tabs contracts verified\n")]
    [InlineData("cell-grid.fpas", "Cell grid contracts verified\n")]
    [InlineData("text-field.fpas", "Text field contracts verified\n")]
    [InlineData("text-area.fpas", "Text area contracts verified\n")]
    public async Task Headless_host_preserves_scheduling_focus_frame_and_effect_contracts(string fixture, string expected)
    {
        using var program = new Debuggee("headless-tui");
        await TuiFixture.Compile(program, fixture);
        Assert.Equal(expected, await program.Run());
    }
}

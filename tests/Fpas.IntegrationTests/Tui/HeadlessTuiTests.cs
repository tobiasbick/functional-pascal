using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Checks standalone terminal contracts and the filter-list application.</summary>
public sealed class HeadlessTuiTests
{
    /// <summary>Builds and executes each terminal contract through the real FPAS compiler.</summary>
    /// <param name="fixture">Contract source copied into the test fixtures.</param>
    /// <param name="expected">Successful contract output.</param>
    [Theory]
    [InlineData("headless.fpas", "headless contracts verified\n")]
    [InlineData("canvas-input.fpas", "canvas input contracts verified\n")]
    [InlineData("effects.fpas", "effect contracts verified\n")]
    [InlineData("tabs.fpas", "tab fallback contracts verified\n")]
    [InlineData("table.fpas", "table contracts verified\n")]
    [InlineData("multiline.fpas", "multiline editor contracts verified\n")]
    [InlineData("shared.fpas", "shared terminal contracts verified\n")]
    [InlineData("cui.fpas", "CUI terminal contracts verified\n")]
    public async Task Headless_host_preserves_scheduling_focus_frame_and_effect_contracts(string fixture, string expected)
    {
        using var program = new Debuggee("headless-tui");
        await TuiFixture.Compile(program, fixture);
        Assert.Equal(expected, await program.Run());
    }

    /// <summary>Exercises palette switching, selection retention and background updates.</summary>
    [Fact]
    public async Task Filter_list_example_preserves_selection_themes_and_background_updates()
    {
        using var program = new Debuggee("filter-list");
        await TuiFixture.Compile(program, "example.fpas");
        var result = await ProcessRunner.RunAsync([program.Assembly, "--headless"]);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Contains("Up to date", result.Output, StringComparison.Ordinal);
        Assert.Contains("100%", result.Output, StringComparison.Ordinal);
    }

}

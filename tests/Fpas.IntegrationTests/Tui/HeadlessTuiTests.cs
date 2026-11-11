using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Checks standalone terminal host contracts.</summary>
public sealed class HeadlessTuiTests
{
    /// <summary>Builds and executes each terminal contract through the real FPAS compiler.</summary>
    /// <param name="fixture">Contract source copied into the test fixtures.</param>
    /// <param name="expected">Successful contract output.</param>
    [Theory]
    [InlineData("headless.fpas", "headless contracts verified\n")]
    [InlineData("effects.fpas", "effect contracts verified\n")]
    [InlineData("cui.fpas", "CUI terminal contracts verified\n")]
    [InlineData("rendering.fpas", "Rendering contracts verified\n")]
    [InlineData("confirm.fpas", "Confirm button contracts verified\n")]
    [InlineData("status.fpas", "Status button contracts verified\n")]
    [InlineData("theme-switcher.fpas", "Theme switcher contracts verified\n")]
    [InlineData("theme-toggle.fpas", "Theme toggle contracts verified\n")]
    [InlineData("segmented.fpas", "Segmented control contracts verified\n")]
    [InlineData("dropdown.fpas", "Dropdown menu contracts verified\n")]
    public async Task Headless_host_preserves_scheduling_focus_frame_and_effect_contracts(string fixture, string expected)
    {
        using var program = new Debuggee("headless-tui");
        await TuiFixture.Compile(program, fixture);
        Assert.Equal(expected, await program.Run());
    }
}

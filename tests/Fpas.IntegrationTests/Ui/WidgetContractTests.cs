using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Ui;

public sealed class WidgetContractTests
{
    [Fact]
    public async Task Shared_widget_preflight_validates_portability_and_resolves_cached_subviews()
    {
        using var program = new Debuggee("shared-widgets");
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "ui");
        var sources = Directory.GetFiles(Path.Combine(fixture, "lib"), "*.fpas", SearchOption.AllDirectories);
        var library = program.FilePath("Fpas.Ui.dll");
        var compiled = await ProcessRunner.RunAsync(
            [TestEnvironment.Metadata("FpasCliPath"), "compile", .. sources, "--target", "library",
             "--reference-dir", TestEnvironment.ReferenceDirectory, "--output", library]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);

        await program.Compile(await File.ReadAllTextAsync(Path.Combine(fixture, "WidgetContract.fpas")),
            "--reference", library);
        Assert.Equal("shared widget contracts verified\n", await program.Run());
    }
}

using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

internal static class TuiFixture
{
    internal static async Task<string> Compile(Debuggee program, string fixture)
    {
        var fixtures = Path.Combine(AppContext.BaseDirectory, "Fixtures", "tui");
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var platform = Path.Combine(Path.GetDirectoryName(cli)!, "Fpas.Processes.dll");
        var library = program.FilePath("Fpas.Tui.dll");
        var sources = Directory.GetFiles(Path.Combine(fixtures, "lib"), "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", platform]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        File.Copy(platform, program.FilePath("Fpas.Processes.dll"));
        var source = await File.ReadAllTextAsync(Path.Combine(fixtures, fixture));
        await program.Compile(source, "--reference", library, "--reference", platform);
        return source;
    }
}

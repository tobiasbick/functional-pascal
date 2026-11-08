using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

internal static class TuiFixture
{
    internal static async Task<string> Compile(Debuggee program, string fixture)
    {
        var library = await BuildLibrary(program);
        var source = await File.ReadAllTextAsync(Path.Combine(Fixtures, fixture));
        await program.Compile(source, "--reference", library, "--reference", program.FilePath("Fpas.Processes.dll"));
        return source;
    }

    internal static string Fixtures => Path.Combine(AppContext.BaseDirectory, "Fixtures", "tui");

    internal static async Task CompileExample(Debuggee program, string example)
    {
        var library = await BuildLibrary(program);
        var folder = Path.Combine(Fixtures, example);
        var main = Path.Combine(folder, "main.fpas");
        await program.Compile(await File.ReadAllTextAsync(main),
            [.. Directory.GetFiles(folder, "*.fpas").Where(path => path != main),
             "--reference", library, "--reference", program.FilePath("Fpas.Processes.dll")]);
    }

    internal static async Task<string> BuildLibrary(Debuggee program)
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var platform = Path.Combine(Path.GetDirectoryName(cli)!, "Fpas.Processes.dll");
        var library = program.FilePath("Fpas.Tui.dll");
        var sources = Directory.GetFiles(Path.Combine(Fixtures, "lib"), "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", platform]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        File.Copy(platform, program.FilePath("Fpas.Processes.dll"));
        return library;
    }
}

using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Tui;

internal static class TuiFixture
{
    internal static async Task<string> Compile(Debuggee program, string fixture)
    {
        var library = await BuildLibrary(program);
        var source = await File.ReadAllTextAsync(Path.Combine(Fixtures, fixture));
        await program.Compile(source, References(program, library));
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
             .. References(program, library)]);
    }

    internal static async Task<string> BuildLibrary(Debuggee program)
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var platform = Path.Combine(Path.GetDirectoryName(cli)!, "Fpas.Processes.dll");
        var mvu = await BuildMvu(program);
        var ui = await BuildUi(program);
        var library = program.FilePath("Fpas.Tui.dll");
        var sources = Directory.GetFiles(Path.Combine(Fixtures, "lib"), "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", platform, "--reference", mvu,
            "--reference", ui], environment: new Dictionary<string, string?>
            { ["FpasCuiCMakeCommand"] = "unavailable-gui-toolchain" });
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        File.Copy(platform, program.FilePath("Fpas.Processes.dll"));
        return library;
    }

    private static async Task<string> BuildMvu(Debuggee program)
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var library = program.FilePath("Fpas.Mvu.dll");
        var folder = Path.Combine(AppContext.BaseDirectory, "Fixtures", "mvu", "lib");
        var sources = Directory.GetFiles(folder, "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        return library;
    }

    private static string[] References(Debuggee program, string library) =>
        ["--reference", library, "--reference", program.FilePath("Fpas.Mvu.dll"),
         "--reference", program.FilePath("Fpas.Processes.dll"), "--reference", program.FilePath("Fpas.Ui.dll")];

    private static async Task<string> BuildUi(Debuggee program)
    {
        var library = program.FilePath("Fpas.Ui.dll");
        var folder = Path.Combine(AppContext.BaseDirectory, "Fixtures", "ui", "lib");
        var sources = Directory.GetFiles(folder, "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", .. sources,
            "--output", library, "--target", "library", "--reference-dir", TestEnvironment.ReferenceDirectory]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        return library;
    }
}

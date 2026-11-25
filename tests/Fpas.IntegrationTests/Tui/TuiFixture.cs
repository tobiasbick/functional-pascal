// [Documentation](../../coverage.md#terminal-ui)
using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;
using System.Resources;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Tui;

internal static class TuiFixture
{
    internal static async Task<string> Compile(Debuggee program, string fixture)
    {
        var library = await BuildLibrary(program);
        var source = await File.ReadAllTextAsync(Path.Combine(Fixtures, fixture));
        await program.Compile(source, CompilationInputs(program, library));
        return source;
    }

    internal static string Fixtures => Path.Combine(AppContext.BaseDirectory, "Fixtures", "tui");

    internal static async Task CompileExample(Debuggee program, string example, string? fixture = null)
    {
        var library = await BuildLibrary(program);
        var folder = Path.Combine(Fixtures, example);
        var main = Path.Combine(folder, "main.fpas");
        var entry = fixture is null ? main : Path.Combine(Fixtures, fixture);
        var catalog = Path.Combine(folder, "Strings.resx");
        var resources = Array.Empty<string>();
        if (File.Exists(catalog))
        {
            var compiled = program.FilePath("Gallery.Shared.Strings.resources");
            using (var writer = new ResourceWriter(compiled))
                foreach (var item in XDocument.Load(catalog).Root!.Elements("data"))
                    writer.AddResource(item.Attribute("name")!.Value, item.Element("value")!.Value);
            resources = ["--embedded-resource", JsonSerializer.Serialize(new[] { "Gallery.Shared.Strings.resources", compiled })];
        }
        await program.Compile(await File.ReadAllTextAsync(entry),
            [.. Directory.GetFiles(folder, "*.fpas").Where(path => path != main),
             .. CompilationInputs(program, library), .. resources]);
    }

    internal static async Task<string> BuildLibrary(Debuggee program)
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var platform = Path.Combine(Path.GetDirectoryName(cli)!, "Fpas.Processes.dll");
        var mvu = await BuildMvu(program);
        var ui = await BuildUi(program);
        var library = program.FilePath("Fpas.Ui.Tui.dll");
        var sources = Directory.GetFiles(Path.Combine(Fixtures, "lib"), "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", platform, "--reference", mvu,
            "--reference", ui, "--reference", program.FilePath("Fpas.Execution.Effects.dll")], environment: new Dictionary<string, string?>
            { ["FpasCuiCMakeCommand"] = "unavailable-gui-toolchain" });
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        File.Copy(platform, program.FilePath("Fpas.Processes.dll"));
        return library;
    }

    private static async Task<string> BuildMvu(Debuggee program)
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var effects = program.FilePath("Fpas.Execution.Effects.dll");
        await program.CompileFile(Path.Combine(AppContext.BaseDirectory, "Fixtures", "effects", "Effect.fpas"), effects, "--target", "library");
        var library = program.FilePath("Fpas.Ui.Mvu.dll");
        var folder = Path.Combine(AppContext.BaseDirectory, "Fixtures", "mvu", "lib");
        var sources = Directory.GetFiles(folder, "*.fpas", SearchOption.AllDirectories);
        var compiled = await ProcessRunner.RunAsync([cli, "compile", .. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", effects]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        return library;
    }

    private static string[] CompilationInputs(Debuggee program, string library) =>
        [Path.Combine(Fixtures, "EffectReplies.fpas"), "--reference", library, "--reference", program.FilePath("Fpas.Ui.Mvu.dll"),
         "--reference", program.FilePath("Fpas.Processes.dll"), "--reference", program.FilePath("Fpas.Ui.dll"),
         "--reference", program.FilePath("Fpas.Execution.Effects.dll")];

    private static async Task<string> BuildUi(Debuggee program)
    {
        var library = program.FilePath("Fpas.Ui.dll");
        var folder = Path.Combine(AppContext.BaseDirectory, "Fixtures", "ui", "lib");
        var sources = Directory.GetFiles(folder, "*.fpas", SearchOption.AllDirectories);
        string[] arguments = [.. sources, "--output", library, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory];
        // The shared UI has enough sources to exceed the Windows process command-line limit.
        var response = program.FilePath("ui-compile.json");
        File.WriteAllText(response, JsonSerializer.Serialize(arguments));
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile",
            "--response-file", response], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        return library;
    }
}

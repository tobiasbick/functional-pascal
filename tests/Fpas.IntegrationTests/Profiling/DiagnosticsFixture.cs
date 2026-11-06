using System.Xml.Linq;
using Fpas.IntegrationTests.Build;
using Fpas.Processes;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Profiling;

public sealed class DiagnosticsFixture : IAsyncLifetime
{
    private readonly SdkPackageFixture sdk = new();
    private readonly TemporaryDirectory directory = new();
    internal string Configuration => TestEnvironment.Metadata("FpasBuildConfiguration");
    internal string Repository => Path.GetFullPath(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasSdkProject"))!, "..", ".."));
    internal string Workload => directory.File($"Workload/bin/{Configuration}/net10.0/Workload.dll");
    internal string Measure => directory.File($"Measure/bin/{Configuration}/net10.0/Measure.dll");
    internal string Reports { get; private set; } = "";

    public async Task InitializeAsync()
    {
        await sdk.InitializeAsync();
        Reports = Environment.GetEnvironmentVariable("FPAS_DIAGNOSTICS_REPORTS") ?? directory.File("reports");
        Directory.CreateDirectory(Reports);
        var restored = await ProcessRunner.RunAsync(["tool", "restore"], workingDirectory: Repository, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        File.WriteAllText(Path.Combine(Reports, "tool-restore.log"), restored.Output + restored.Error);
        var source = Path.Combine(Repository, "apps", "diagnostics");
        foreach (var file in Directory.GetFiles(source, "*", SearchOption.AllDirectories).Where(file =>
            !Path.GetRelativePath(source, file).Split(Path.DirectorySeparatorChar).Any(part => part is "bin" or "obj")))
        {
            var target = directory.File(Path.GetRelativePath(source, file));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.WriteAllText(target, File.ReadAllText(file).Replace("Fpas.NET.Sdk/0.1.0-dev", "Fpas.NET.Sdk/" + sdk.Version, StringComparison.Ordinal));
        }
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas-test"), new XAttribute("value", sdk.Feed)))))
            .Save(directory.File("NuGet.Config"));
        var project = directory.File("Measure/Measure.fpasproj"); var xml = XDocument.Load(project);
        xml.Descendants("ProjectReference").Remove();
        xml.Root!.Add(new XElement("ItemGroup", new XElement("Reference", new XAttribute("Include", "Fpas.Processes"),
            new XElement("HintPath", typeof(OwnedProcess).Assembly.Location)))); xml.Save(project);
        foreach (var name in new[] { "Workload", "Measure" })
        {
            var built = await ProcessRunner.RunAsync(["build", directory.File($"{name}/{name}.fpasproj"), "-c", Configuration, "-warnaserror"],
                workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(built.ExitCode == 0, built.Output + built.Error);
            File.WriteAllText(Path.Combine(Reports, "build-" + name + ".log"), built.Output + built.Error);
        }
        var symbols = Path.Combine(Reports, "subject"); Directory.CreateDirectory(symbols);
        foreach (var file in Directory.GetFiles(Path.GetDirectoryName(Workload)!)) File.Copy(file, Path.Combine(symbols, Path.GetFileName(file)), overwrite: true);
        foreach (var file in Directory.GetFiles(directory.File("Workload"), "*.fpas")) File.Copy(file, Path.Combine(symbols, Path.GetFileName(file)), overwrite: true);
    }

    internal Task<ProcessResult> Run(params string[] arguments) => ProcessRunner.RunAsync(arguments, workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));

    internal async Task<ProcessResult> Tool(string name, string log, params string[] arguments)
    {
        var result = await ProcessRunner.RunAsync(["tool", "run", name, "--", .. arguments], workingDirectory: Repository, timeLimit: TimeSpan.FromMinutes(2));
        File.WriteAllText(Path.Combine(Reports, log), result.Output + result.Error);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        return result;
    }

    public async Task DisposeAsync() { directory.Dispose(); await sdk.DisposeAsync(); }
}

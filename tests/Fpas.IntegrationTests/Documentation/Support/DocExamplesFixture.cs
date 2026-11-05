using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Documentation.Support;

[CollectionDefinition("Documentation examples")]
public sealed class DocExamplesCollection : ICollectionFixture<DocExamplesFixture>;

public sealed class DocExamplesFixture : IAsyncLifetime
{
    private readonly TemporaryDirectory directory = new();
    private readonly SdkPackageFixture sdk = new();
    private readonly object gate = new();
    private Task<JsonElement>? repositoryReport;
    internal string Application => directory.File("Application/bin/Debug/net10.0/DocExamples.dll");
    internal string Repository => directory.File("Repository");
    internal string SdkVersion => sdk.Version;
    internal string Feed => sdk.Feed;

    public async Task InitializeAsync()
    {
        await sdk.InitializeAsync();
        Copy(Path.Combine(AppContext.BaseDirectory, "Fixtures/doc-examples"), directory.File("Application"));
        var project = directory.File("Application/DocExamples.fpasproj");
        var xml = XDocument.Load(project);
        xml.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + sdk.Version);
        xml.Descendants("ProjectReference").Single().ReplaceWith(new XElement("Reference", new XAttribute("Include", "Fpas.Processes"),
            new XElement("HintPath", typeof(Fpas.Processes.OwnedProcess).Assembly.Location)));
        xml.Save(project);
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas-test"), new XAttribute("value", sdk.Feed)))))
            .Save(directory.File("NuGet.Config"));
        var restore = await ProcessRunner.RunAsync(["restore", project], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        var build = await ProcessRunner.RunAsync(["build", project, "--no-restore", "--configuration", "Debug", "--disable-build-servers", "-warnaserror"],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        Assert.True(File.Exists(Application));
        Copy(Path.Combine(AppContext.BaseDirectory, "Fixtures/specification"), Path.Combine(Repository, "docs/pascal"));
        Directory.CreateDirectory(Repository);
        File.Copy(Path.Combine(AppContext.BaseDirectory, "Fixtures/repository/README.md"), Path.Combine(Repository, "README.md"));
        Copy(directory.File("Application/contexts"), Path.Combine(Repository, "apps/doc-examples/contexts"));
    }

    internal string[] Arguments(string root, params string[] options)
        => [Application, "--root", root, "--compiler", TestEnvironment.Metadata("FpasCliPath"),
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--dotnet", TestEnvironment.DotnetHost,
            "--reference", typeof(FactAttribute).Assembly.Location, "--reference", typeof(Assert).Assembly.Location, .. options];

    internal async Task<ProcessResult> Run(string root, params string[] options)
        => await ProcessRunner.RunAsync(Arguments(root, options), workingDirectory: root, timeLimit: TimeSpan.FromMinutes(3));

    internal Task<JsonElement> RepositoryReport()
    {
        lock (gate) return repositoryReport ??= CheckRepository();
    }

    private async Task<JsonElement> CheckRepository()
    {
        var result = await Run(Repository, "--json", "--timeout", "30");
        Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal(0, json.RootElement.GetProperty("failed").GetInt32());
        Assert.NotEmpty(json.RootElement.GetProperty("compilerHash").GetString()!);
        Assert.Equal(json.RootElement.GetProperty("compilerHash").GetString(), json.RootElement.GetProperty("sdkCompilerHash").GetString());
        return json.RootElement.Clone();
    }

    internal async Task<string> Assembly(MarkdownExample example)
    {
        var report = await RepositoryReport();
        var path = Path.GetFullPath(example.Document == "repository/README.md" ? Path.Combine(Repository, "README.md") : Path.Combine(Repository, "docs/pascal", example.Document));
        var matches = report.GetProperty("results").EnumerateArray().Where(item => item.GetProperty("document").GetString() == path &&
            Path.GetFileName(item.GetProperty("assembly").GetString()) == example.Name + ".dll").ToArray();
        Assert.True(matches.Length == 1, "Expected one compiled example: " + example.Document + " / " + example.Name);
        var item = matches[0];
        Assert.True(item.GetProperty("passed").GetBoolean(), item.ToString());
        return item.GetProperty("assembly").GetString()!;
    }

    private static void Copy(string source, string destination)
    {
        foreach (var file in Directory.GetFiles(source, "*", SearchOption.AllDirectories))
        {
            var target = Path.Combine(destination, Path.GetRelativePath(source, file));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!); File.Copy(file, target);
        }
    }

    public async Task DisposeAsync()
    {
        directory.Dispose(); await sdk.DisposeAsync();
    }
}

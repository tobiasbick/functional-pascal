using System.IO.Compression;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed class SdkPackageFixture : IAsyncLifetime
{
    private readonly TemporaryDirectory directory = new();
    internal string Version { get; } = "0.1.0-test.g" + Guid.NewGuid().ToString("N");
    internal string Feed => directory.File("feed");

    public async Task InitializeAsync()
    {
        Directory.CreateDirectory(Feed);
        var result = await ProcessRunner.RunAsync(["pack", Path.GetFullPath(TestEnvironment.Metadata("FpasSdkProject")),
            "--no-build", "--configuration", TestEnvironment.Metadata("FpasBuildConfiguration"), "--output", Feed,
            "-p:PackageVersion=" + Version], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.True(File.Exists(Path.Combine(Feed, $"Fpas.NET.Sdk.{Version}.nupkg")));
        using var archive = ZipFile.OpenRead(Path.Combine(Feed, $"Fpas.NET.Sdk.{Version}.nupkg"));
        foreach (var name in new[] { "Fpas.Cli.dll", "Fpas.Workspaces.dll", "Fpas.SdkResolver.dll", "Fpas.Processes.dll", "Microsoft.VisualStudio.SolutionPersistence.dll",
            "Fpas.LanguageServer.dll", "OmniSharp.Extensions.LanguageProtocol.dll", "StreamJsonRpc.dll", "Newtonsoft.Json.dll" })
            Assert.Single(archive.Entries, entry => Path.GetFileName(entry.FullName) == name);
        Assert.DoesNotContain(archive.Entries, entry => Path.GetFileName(entry.FullName) is "Microsoft.Build.dll" or "Microsoft.Build.Framework.dll" or "Microsoft.Build.Utilities.Core.dll");
        foreach (var path in new[] { "tools/tasks/Fpas.Deployment.dll", "tools/tasks/Fpas.Build.Tasks.deps.json",
            "tools/tasks/System.Security.Cryptography.Pkcs.dll", "tools/tasks/runtimes/win/lib/net10.0/System.Security.Cryptography.Pkcs.dll" })
            Assert.Single(archive.Entries, entry => entry.FullName == path);
    }

    internal TemporaryDirectory CreateProject()
    {
        var project = new TemporaryDirectory();
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "msbuild");
        foreach (var source in Directory.GetFiles(fixture, "*", SearchOption.AllDirectories))
        {
            var target = Path.Combine(project.Path, Path.GetRelativePath(fixture, source));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.WriteAllText(target, File.ReadAllText(source).Replace("SDK_VERSION", Version, StringComparison.Ordinal));
        }
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas-test"), new XAttribute("value", Feed)))))
            .Save(project.File("NuGet.Config"));
        return project;
    }

    internal Task<ProcessResult> Run(TemporaryDirectory project, params string[] args)
        => ProcessRunner.RunAsync(args, workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(2));

    public Task DisposeAsync()
    {
        directory.Dispose();
        return Task.CompletedTask;
    }
}

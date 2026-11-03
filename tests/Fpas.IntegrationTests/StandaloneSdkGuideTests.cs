using System.Text.RegularExpressions;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class StandaloneSdkGuideTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Documented_consumer_uses_only_packaged_tools_and_deploys_without_build_inputs(string configuration)
    {
        using var directory = new TemporaryDirectory();
        var consumer = directory.File("consumer");
        var guide = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", "standalone-sdk.md"));
        foreach (Match block in Regex.Matches(guide, @"(?m)^```(?:xml|csharp|pascal) file=(\S+)\r?\n([\s\S]*?)^```\r?$"))
        {
            var path = Path.GetFullPath(Path.Combine(consumer, block.Groups[1].Value));
            Assert.StartsWith(consumer + Path.DirectorySeparatorChar, path);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, block.Groups[2].Value);
        }
        var sdk = XDocument.Load(Path.Combine(consumer, "src", "App", "App.fpasproj")).Root!.Attribute("Sdk")!.Value;
        var version = sdk.Split('/')[1];
        var feed = Path.Combine(consumer, "feed");
        var packages = Path.Combine(consumer, ".packages");
        Directory.CreateDirectory(feed);
        Assert.False(Directory.Exists(packages));
        var environment = new Dictionary<string, string?>
        {
            ["NUGET_PACKAGES"] = packages,
            ["NUGET_HTTP_CACHE_PATH"] = directory.File("http-cache")
        };

        // A fresh consumer must not resolve this identity from another package cache.
        var missingPackage = await ProcessRunner.RunAsync(["restore", "Consumer.slnx", "--configfile", "NuGet.Config", "-warnaserror"],
            workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(missingPackage.ExitCode != 0, missingPackage.Output + missingPackage.Error);
        Assert.Contains("NU1503", missingPackage.Output + missingPackage.Error);

        await Run(["pack", Path.GetFullPath(TestEnvironment.Metadata("FpasSdkProject")), "--no-build",
            "--configuration", TestEnvironment.Metadata("FpasBuildConfiguration"), "--output", feed,
            "-p:PackageVersion=" + version]);
        Assert.True(File.Exists(Path.Combine(feed, $"Fpas.NET.Sdk.{version}.nupkg")));
        await Run(["restore", "Consumer.slnx", "--configfile", "NuGet.Config", "-warnaserror"], consumer, environment);
        Assert.True(File.Exists(Path.Combine(packages, "fpas.net.sdk", version, "tools", "compiler", "Fpas.Cli.dll")));
        await Run(["build", "Consumer.slnx", "--no-restore", "--configuration", configuration, "-warnaserror"], consumer, environment);

        foreach (var name in new[] { "App", "Host" })
        {
            var output = Path.Combine(consumer, "src", name, "bin", configuration, "net10.0");
            foreach (var file in new[] { name + ".dll", name + ".deps.json", name + ".runtimeconfig.json",
                         "Messages.dll", "Messages.pdb", "Messages.xml", "Shared.dll", "Fpas.Core.dll" })
                Assert.True(File.Exists(Path.Combine(output, file)), "Missing deployment file: " + file);
            var result = await Run([Path.Combine(output, name + ".dll")], consumer, environment);
            Assert.Equal(Expected(name), result.Output.Trim());
            var deployment = directory.File("deploy-" + name);
            foreach (var source in Directory.GetFiles(output, "*", SearchOption.AllDirectories))
            {
                var target = Path.Combine(deployment, Path.GetRelativePath(output, source));
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                File.Copy(source, target);
            }
        }

        // Both paths are owned children of this test directory; remove the original build paths from use.
        var retired = directory.File("retired-consumer");
        Assert.Equal(directory.Path, Path.GetDirectoryName(consumer));
        Assert.Equal(directory.Path, Path.GetDirectoryName(retired));
        Directory.Move(consumer, retired);
        Assert.False(Directory.Exists(consumer));
        Assert.False(Directory.Exists(packages));
        foreach (var name in new[] { "App", "Host" })
        {
            var result = await Run([name + ".dll"], directory.File("deploy-" + name), environment);
            Assert.Equal(Expected(name), result.Output.Trim());
        }
    }

    private static string Expected(string name) => name == "App" ? "Hello, FPAS" : "Hello, C#";

    private static async Task<ProcessResult> Run(string[] arguments, string? workingDirectory = null,
        IReadOnlyDictionary<string, string?>? environment = null)
    {
        var result = await ProcessRunner.RunAsync(arguments, workingDirectory: workingDirectory,
            environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        return result;
    }
}

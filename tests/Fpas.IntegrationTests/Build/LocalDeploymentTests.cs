using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    [Fact]
    public async Task Dist_delivers_one_offline_executable_with_matching_SDK_and_refreshes_only_its_cached_version()
    {
        using var directory = new TemporaryDirectory();
        var repository = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasSdkProject"))!, "..", ".."));
        var checkout = directory.File("checkout");
        Directory.CreateDirectory(checkout);
        foreach (var name in new[] { "src", "sdk" }) Copy(Path.Combine(repository, name), Path.Combine(checkout, name));
        foreach (var name in new[] { "Directory.Build.props", "Directory.Packages.props", "global.json", "NuGet.Config", "dist.ps1", "dist.sh" })
            File.Copy(Path.Combine(repository, name), Path.Combine(checkout, name));
        var sdkProject = Path.Combine(checkout, "sdk", "Fpas.NET.Sdk", "Fpas.NET.Sdk.csproj");
        var version = "0.1.0-dist.g" + Guid.NewGuid().ToString("N");
        var xml = XDocument.Load(sdkProject); xml.Root!.Element("PropertyGroup")!.Element("Version")!.Value = version; xml.Save(sdkProject);
        var packages = directory.File("packages");
        var sdkCache = Path.Combine(packages, "fpas.net.sdk", version);
        var otherVersion = Path.Combine(packages, "fpas.net.sdk", "0.0.0-preserve");
        Directory.CreateDirectory(otherVersion); File.WriteAllText(Path.Combine(otherVersion, "keep"), "other SDK version");
        var unrelated = Path.Combine(packages, "unrelated", "1.0.0");
        Directory.CreateDirectory(unrelated); File.WriteAllText(Path.Combine(unrelated, "keep"), "other package");
        var assets = directory.File("assets");
        var environment = new Dictionary<string, string?> { ["NUGET_PACKAGES"] = packages,
            ["FPAS_ASSET_CACHE"] = assets, ["DOTNET_BUNDLE_EXTRACT_BASE_DIR"] = directory.File("bundle") };
        var consumer = directory.File("consumer"); Directory.CreateDirectory(consumer);
        File.WriteAllText(Path.Combine(consumer, "Probe.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{version}\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(Path.Combine(consumer, "Probe.fpas"), "unit Probe; public function Read(): int; begin return 42; end function; end unit;");
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "dist"), new XAttribute("value", directory.File("exported-feed"))))))
            .Save(Path.Combine(consumer, "NuGet.Config"));
        var delivery = directory.File("delivery"); Directory.CreateDirectory(delivery);
        var launcher = Path.Combine(delivery, OperatingSystem.IsWindows() ? "fpas.exe" : "fpas");

        for (var run = 0; run < 2; run++)
        {
            Directory.CreateDirectory(sdkCache); File.WriteAllText(Path.Combine(sdkCache, "stale"), "cached previous tools");
            var script = Path.Combine(checkout, OperatingSystem.IsWindows() ? "dist.ps1" : "dist.sh");
            var shell = OperatingSystem.IsWindows()
                ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "WindowsPowerShell", "v1.0", "powershell.exe") : "/bin/sh";
            string[] arguments = OperatingSystem.IsWindows() ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script] : [script];
            var deployed = await ProcessRunner.RunExecutableAsync(shell, arguments, workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(5));
            Assert.True(deployed.ExitCode == 0, deployed.Output + deployed.Error);
            string[] expected = run == 0 ? [Path.GetFileName(launcher)] : ["keep", Path.GetFileName(launcher)];
            Assert.Equal(expected.Order(),
                Directory.GetFiles(Path.Combine(checkout, "bin")).Select(Path.GetFileName).Order());
            File.Copy(Path.Combine(checkout, "bin", Path.GetFileName(launcher)), launcher, overwrite: true);
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(launcher, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
            Assert.Single(Directory.GetFileSystemEntries(delivery));
            Assert.True(File.Exists(Path.Combine(checkout, "sdk", "feed", $"Fpas.NET.Sdk.{version}.nupkg")));
            Assert.False(Directory.Exists(sdkCache));
            Assert.Equal("other SDK version", File.ReadAllText(Path.Combine(otherVersion, "keep")));
            Assert.Equal("other package", File.ReadAllText(Path.Combine(unrelated, "keep")));
            if (run == 0) File.WriteAllText(Path.Combine(checkout, "bin", "keep"), "unrelated deployment file");
            Assert.Equal("unrelated deployment file", File.ReadAllText(Path.Combine(checkout, "bin", "keep")));

            await CheckSingleFile(checkout, consumer, launcher, directory.File("exported-feed"), version, environment, firstRun: run == 0);

            var found = await ProcessRunner.RunExecutableAsync(launcher, ["query", "find", "Probe.fpasproj", "--name", "Read", "--match", "exact"],
                workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(found.ExitCode == 0, found.Output + found.Error);
            using var find = JsonDocument.Parse(found.Output);
            Assert.Equal("ok", find.RootElement.GetProperty("status").GetString());
            var match = Assert.Single(find.RootElement.GetProperty("result").GetProperty("find").GetProperty("matches").EnumerateArray());
            var span = match.GetProperty("span");
            var inspected = await ProcessRunner.RunExecutableAsync(launcher, ["query", "inspect", "Probe.fpasproj", "--file", "Probe.fpas", "--line", span.GetProperty("line").GetInt32().ToString(), "--column", span.GetProperty("column").GetInt32().ToString(), "--no-restore"],
                workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(inspected.ExitCode == 0, inspected.Output + inspected.Error);
            using var inspect = JsonDocument.Parse(inspected.Output);
            Assert.Equal("public function Read(): int; begin return 42; end function;",
                inspect.RootElement.GetProperty("result").GetProperty("inspect").GetProperty("source").GetProperty("declaration").GetProperty("text").GetString());
            Assert.Equal(File.ReadAllBytes(Assert.Single(Directory.GetFiles(assets, "Fpas.Compiler.dll", SearchOption.AllDirectories))),
                File.ReadAllBytes(Path.Combine(sdkCache, "tools", "compiler", "Fpas.Compiler.dll")));
            await CheckLanguageServer(consumer, launcher, sdkCache, environment);
            if (run == 0)
            {
                await CheckPublishing(consumer, launcher, directory.File("exported-feed"), version, environment);
                await CheckSdkCompiler(consumer, launcher, sdkCache, environment);
                await CheckDebugger(checkout, consumer, launcher, version, packages, environment);
            }
            Assert.Single(Directory.GetFileSystemEntries(delivery));
        }

        static void Copy(string source, string target)
        {
            Directory.CreateDirectory(target);
            foreach (var file in Directory.GetFiles(source)) File.Copy(file, Path.Combine(target, Path.GetFileName(file)));
            foreach (var child in Directory.GetDirectories(source).Where(path => Path.GetFileName(path) is not ("bin" or "obj" or "feed")))
                Copy(child, Path.Combine(target, Path.GetFileName(child)));
        }
    }
}

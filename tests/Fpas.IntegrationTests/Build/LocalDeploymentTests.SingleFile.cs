using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    private static async Task CheckSingleFile(string checkout, string consumer, string launcher, string feed, string version,
        IReadOnlyDictionary<string, string?> environment, bool firstRun)
    {
        var isolated = environment.ToDictionary(pair => pair.Key, pair => pair.Value);
        foreach (var name in new[] { "DOTNET_ROOT", "DOTNET_ROOT_X64", "DOTNET_HOST_PATH" }) isolated[name] = Path.Combine(consumer, "absent-dotnet");
        isolated["PATH"] = ""; isolated["DOTNET_MULTILEVEL_LOOKUP"] = "0";
        var output = Path.Combine(Path.GetDirectoryName(consumer)!, "direct"); Directory.CreateDirectory(output);
        var source = Path.Combine(consumer, "Probe.fpas");
        async Task<ProcessResult> Compile(string name) => await ProcessRunner.RunExecutableAsync(launcher,
            ["compile", source, "--target", "library", "--output", Path.Combine(output, name + ".dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"], workingDirectory: consumer, environment: isolated);
        Directory.Move(checkout, checkout + ".offline");
        try
        {
            var compiled = await Task.WhenAll(Compile("first"), Compile("second"));
            foreach (var result in compiled)
            { Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Empty(result.Error); }
            var invalid = Path.Combine(output, "invalid.fpas"); File.WriteAllText(invalid, "unit Broken; function ; end unit;");
            var rejected = await ProcessRunner.RunExecutableAsync(launcher, ["compile", invalid, "--target", "library", "--output", Path.Combine(output, "invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"], workingDirectory: consumer, environment: isolated);
            Assert.Equal(1, rejected.ExitCode);
            using var diagnostics = JsonDocument.Parse(rejected.Output);
            Assert.False(diagnostics.RootElement.GetProperty("success").GetBoolean());
            Assert.NotEmpty(diagnostics.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.False(File.Exists(Path.Combine(output, "invalid.dll")));

            if (!firstRun)
            {
                var previous = Path.Combine(feed, "Fpas.NET.Sdk." + version + ".nupkg");
                File.WriteAllText(previous, "conflicting package");
                var conflict = await ProcessRunner.RunExecutableAsync(launcher, ["sdk", "export", feed], workingDirectory: consumer, environment: isolated);
                Assert.Equal(1, conflict.ExitCode); Assert.Contains("different SDK package", conflict.Error);
                Assert.Equal("conflicting package", File.ReadAllText(previous));
                var replaced = await ProcessRunner.RunExecutableAsync(launcher, ["sdk", "export", feed, "--force"], workingDirectory: consumer, environment: isolated);
                Assert.True(replaced.ExitCode == 0, replaced.Output + replaced.Error);
            }
            var exports = await Task.WhenAll(Enumerable.Range(0, 2).Select(_ => ProcessRunner.RunExecutableAsync(launcher,
                ["sdk", "export", feed], workingDirectory: consumer, environment: isolated)));
            foreach (var result in exports) Assert.True(result.ExitCode == 0, result.Output + result.Error);
            var licenses = await ProcessRunner.RunExecutableAsync(launcher, ["licenses"], workingDirectory: consumer, environment: isolated);
            Assert.Equal(0, licenses.ExitCode); Assert.Contains("MIT License", licenses.Output);
            var core = Assert.Single(Directory.GetFiles(environment["FPAS_ASSET_CACHE"]!, "Fpas.Core.dll", SearchOption.AllDirectories));
            var bytes = File.ReadAllBytes(core); File.WriteAllText(core, "corrupted cache");
            Assert.Equal(0, (await Compile("repaired")).ExitCode); Assert.Equal(bytes, File.ReadAllBytes(core));
        }
        finally { Directory.Move(checkout + ".offline", checkout); }
        Assert.Equal(File.ReadAllBytes(Path.Combine(checkout, "sdk", "feed", "Fpas.NET.Sdk." + version + ".nupkg")),
            File.ReadAllBytes(Path.Combine(feed, "Fpas.NET.Sdk." + version + ".nupkg")));
    }

    private static async Task CheckSdkCompiler(string consumer, string launcher, string sdkCache, IReadOnlyDictionary<string, string?> environment)
    {
        var compiler = Path.Combine(sdkCache, "tools", "compiler", "Fpas.Cli.dll");
        var metadata = Path.Combine(sdkCache, "tools", "compiler", "Fpas.Compiler.dll");
        var original = File.ReadAllBytes(metadata);
        File.Move(compiler, compiler + ".offline");
        try
        {
            async Task<ProcessResult> Build() => await ProcessRunner.RunAsync(["build", "Probe.fpasproj", "--no-restore", "-t:Rebuild", "-p:FpasCompilerPath=" + launcher],
                workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            var built = await Build(); Assert.True(built.ExitCode == 0, built.Output + built.Error);
            File.AppendAllText(metadata, "different compiler");
            var rejected = await Build(); Assert.NotEqual(0, rejected.ExitCode);
            Assert.Contains("Selected SDK compiler differs", rejected.Output + rejected.Error);
            Assert.False(File.Exists(Path.Combine(consumer, "bin", "Debug", "net10.0", "Probe.dll")));
        }
        finally { File.WriteAllBytes(metadata, original); File.Move(compiler + ".offline", compiler); }
    }
}

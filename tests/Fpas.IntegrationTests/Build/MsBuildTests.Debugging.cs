using Fpas.Tests.Support;
using System.IO.Compression;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task SDK_contains_the_own_debugger_native_assets_notices_and_working_help()
    {
        using var extracted = new TemporaryDirectory();
        ZipFile.ExtractToDirectory(Path.Combine(sdk.Feed, $"Fpas.NET.Sdk.{sdk.Version}.nupkg"), extracted.Path);
        var tools = extracted.File("tools/compiler");
        foreach (var name in new[] { "Fpas.Debugging.dll", "debugger-notices.txt" })
            Assert.True(File.Exists(Path.Combine(tools, name)), name);
        Assert.Equal(new[] { "runtimes/linux-x64/native/libdbgshim.so", "runtimes/win-x64/native/dbgshim.dll" },
            Directory.GetFiles(tools, "*dbgshim*", SearchOption.AllDirectories)
                .Select(path => Path.GetRelativePath(tools, path).Replace('\\', '/')).Order(StringComparer.Ordinal));
        var result = await ProcessRunner.RunAsync([Path.Combine(tools, "Fpas.Cli.dll"), "debug", "--help"]);
        Assert.True(result.ExitCode == 0, result.Error);
        Assert.Contains("--protocol", result.Output); Assert.Contains("--print", result.Output);
    }
}

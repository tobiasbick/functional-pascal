using Fpas.Tests.Support;
using System.IO.Compression;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task SDK_contains_the_agent_debugger_and_all_managed_tool_dependencies()
    {
        using var extracted = new TemporaryDirectory();
        ZipFile.ExtractToDirectory(Path.Combine(sdk.Feed, $"Fpas.NET.Sdk.{sdk.Version}.nupkg"), extracted.Path);
        var tools = extracted.File("tools/compiler");
        foreach (var name in new[] { "Fpas.Debugging.dll", "ModelContextProtocol.dll", "ModelContextProtocol.Core.dll", "Microsoft.Extensions.Hosting.dll" })
            Assert.True(File.Exists(Path.Combine(tools, name)), name);
        Assert.DoesNotContain(Directory.GetFiles(tools, "*", SearchOption.AllDirectories), file => Path.GetFileNameWithoutExtension(file).Equals("netcoredbg", StringComparison.OrdinalIgnoreCase));
        var result = await ProcessRunner.RunAsync([Path.Combine(tools, "Fpas.Cli.dll"), "debug", "--help"]);
        Assert.True(result.ExitCode == 0, result.Error);
        Assert.Contains("--netcoredbg", result.Output);
    }
}

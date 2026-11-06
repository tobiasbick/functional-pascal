using System.IO.Compression;
using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed class NativeAssetTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("valid")]
    [InlineData("missing")]
    [InlineData("incompatible")]
    [InlineData("wrong_architecture")]
    public async Task Delivered_debugger_rejects_unusable_native_assets_before_target_execution(string mode)
    {
        using var program = new Debuggee("native-asset");
        await program.Compile("program NativeAssetGuard; uses System.IO.File as File; begin File.WriteAllText('ran.txt', 'ran'); end; end program;");
        using var extracted = new TemporaryDirectory();
        ZipFile.ExtractToDirectory(Path.Combine(sdk.Feed, $"Fpas.NET.Sdk.{sdk.Version}.nupkg"), extracted.Path);
        var windows = OperatingSystem.IsWindows();
        var tools = extracted.File("tools/compiler");
        var shim = Path.Combine(tools, windows ? "runtimes/win-x64/native/dbgshim.dll" : "runtimes/linux-x64/native/libdbgshim.so");
        Assert.True(File.Exists(shim));
        var fallback = extracted.File("external-native"); Directory.CreateDirectory(fallback);
        File.Copy(shim, Path.Combine(fallback, Path.GetFileName(shim)));
        var environment = new Dictionary<string, string?>
        {
            [windows ? "PATH" : "LD_LIBRARY_PATH"] = fallback + Path.PathSeparator + Environment.GetEnvironmentVariable(windows ? "PATH" : "LD_LIBRARY_PATH")
        };
        if (mode == "missing") File.Delete(shim);
        else if (mode == "incompatible") File.Copy(Path.Combine(tools,
            windows ? "runtimes/linux-x64/native/libdbgshim.so" : "runtimes/win-x64/native/dbgshim.dll"), shim, overwrite: true);
        else if (mode == "wrong_architecture")
            File.Copy(TestEnvironment.Metadata(windows ? "FpasWindowsArmShim" : "FpasLinuxArmShim"), shim, overwrite: true);

        var result = await ProcessRunner.RunAsync([Path.Combine(tools, "Fpas.Cli.dll"), "debug", program.Assembly,
            "--cwd", program.DirectoryPath, "--timeout", "10"], workingDirectory: Path.GetTempPath(), environment: environment, timeLimit: TimeSpan.FromSeconds(20));
        Assert.True(result.ExitCode == (mode == "valid" ? 0 : 1), result.Output + result.Error);
        Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal(mode == "valid" ? "completed" : "error", report.RootElement.GetProperty("outcome").GetString());
        Assert.Equal(mode == "valid", File.Exists(program.FilePath("ran.txt")));
        if (mode != "valid") Assert.NotEmpty(report.RootElement.GetProperty("issues").EnumerateArray());
        using var image = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
        using var symbols = new FileStream(Path.ChangeExtension(program.Assembly, ".pdb"), FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }
}

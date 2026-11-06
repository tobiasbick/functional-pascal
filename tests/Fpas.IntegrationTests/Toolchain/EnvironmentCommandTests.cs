using Fpas.Compiler;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;
using System.Diagnostics;
using System.IO.Compression;
using System.Security.Cryptography;
using System.Text.Json;

namespace Fpas.IntegrationTests.Toolchain;

public sealed class EnvironmentCommandTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private static string Cli => TestEnvironment.Metadata("FpasCliPath");

    [Fact]
    public async Task Actual_host_selection_and_loaded_compiler_are_reported_without_evaluating_projects_or_dumping_environment()
    {
        using var root = new TemporaryDirectory();
        File.WriteAllText(root.File("Broken.fpasproj"), "<invalid project");
        var before = Directory.GetFiles(root.Path);
        var environment = Settings(root);
        environment["FPAS_ENV_PRIVATE_TEST"] = "private-test-sentinel";
        var result = await Run(root, environment);
        using var json = JsonDocument.Parse(result.Output);
        var report = json.RootElement;
        Assert.Equal(0, result.ExitCode); Assert.Empty(result.Error);
        Assert.Equal(2, report.GetProperty("schemaVersion").GetInt32());
        Assert.True(report.GetProperty("success").GetBoolean());
        var selected = await ProcessRunner.RunAsync(["--version"], workingDirectory: root.Path);
        Assert.Equal(selected.Output.Trim(), report.GetProperty("dotnet").GetProperty("selectedSdk").GetString());
        var compiler = report.GetProperty("fpas").GetProperty("compiler");
        Assert.Equal(Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(typeof(FpasCompiler).Assembly.Location))), compiler.GetProperty("sha256").GetString());
        Assert.All(report.GetProperty("dotnet").GetProperty("sdks").EnumerateArray(), entry => Assert.True(Directory.Exists(entry.GetProperty("path").GetString())));
        Assert.Equal("available", report.GetProperty("debugger").GetProperty("status").GetString());
        Assert.False(report.TryGetProperty("netcoredbg", out _));
        Assert.Equal("not_implemented", report.GetProperty("languageServer").GetProperty("status").GetString());
        Assert.Contains(report.GetProperty("issues").EnumerateArray(), issue => issue.GetProperty("code").GetString() == "sdk_not_cached");
        Assert.DoesNotContain("private-test-sentinel", result.Output);
        Assert.Equal(before, Directory.GetFiles(root.Path)); Assert.False(Directory.Exists(root.File("obj")));
        Assert.Equal("<invalid project", File.ReadAllText(root.File("Broken.fpasproj")));
    }

    [Fact]
    public async Task Packaged_CLI_identifies_its_SDK_and_detects_same_version_cache_mismatch_even_in_a_large_cache()
    {
        using var root = new TemporaryDirectory();
        var package = Path.Combine(sdk.Feed, "Fpas.NET.Sdk." + sdk.Version + ".nupkg");
        var runner = root.File("Runner");
        var cached = root.File("Packages/fpas.net.sdk/" + sdk.Version.ToLowerInvariant());
        ZipFile.ExtractToDirectory(package, runner); ZipFile.ExtractToDirectory(package, cached);
        for (var index = 0; index < 140; index++) Directory.CreateDirectory(root.File("Packages/fpas.net.sdk/0.0." + index));
        var cli = Path.Combine(runner, "tools", "compiler", "Fpas.Cli.dll");
        var environment = Settings(root);
        var initial = await Run(root, environment, cli: cli);
        Assert.True(initial.ExitCode == 0, initial.Output + initial.Error); Assert.Empty(initial.Error);
        using (var json = JsonDocument.Parse(initial.Output))
        {
            Assert.Equal(sdk.Version, json.RootElement.GetProperty("fpas").GetProperty("expectedSdkVersion").GetString());
            Assert.True(json.RootElement.GetProperty("truncated").GetBoolean());
            var entry = Assert.Single(json.RootElement.GetProperty("sdkCache").GetProperty("installations").EnumerateArray(), item => item.GetProperty("expected").GetBoolean());
            Assert.Equal("matching", entry.GetProperty("compatibility").GetString());
            Assert.Equal(cached, entry.GetProperty("path").GetString());
        }
        var compiler = Path.Combine(cached, "tools", "compiler", "Fpas.Compiler.dll");
        using (var stream = File.Open(compiler, FileMode.Append, FileAccess.Write)) stream.WriteByte(0);
        var stale = await Run(root, environment, cli: cli);
        Assert.Equal(1, stale.ExitCode); Assert.Empty(stale.Error);
        using (var json = JsonDocument.Parse(stale.Output))
        {
            Assert.Equal("issues", json.RootElement.GetProperty("status").GetString());
            Assert.Contains(json.RootElement.GetProperty("issues").EnumerateArray(), issue => issue.GetProperty("code").GetString() == "sdk_mismatch");
            var entry = Assert.Single(json.RootElement.GetProperty("sdkCache").GetProperty("installations").EnumerateArray(), item => item.GetProperty("expected").GetBoolean());
            Assert.Equal("different", entry.GetProperty("compatibility").GetString());
        }
        File.Delete(Path.Combine(cached, "tools", "compiler", "Fpas.Core.dll"));
        var missing = await Run(root, environment, cli: cli);
        Assert.Equal(1, missing.ExitCode);
        using var incomplete = JsonDocument.Parse(missing.Output);
        Assert.Equal("incomplete", Assert.Single(incomplete.RootElement.GetProperty("sdkCache").GetProperty("installations").EnumerateArray(),
            entry => entry.GetProperty("expected").GetBoolean()).GetProperty("compatibility").GetString());

        File.WriteAllText(Assert.Single(Directory.GetFiles(runner, "*.nuspec")), "<package />");
        var malformed = await Run(root, environment, cli: cli);
        Assert.Equal(1, malformed.ExitCode); Assert.Empty(malformed.Error);
        using var failed = JsonDocument.Parse(malformed.Output);
        Assert.Contains(failed.RootElement.GetProperty("issues").EnumerateArray(), issue => issue.GetProperty("code").GetString() == "compiler_inspection_failed");
        Assert.True(failed.RootElement.GetProperty("dotnet").TryGetProperty("selectedSdk", out _));
    }

    [Theory]
    [InlineData("missing_host", "dotnet_unavailable")]
    [InlineData("pinned_sdk", "dotnet_sdk_unavailable")]
    public async Task Broken_dotnet_setup_retains_compiler_identity_and_inventory_evidence(string mode, string issue)
    {
        using var root = new TemporaryDirectory();
        var environment = Settings(root);
        if (mode == "missing_host") environment["DOTNET_HOST_PATH"] = root.File("absent-host");
        else File.WriteAllText(root.File("global.json"), """{"sdk":{"version":"999.0.100","rollForward":"disable"}}""");
        var result = await Run(root, environment);
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.True(json.RootElement.TryGetProperty("fpas", out _));
        Assert.Contains(json.RootElement.GetProperty("issues").EnumerateArray(), entry => entry.GetProperty("code").GetString() == issue);
        if (mode == "pinned_sdk") Assert.NotEmpty(json.RootElement.GetProperty("dotnet").GetProperty("sdks").EnumerateArray());
    }

    [Theory]
    [InlineData("--timeout", "0")]
    [InlineData("--timeout", "bad")]
    [InlineData("--netcoredbg", "relative-backend")]
    [InlineData("--directory", "absent-directory")]
    [InlineData("--unknown", "value")]
    public async Task Argument_errors_are_a_single_JSON_response(string option, string value)
    {
        using var root = new TemporaryDirectory();
        var result = await Run(root, Settings(root), [option, value]);
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", Assert.Single(json.RootElement.GetProperty("issues").EnumerateArray()).GetProperty("code").GetString());
    }

    [Fact]
    public async Task Packaged_own_debugger_reports_its_shim_without_probing_an_external_backend()
    {
        using var root = new TemporaryDirectory();
        var environment = Settings(root);
        environment["FPAS_NETCOREDBG_PATH"] = root.File("absent-backend");
        var result = await Run(root, environment);
        Assert.Equal(0, result.ExitCode); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var debugger = json.RootElement.GetProperty("debugger");
        Assert.Equal("available", debugger.GetProperty("status").GetString());
        Assert.Equal("10.0.750501", debugger.GetProperty("shimVersion").GetString());
        var shim = debugger.GetProperty("shimPath").GetString()!;
        Assert.Equal(Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(shim))), debugger.GetProperty("shimSha256").GetString());
        Assert.Equal(new[] { "batch", "dap" }, debugger.GetProperty("protocols").EnumerateArray().Select(item => item.GetString()));
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Deadline_and_console_cancellation_join_a_waiting_version_probe(bool cancel)
    {
        using var root = new TemporaryDirectory();
        var probe = await EnvironmentProbe.Build(root, sdk);
        var environment = Settings(root); environment["FPAS_ENV_PROBE_READY"] = root.File("ready.pid");
        environment["DOTNET_HOST_PATH"] = probe;
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        await using var process = InterruptibleProcess.Start(BuildProcess.DotnetHost(),
            [Cli, "env", "--json", "--timeout", cancel ? "30" : "5"], root.Path,
            environment.ToDictionary(pair => pair.Key, pair => pair.Value!));
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        while (!File.Exists(root.File("ready.pid")))
        {
            Assert.False(process.HasExited, "Environment inspection exited before the version probe became ready.");
            await Task.Delay(20, deadline.Token);
        }
        var pid = int.Parse(File.ReadAllText(root.File("ready.pid")), System.Globalization.CultureInfo.InvariantCulture);
        if (cancel) await process.Interrupt();
        await process.WaitForExit(deadline.Token);
        Assert.Equal(cancel ? 130 : 1, process.ExitCode); Assert.Empty(await error);
        using var json = JsonDocument.Parse(await output);
        Assert.Equal(cancel ? "cancelled" : "timeout", json.RootElement.GetProperty("status").GetString());
        Assert.Throws<ArgumentException>(() => Process.GetProcessById(pid));
    }

    private static Dictionary<string, string?> Settings(TemporaryDirectory root)
    {
        var host = BuildProcess.DotnetHost();
        return new()
        {
            ["DOTNET_HOST_PATH"] = host, ["DOTNET_ROOT"] = Path.GetDirectoryName(host), ["PATH"] = Path.GetDirectoryName(host),
            ["NUGET_PACKAGES"] = root.File("Packages"), ["FPAS_NETCOREDBG_PATH"] = null
        };
    }

    private static Task<ProcessResult> Run(TemporaryDirectory root, IReadOnlyDictionary<string, string?> environment, string[]? arguments = null, string? cli = null)
        => ProcessRunner.RunAsync([cli ?? Cli, "env", "--json", .. arguments ?? []], workingDirectory: root.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(45));
}

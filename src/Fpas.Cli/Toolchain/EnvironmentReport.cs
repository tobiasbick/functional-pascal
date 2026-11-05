using Fpas.Cli.Diagnostics;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Toolchain;

internal sealed record EnvironmentIssue(string Severity, string Code, string Message, string? Path = null);
internal sealed record ManagedInstallation(string Path, string Version, string Sha256);
internal sealed record CompilerInstallation(ManagedInstallation Cli, ManagedInstallation Compiler, ManagedInstallation Core, string ExpectedSdkVersion);
internal sealed record RuntimeInstallation(string Framework, string Os, string Architecture);
internal sealed record DotnetSdk(string Version, string Path);
internal sealed record DotnetRuntime(string Name, string Version, string Path);
internal sealed record DotnetInstallation(string Status, string? Path, string? SelectedSdk,
    IReadOnlyList<DotnetSdk> Sdks, IReadOnlyList<DotnetRuntime> Runtimes, string? GlobalPackages);
internal sealed record CachedSdk(string Version, string Path, bool Expected, string Compatibility, string? CompilerSha256, string? CoreSha256);
internal sealed record SdkCache(string Status, string? Path, IReadOnlyList<CachedSdk> Installations);
internal sealed record ToolInstallation(string Status, string? Path = null, string? Version = null, string? Source = null);
internal sealed record EnvironmentReport(bool Success, string Status, string Directory)
{
    public int SchemaVersion => 1;
    public CompilerInstallation? Fpas { get; init; }
    public RuntimeInstallation? Runtime { get; init; }
    public DotnetInstallation? Dotnet { get; init; }
    public SdkCache? SdkCache { get; init; }
    public ToolInstallation? Netcoredbg { get; init; }
    public ToolInstallation LanguageServer { get; init; } = new("not_implemented");
    public IReadOnlyList<EnvironmentIssue> Issues { get; init; } = [];
    public bool Truncated { get; init; }

    internal bool Write(bool json)
    {
        var output = JsonSerializer.Serialize(this, EnvironmentJson.Default.EnvironmentReport);
        if (Encoding.UTF8.GetByteCount(output) > 1_048_576)
        {
            new EnvironmentReport(false, "failed", Directory)
            { Truncated = true, Issues = [new("error", "result_limit", "Environment report exceeded 1 MiB.")] }.Write(json);
            return false;
        }
        if (json) { Console.WriteLine(output); return true; }
        void Line(string text) => Console.WriteLine(DiagnosticRenderer.Safe(text));
        Line("FPAS");
        if (Fpas is { } fpas)
        {
            Line($"  CLI: {fpas.Cli.Version} ({fpas.Cli.Path})");
            Line($"  Compiler: {fpas.Compiler.Version} ({fpas.Compiler.Path})");
            Line($"  Core: {fpas.Core.Version} ({fpas.Core.Path})");
            Line($"  Expected SDK: {fpas.ExpectedSdkVersion}");
        }
        Line(""); Line(".NET");
        if (Runtime is { } runtime) Line($"  Running: {runtime.Framework}; {runtime.Os}; {runtime.Architecture}");
        if (Dotnet is { } dotnet)
        {
            Line($"  Host: {dotnet.Path ?? dotnet.Status}");
            Line($"  Selected SDK: {dotnet.SelectedSdk ?? "unavailable"}");
            foreach (var sdk in dotnet.Sdks) Line($"  SDK: {sdk.Version} ({sdk.Path})");
            foreach (var installed in dotnet.Runtimes) Line($"  Runtime: {installed.Name} {installed.Version} ({installed.Path})");
        }
        Line(""); Line("FPAS SDK cache");
        Line("  " + (SdkCache?.Path ?? "unavailable"));
        foreach (var sdk in SdkCache?.Installations ?? []) Line($"  {sdk.Version}: {sdk.Compatibility}{(sdk.Expected ? " (expected)" : "")} ({sdk.Path})");
        Line(""); Line($"NetCoreDbg: {Netcoredbg?.Status ?? "unavailable"}");
        if (Netcoredbg?.Path is { } path) Line($"  {Netcoredbg.Version ?? "unverified version"} ({path}; {Netcoredbg.Source})");
        Line("Language server: not implemented");
        if (Issues.Count != 0) { Line(""); foreach (var issue in Issues) Line($"{issue.Severity} {issue.Code}: {issue.Message}" + (issue.Path is null ? "" : $" ({issue.Path})")); }
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(EnvironmentReport))]
internal partial class EnvironmentJson : JsonSerializerContext;

using System.Runtime.InteropServices;
using System.Security.Cryptography;
using Fpas.Debugging.Native;

namespace Fpas.Debugging;

public sealed record DebuggerDeployment(string Status, string Version, string ShimVersion, string? ShimPath, string? ShimSha256,
    string Os, string Architecture, string RequiredRuntime, string[] Protocols)
{
    public static async Task<DebuggerDeployment> Inspect(CancellationToken token = default)
    {
        var os = OperatingSystem.IsWindows() ? "win" : OperatingSystem.IsLinux() ? "linux" : "unsupported";
        var architecture = RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant();
        var path = FindShim();
        string? hash = null;
        if (path is not null) { await using var file = File.OpenRead(path); hash = Convert.ToHexString(await SHA256.HashDataAsync(file, token)); }
        return new(os == "unsupported" || architecture != "x64" ? "unsupported_platform" : path is null ? "missing_shim" : "available",
            typeof(DebuggerDeployment).Assembly.GetName().Version!.ToString(), DbgShim.Version, path, hash, os, architecture, ".NET 10", ["batch", "dap"]);
    }

    internal static string? FindShim()
    {
        var os = OperatingSystem.IsWindows() ? "win" : OperatingSystem.IsLinux() ? "linux" : "unsupported";
        var architecture = RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant();
        var name = os == "win" ? "dbgshim.dll" : "libdbgshim.so";
        if (Fpas.Deployment.DistributionAssets.IsBundled)
            return Fpas.Deployment.DistributionAssets.Materialize(name);
        return new[] { Path.Combine(AppContext.BaseDirectory, "runtimes", os + "-" + architecture, "native", name),
            Path.Combine(AppContext.BaseDirectory, name) }.FirstOrDefault(File.Exists);
    }
}

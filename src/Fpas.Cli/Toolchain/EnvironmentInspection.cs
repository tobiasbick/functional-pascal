using Fpas.Compiler;
using Fpas.Cli.Projects;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

namespace Fpas.Cli.Toolchain;

internal static class EnvironmentInspection
{
    internal static async Task<EnvironmentReport> Read(string directory, string? debugger, CancellationToken token)
    {
        var issues = new List<EnvironmentIssue>();
        var runtime = new RuntimeInstallation(RuntimeInformation.FrameworkDescription,
            OperatingSystem.IsWindows() ? "windows" : OperatingSystem.IsLinux() ? "linux" : "unsupported",
            RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant());
        CompilerInstallation? compiler = null;
        try
        {
            compiler = new(await Managed(typeof(EnvironmentInspection).Assembly, token), await Managed(typeof(FpasCompiler).Assembly, token),
                await Managed(typeof(Core.RecordAttribute).Assembly, token), ToolSdk.Version);
        }
        catch (Exception exception) when (Recoverable(exception)) { issues.Add(new("error", "compiler_inspection_failed", exception.Message)); }
        var (dotnet, truncated) = await DotnetInspection.Read(directory, issues, token);
        var (cache, cacheTruncated) = await InstalledSdkInspection.Read(dotnet.GlobalPackages, compiler, issues, token);
        var backend = await DebuggerInspection.Read(directory, debugger, issues, token);
        token.ThrowIfCancellationRequested();
        var success = !issues.Any(issue => issue.Severity == "error");
        return new(success, success ? "ok" : "issues", directory)
        { Fpas = compiler, Runtime = runtime, Dotnet = dotnet, SdkCache = cache, Netcoredbg = backend, Issues = issues.AsReadOnly(), Truncated = truncated || cacheTruncated };
    }

    private static async Task<ManagedInstallation> Managed(Assembly assembly, CancellationToken token)
        => new(assembly.Location, assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? assembly.GetName().Version!.ToString(),
            await Hash(assembly.Location, token));

    internal static async Task<string> Hash(string path, CancellationToken token)
    {
        await using var stream = File.OpenRead(path);
        if (stream.Length > 64 * 1024 * 1024) throw new IOException("A toolchain assembly exceeds 64 MiB.");
        return Convert.ToHexString(await SHA256.HashDataAsync(stream, token));
    }

    internal static bool Recoverable(Exception exception)
        => exception is IOException or UnauthorizedAccessException or ArgumentException or System.Xml.XmlException or
            System.ComponentModel.Win32Exception or PlatformNotSupportedException or System.Security.SecurityException;
}

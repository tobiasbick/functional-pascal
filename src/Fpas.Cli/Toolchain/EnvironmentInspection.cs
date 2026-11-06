using Fpas.Compiler;
using Fpas.Cli.Projects;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

namespace Fpas.Cli.Toolchain;

internal static class EnvironmentInspection
{
    internal static async Task<EnvironmentReport> Read(string directory, CancellationToken token)
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
        LanguageServerInstallation languageServer = new("unavailable");
        try { languageServer = new("available", await Managed(typeof(Fpas.LanguageServer.LanguageServerApplication).Assembly, token)); }
        catch (Exception exception) when (Recoverable(exception)) { issues.Add(new("error", "language_server_unavailable", exception.Message)); }
        var (dotnet, truncated) = await DotnetInspection.Read(directory, issues, token);
        var (cache, cacheTruncated) = await InstalledSdkInspection.Read(dotnet.GlobalPackages, compiler, issues, token);
        var debugger = await Fpas.Debugging.DebuggerDeployment.Inspect(token);
        if (debugger.Status != "available") issues.Add(new("error", "debugger_unavailable", "The packaged debugger shim is unavailable for this platform."));
        token.ThrowIfCancellationRequested();
        var success = !issues.Any(issue => issue.Severity == "error");
        return new(success, success ? "ok" : "issues", directory)
        { Fpas = compiler, Runtime = runtime, Dotnet = dotnet, SdkCache = cache, Debugger = debugger, LanguageServer = languageServer,
            Issues = issues.AsReadOnly(), Truncated = truncated || cacheTruncated };
    }

    private static async Task<ManagedInstallation> Managed(Assembly assembly, CancellationToken token)
    {
        var executable = Fpas.Deployment.DistributionAssets.IsBundled && assembly == typeof(EnvironmentInspection).Assembly;
        var path = executable ? Environment.ProcessPath! : Fpas.Deployment.DistributionAssets.Managed(assembly);
        return new(path, assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? assembly.GetName().Version!.ToString(),
            await Hash(path, token, executable));
    }

    internal static async Task<string> Hash(string path, CancellationToken token, bool executable = false)
    {
        await using var stream = File.OpenRead(path);
        if (stream.Length > (executable ? 1024L : 64L) * 1024 * 1024)
            throw new IOException(executable ? "The compiler executable exceeds 1 GiB." : "A toolchain assembly exceeds 64 MiB.");
        return Convert.ToHexString(await SHA256.HashDataAsync(stream, token));
    }

    internal static bool Recoverable(Exception exception)
        => exception is IOException or UnauthorizedAccessException or ArgumentException or System.Xml.XmlException or
            System.ComponentModel.Win32Exception or PlatformNotSupportedException or System.Security.SecurityException;
}

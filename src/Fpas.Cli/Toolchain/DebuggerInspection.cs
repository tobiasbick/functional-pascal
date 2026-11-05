using Fpas.Processes;
using Fpas.Workspaces.Processes;
using System.Text.RegularExpressions;

namespace Fpas.Cli.Toolchain;

internal static partial class DebuggerInspection
{
    internal static async Task<ToolInstallation> Read(string directory, string? explicitPath, List<EnvironmentIssue> issues, CancellationToken token)
    {
        var path = explicitPath ?? Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH");
        var source = explicitPath is not null ? "option" : string.IsNullOrEmpty(path) ? "path" : "FPAS_NETCOREDBG_PATH";
        try
        {
            if (string.IsNullOrEmpty(path))
            {
                var name = OperatingSystem.IsWindows() ? "netcoredbg.exe" : "netcoredbg";
                foreach (var entry in (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator))
                {
                    token.ThrowIfCancellationRequested();
                    if (string.IsNullOrEmpty(entry)) continue;
                    var candidate = Path.GetFullPath(Path.Combine(entry, name), directory);
                    if (File.Exists(candidate)) { path = candidate; break; }
                }
            }
            if (string.IsNullOrEmpty(path)) return new("not_found");
            OwnedProcess.Validate(path);
            var result = await BuildProcess.RunTool(path, directory, ["--version"], token);
            var match = VersionLine().Match(result.Output);
            if (result.ExitCode != 0 || result.Truncated || !match.Success)
            {
                issues.Add(new("error", "debugger_probe_failed", "NetCoreDbg did not return a recognized bounded version response.", path));
                return new("unverified", path, Source: source);
            }
            return new("available", path, match.Groups[1].Value, source);
        }
        catch (Exception exception) when (EnvironmentInspection.Recoverable(exception))
        { issues.Add(new("error", "debugger_unavailable", exception.Message, path)); return new("unavailable", path, Source: source); }
    }

    [GeneratedRegex(@"(?m)^NET Core debugger ([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)(?:\s|$)")]
    private static partial Regex VersionLine();
}

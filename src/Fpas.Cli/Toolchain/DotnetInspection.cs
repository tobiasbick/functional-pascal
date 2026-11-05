using Fpas.Processes;
using Fpas.Workspaces.Processes;
using System.Text.RegularExpressions;

namespace Fpas.Cli.Toolchain;

internal static partial class DotnetInspection
{
    internal static async Task<(DotnetInstallation Installation, bool Truncated)> Read(string directory, List<EnvironmentIssue> issues, CancellationToken token)
    {
        string? path = null; string? selected = null;
        var sdks = new List<DotnetSdk>(); var runtimes = new List<DotnetRuntime>(); var truncated = false;
        var packages = Environment.GetEnvironmentVariable("NUGET_PACKAGES");
        try { path = BuildProcess.DotnetHost(); OwnedProcess.Validate(path); }
        catch (Exception exception) when (EnvironmentInspection.Recoverable(exception))
        {
            issues.Add(new("error", "dotnet_unavailable", exception.Message, path));
            return (new("unavailable", path, null, [], [], NormalizePackages()), false);
        }
        var sdkOutput = await Probe(["--list-sdks"], "dotnet_sdks_failed");
        foreach (var line in Lines(sdkOutput))
        {
            var match = SdkLine().Match(line);
            if (!match.Success || !VersionText().IsMatch(match.Groups[1].Value) || !Path.IsPathFullyQualified(match.Groups[2].Value))
            { issues.Add(new("error", "dotnet_sdks_invalid", "Cannot parse the installed .NET SDK inventory.")); break; }
            sdks.Add(new(match.Groups[1].Value, Path.Combine(match.Groups[2].Value, match.Groups[1].Value)));
        }
        var runtimeOutput = await Probe(["--list-runtimes"], "dotnet_runtimes_failed");
        foreach (var line in Lines(runtimeOutput))
        {
            var match = RuntimeLine().Match(line);
            if (!match.Success || !VersionText().IsMatch(match.Groups[2].Value) || !Path.IsPathFullyQualified(match.Groups[3].Value))
            { issues.Add(new("error", "dotnet_runtimes_invalid", "Cannot parse the installed .NET runtime inventory.")); break; }
            runtimes.Add(new(match.Groups[1].Value, match.Groups[2].Value, Path.Combine(match.Groups[3].Value, match.Groups[2].Value)));
        }
        var version = (await Probe(["--version"], "dotnet_sdk_unavailable"))?.Trim();
        if (version is not null)
        {
            if (VersionText().IsMatch(version)) selected = version;
            else issues.Add(new("error", "dotnet_version_invalid", "Cannot identify the .NET SDK selected for the working directory."));
        }
        if (sdks.Count == 0) issues.Add(new("error", "dotnet_sdk_missing", "No .NET SDK is installed for the selected host."));
        if (string.IsNullOrEmpty(packages) && selected is not null)
        {
            var output = (await Probe(["nuget", "locals", "global-packages", "--list", "--force-english-output"], "nuget_cache_unavailable"))?.Trim();
            const string prefix = "global-packages: ";
            if (output?.StartsWith(prefix, StringComparison.Ordinal) == true) packages = output[prefix.Length..].Trim();
            else if (output is not null) issues.Add(new("error", "nuget_cache_invalid", "Cannot identify the active NuGet global-packages directory."));
        }
        return (new(selected is null ? "unavailable" : "available", path, selected, sdks.AsReadOnly(), runtimes.AsReadOnly(), NormalizePackages()), truncated);

        string? NormalizePackages()
        {
            if (string.IsNullOrEmpty(packages)) return null;
            if (Path.IsPathFullyQualified(packages)) return Path.GetFullPath(packages);
            issues.Add(new("error", "nuget_cache_invalid", "NuGet global-packages must be an absolute directory."));
            return null;
        }

        IEnumerable<string> Lines(string? output)
        {
            var lines = (output ?? "").Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Take(129).ToArray();
            if (lines.Length > 128) { truncated = true; issues.Add(new("error", "inventory_limit", "A .NET inventory exceeded 128 entries.")); }
            return lines.Take(128);
        }

        async Task<string?> Probe(string[] arguments, string code)
        {
            try
            {
                var result = await BuildProcess.RunTool(path, directory, arguments, token);
                if (result.ExitCode == 0 && !result.Truncated) return result.Output;
                truncated |= result.Truncated;
                issues.Add(new("error", code, "The selected dotnet host could not complete " + string.Join(' ', arguments) + ".", path));
            }
            catch (Exception exception) when (EnvironmentInspection.Recoverable(exception)) { issues.Add(new("error", code, exception.Message, path)); }
            return null;
        }
    }

    [GeneratedRegex(@"^(\S+) \[(.+)\]$")] private static partial Regex SdkLine();
    [GeneratedRegex(@"^(\S+) (\S+) \[(.+)\]$")] private static partial Regex RuntimeLine();
    [GeneratedRegex(@"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.+-]+)?$")] private static partial Regex VersionText();
}

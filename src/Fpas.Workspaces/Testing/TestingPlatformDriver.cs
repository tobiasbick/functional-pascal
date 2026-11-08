using System.Text.Json;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;

namespace Fpas.Workspaces.Testing;

internal static class TestingPlatformDriver
{
    internal static Task<BuildResult> Run(TestProject project, string directory, string? filter, string? settings, TestRunOptions? options,
        bool list, CancellationToken token, ProcessOutput? liveOutput = null)
    {
        if (options is { AdapterPaths.Count: > 0 } or { Collectors.Count: > 0 })
            throw new ContextFailure("invalid_argument", "--adapter-path and --collect are VSTest options. Pass MTP extension options after --.");
        var args = new List<string> { project.Assembly, "--no-banner", "--results-directory", directory };
        if (list) args.AddRange(["--list-tests", "json"]);
        else args.AddRange(["--report-trx", "--report-trx-filename", "results.trx"]);
        if (filter is not null) args.AddRange(["--filter", filter]);
        if (settings is not null) args.AddRange(["--settings", settings]);
        foreach (var argument in options?.Arguments ?? [])
        {
            var name = argument.Split('=')[0];
            if (argument.StartsWith('@') || new[] { "--list-tests", "--report-trx", "--report-trx-filename", "--results-directory", "--ignore-exit-code", "--help", "--info", "--server", "--config-file" }.Contains(name, StringComparer.Ordinal))
                throw new ContextFailure("invalid_argument", "FPAS owns MTP mode, result location and exit status; do not override " + name + ".");
            args.Add(argument);
        }
        return BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token, captureCancellation: true, liveOutput: liveOutput);
    }

    internal static async Task<(BuildResult Process, IReadOnlyList<DiscoveredTest> Tests)> Discover(TestProject project, string directory, string? filter,
        string? settings, TestRunOptions? options, CancellationToken token)
    {
        var process = await Run(project, directory, filter, settings, options, list: true, token);
        if (process.ExitCode != 0 || process.Truncated) return (process, []);
        IReadOnlyList<DiscoveredTest> tests;
        try { tests = ReadDiscovery(project, process.Output, token); }
        catch (Exception exception) when (exception is JsonException or KeyNotFoundException or InvalidOperationException or FormatException)
        { throw new ContextFailure("discovery_failed", "Native MTP discovery did not return valid schemaVersion 1 JSON: " + exception.Message); }
        await File.WriteAllTextAsync(Path.Combine(directory, "discovery.json"), process.Output, token);
        return (process, tests);
    }

    private static IReadOnlyList<DiscoveredTest> ReadDiscovery(TestProject project, string output, CancellationToken token)
    {
        using var json = JsonDocument.Parse(output);
        var root = json.RootElement;
        if (root.GetProperty("schemaVersion").GetInt32() != 1) throw new ContextFailure("discovery_failed", "Unsupported MTP JSON discovery schema; use MTP 2.3+ with schemaVersion 1.");
        var tests = new List<DiscoveredTest>(); var uids = new HashSet<string>(StringComparer.Ordinal);
        foreach (var item in root.GetProperty("tests").EnumerateArray())
        {
            token.ThrowIfCancellationRequested();
            var uid = item.GetProperty("uid").GetString()!;
            if (string.IsNullOrEmpty(uid) || !uids.Add(uid)) throw new ContextFailure("discovery_failed", "MTP discovery returned missing or duplicate test UIDs.");
            var name = uid;
            if (item.TryGetProperty("type", out var type))
            {
                var typeName = type.TryGetProperty("typeName", out var declaring) ? declaring.GetString() : null;
                var methodName = type.TryGetProperty("methodName", out var method) ? method.GetString() : null;
                var scope = type.TryGetProperty("namespace", out var ns) ? ns.GetString() : null;
                if (!string.IsNullOrEmpty(typeName) && !string.IsNullOrEmpty(methodName))
                    name = (!string.IsNullOrEmpty(scope) && !typeName.StartsWith(scope + ".", StringComparison.Ordinal) ? scope + "." : "") + typeName + "." + methodName;
            }
            tests.Add(new(project.Project, name) { Uid = uid, DisplayName = item.GetProperty("displayName").GetString() });
        }
        return tests.OrderBy(test => test.FullyQualifiedName, StringComparer.Ordinal).ThenBy(test => test.Uid, StringComparer.Ordinal).ToArray();
    }
}

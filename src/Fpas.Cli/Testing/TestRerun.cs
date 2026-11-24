// [Documentation](../../../docs/pascal/tools/testing.md#repeat-a-failed-case)
using Fpas.Workspaces.Testing;
using System.Text;
using System.Text.Json;

namespace Fpas.Cli.Testing;

internal sealed record TestRerun(int CaseIndex, string Scope, IReadOnlyList<string> Arguments)
{
    internal static IReadOnlyList<TestRerun> Create(TrxReport report, TestProject project, string context, string configuration,
        string? settings, string? resultsDirectory, int timeout, TestRunOptions options, ref int byteBudget)
    {
        if (byteBudget <= 0 || project is { OutputIdentity: null } or { StaleSources: true }) return [];
        var failures = report.Tests.Select((test, index) => (Test: test, Index: index)).Where(item => item.Test.Outcome == "Failed").ToArray();
        if (failures.Length == 0) return [];
        var uid = project.TestingPlatform && report.Tests.Count == 1 ? SelectedUid(options.Arguments) : null;
        IReadOnlyList<string> native;
        if (project.TestingPlatform)
        {
            var originalOptions = WithoutSelection(options.Arguments);
            native = uid is null ? [.. originalOptions, "--minimum-expected-tests", "1"]
                : [.. originalOptions, "--filter-uid", uid, "--minimum-expected-tests", "1"];
        }
        else native = [.. options.Arguments.Where(argument => !argument.StartsWith("RunConfiguration.TreatNoTestsAsError=", StringComparison.OrdinalIgnoreCase)),
            "RunConfiguration.TreatNoTestsAsError=true"];
        if (!TestCommand.AcceptsNativeArguments(native)) return [];
        var common = new List<string> { "test", context, "--project", project.Project, "--configuration", configuration,
            "--no-build", "--expected-output", project.OutputIdentity, "--timeout", timeout.ToString(System.Globalization.CultureInfo.InvariantCulture),
            "--report", "json" };
        if (settings is not null) common.AddRange(["--settings", settings]);
        if (resultsDirectory is not null) common.AddRange(["--results-directory", resultsDirectory]);
        foreach (var path in options.AdapterPaths) common.AddRange(["--adapter-path", path]);
        foreach (var collector in options.Collectors) common.AddRange(["--collect", collector]);

        var reruns = new List<TestRerun>(failures.Length);
        foreach (var (test, index) in failures)
        {
            TestRerun rerun;
            if (uid is not null) rerun = new(index, "case", [.. common, "--", .. native]);
            else
            {
                var name = test.FullyQualifiedName;
                if (string.IsNullOrWhiteSpace(name) || name.IndexOf('.') <= 0 || name.EndsWith('.') || name.Any(char.IsControl)) continue;
                var filter = "FullyQualifiedName=" + Escape(name);
                rerun = new(index, "method", [.. common, "--filter", filter, "--", .. native]);
            }
            byteBudget -= JsonSerializer.SerializeToUtf8Bytes(rerun, TestJson.Default.TestRerun).Length + 1;
            if (byteBudget < 0) break;
            reruns.Add(rerun);
        }
        return reruns;
    }

    private static string? SelectedUid(IReadOnlyList<string> args)
    {
        string? uid = null;
        for (var index = 0; index < args.Count; index++)
        {
            var (name, value) = Split(args[index]);
            if (name != "--filter-uid") continue;
            if (uid is not null) return null;
            uid = value ?? (index + 1 < args.Count ? args[++index] : null);
            if (string.IsNullOrWhiteSpace(uid) || uid.StartsWith('-') || uid.Contains(',')) return null;
        }
        return uid;
    }

    private static IReadOnlyList<string> WithoutSelection(IReadOnlyList<string> args)
    {
        var relevant = new List<string>();
        for (var index = 0; index < args.Count; index++)
        {
            var (name, value) = Split(args[index]);
            if (name is "--filter" or "--filter-uid" or "--minimum-expected-tests")
            {
                if (value is null) index++;
            }
            else relevant.Add(args[index]);
        }
        return relevant;
    }

    private static (string Name, string? Value) Split(string argument)
    {
        var equals = argument.IndexOf('=');
        return equals < 0 ? (argument, null) : (argument[..equals], argument[(equals + 1)..]);
    }

    private static string Escape(string name)
    {
        var result = new StringBuilder(name.Length);
        foreach (var character in name)
        {
            if ("\\()&|=!~".Contains(character)) result.Append('\\');
            result.Append(character);
        }
        return result.ToString();
    }
}

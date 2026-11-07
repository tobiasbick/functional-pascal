using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using Fpas.Workspaces.Solutions;
using Microsoft.VisualStudio.SolutionPersistence.Model;
using System.Text.Json;

namespace Fpas.Workspaces.Testing;

public sealed record TestProject(string Project, string Configuration, string Platform, string Framework, string Assembly)
{
    public bool TestingPlatform { get; init; }
    public string FrameworkMoniker { get; init; } = "";
    public IReadOnlyList<string> VstestExtensionPaths { get; init; } = [];
}
public sealed record DiscoveredTest(string Project, string FullyQualifiedName)
{
    public string? Uid { get; init; }
    public string? DisplayName { get; init; }
}
public sealed record TestRunOptions(IReadOnlyList<string> AdapterPaths, IReadOnlyList<string> Collectors, IReadOnlyList<string> Arguments);

public static class TestDriver
{
    public static async Task<IReadOnlyList<TestProject>> SelectAsync(string context, string configuration, string? project, CancellationToken token)
    {
        context = ProjectDriver.ValidateContext(context);
        if (string.IsNullOrWhiteSpace(configuration)) throw new ContextFailure("invalid_argument", "Configuration must not be empty.");
        SolutionScope scope;
        try { scope = await SolutionScope.Read(context, configuration, token); }
        catch (SolutionException exception) { throw new ContextFailure("invalid_argument", "Cannot read solution: " + exception.Message); }
        var candidates = scope.Projects;
        if (project is not null)
        {
            var path = Path.GetFullPath(project);
            candidates = candidates.Where(candidate => (OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal).Equals(candidate.Path, path)).ToArray();
            if (candidates.Count != 1) throw new ContextFailure("invalid_argument", "--project must select an included project in the context.");
        }
        var selected = new List<TestProject>();
        foreach (var candidate in candidates)
        {
            var args = new List<string> { "msbuild", candidate.Path, "-nologo", "-nr:false", "-p:UseSharedCompilation=false", "-t:GetTargetPath",
                "-p:Configuration=" + MsBuildSyntax.Escape(candidate.Configuration), "-p:Platform=" + MsBuildSyntax.Escape(candidate.Platform),
                "-getProperty:IsTestProject,IsTestingPlatformApplication,OutputType,TargetPath,TargetFramework,TargetFrameworkMoniker,VSTestTestAdapterPath,TraceDataCollectorDirectoryPath" };
            if (scope.ConfigurationXml is not null)
                args.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + MsBuildSyntax.Escape(scope.Directory!),
                    "-p:CurrentSolutionConfigurationContents=" + MsBuildSyntax.Escape(scope.ConfigurationXml)]);
            var result = await BuildProcess.Run(Path.GetDirectoryName(candidate.Path)!, args, token);
            if (result.ExitCode != 0 || result.Truncated) throw new ContextFailure("evaluation_failed", result.Output + result.Error, result.Truncated);
            using var json = JsonDocument.Parse(result.Output);
            var properties = json.RootElement.GetProperty("Properties");
            if (!properties.GetProperty("IsTestProject").GetString()!.Equals("true", StringComparison.OrdinalIgnoreCase)) continue;
            var mtp = properties.GetProperty("IsTestingPlatformApplication").GetString()!.Equals("true", StringComparison.OrdinalIgnoreCase);
            if (mtp && !properties.GetProperty("OutputType").GetString()!.Equals("Exe", StringComparison.OrdinalIgnoreCase))
                throw new ContextFailure("invalid_argument", "An MTP application requires OutputType Exe and an explicit program entry point.");
            var target = properties.GetProperty("TargetPath").GetString(); var framework = properties.GetProperty("TargetFramework").GetString();
            if (string.IsNullOrWhiteSpace(target) || string.IsNullOrWhiteSpace(framework))
                throw new ContextFailure("invalid_argument", "Select test projects with one explicit TargetFramework and TargetPath.");
            var extensions = new[] { "VSTestTestAdapterPath", "TraceDataCollectorDirectoryPath" }
                .SelectMany(name => properties.GetProperty(name).GetString()!.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
                .Select(path => Path.GetFullPath(path, Path.GetDirectoryName(candidate.Path)!)).Distinct(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal).ToArray();
            selected.Add(new(candidate.Path, candidate.Configuration, candidate.Platform, framework, Path.GetFullPath(target, Path.GetDirectoryName(candidate.Path)!))
                { TestingPlatform = mtp, FrameworkMoniker = properties.GetProperty("TargetFrameworkMoniker").GetString()!, VstestExtensionPaths = mtp ? [] : extensions });
        }
        if (selected.Count == 0) throw new ContextFailure("not_test_project", "The selected context contains no enabled test projects.");
        return selected.AsReadOnly();
    }

    public static async Task<BuildResult> RunAsync(TestProject project, string directory, string? filter, string? settings, CancellationToken token, TestRunOptions? options = null)
    {
        if (project.TestingPlatform) return await TestingPlatformDriver.Run(project, directory, filter, settings, options, list: false, token);
        var assembly = await VstestDeployment.Prepare(project, directory, options, token);
        var args = new List<string> { "vstest", assembly, "/Logger:trx;LogFileName=results.trx", "/ResultsDirectory:" + directory };
        if (filter is not null) args.Add("/TestCaseFilter:" + filter);
        if (settings is not null) args.Add("/Settings:" + settings);
        VstestOptions(project, args, options);
        return await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token);
    }

    public static async Task<(BuildResult Process, IReadOnlyList<DiscoveredTest> Tests)> DiscoverAsync(TestProject project,
        string directory, string? filter, string? settings, CancellationToken token, TestRunOptions? options = null)
    {
        RequireOutput(project);
        if (project.TestingPlatform) return await TestingPlatformDriver.Discover(project, directory, filter, settings, options, token);
        var assembly = await VstestDeployment.Prepare(project, directory, options, token);
        var output = Path.Combine(directory, "discovery.txt");
        var args = new List<string> { "vstest", assembly, "/ListFullyQualifiedTests", "/ListTestsTargetPath:" + output };
        if (filter is not null) args.Add("/TestCaseFilter:" + filter);
        if (settings is not null) args.Add("/Settings:" + settings);
        VstestOptions(project, args, options);
        var process = await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token);
        if (process.ExitCode != 0 || process.Truncated) return (process, []);
        if (!File.Exists(output) || new FileInfo(output).Length > 4_194_304)
            throw new ContextFailure("discovery_failed", "Native discovery did not produce a bounded test-name artifact.");
        var names = await File.ReadAllLinesAsync(output, token);
        return (process, Array.AsReadOnly(names.Where(name => name.Length != 0).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal)
            .Select(name => new DiscoveredTest(project.Project, name)).ToArray()));
    }

    private static void VstestOptions(TestProject project, List<string> args, TestRunOptions? options)
    {
        if (!string.IsNullOrEmpty(project.FrameworkMoniker)) args.Add("/Framework:" + project.FrameworkMoniker);
        var paths = project.VstestExtensionPaths.Concat(options?.AdapterPaths ?? []).Distinct(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal).ToArray();
        if (paths.Length != 0) args.Add("/TestAdapterPath:" + string.Join(';', paths));
        foreach (var collector in options?.Collectors ?? []) args.Add("/Collect:" + collector);
        var extra = options?.Arguments ?? [];
        if (extra.Count != 0) args.Add("--");
        args.AddRange(extra);
    }

    public static void RequireOutput(TestProject project)
    {
        if (!File.Exists(project.Assembly) || !File.Exists(Path.ChangeExtension(project.Assembly, ".runtimeconfig.json")))
            throw new ContextFailure("missing_test_output", "Build the test assembly and its runtimeconfig.json before --no-build: " + project.Project);
    }
}

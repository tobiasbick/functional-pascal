using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using Fpas.Workspaces.Solutions;
using Microsoft.VisualStudio.SolutionPersistence.Model;
using System.Text.Json;

namespace Fpas.Workspaces.Testing;

public sealed record TestProject(string Project, string Configuration, string Platform, string Framework, string Assembly);
public sealed record DiscoveredTest(string Project, string FullyQualifiedName);

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
                "-getProperty:IsTestProject,IsTestingPlatformApplication,TargetPath,TargetFramework" };
            if (scope.ConfigurationXml is not null)
                args.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + MsBuildSyntax.Escape(scope.Directory!),
                    "-p:CurrentSolutionConfigurationContents=" + MsBuildSyntax.Escape(scope.ConfigurationXml)]);
            var result = await BuildProcess.Run(Path.GetDirectoryName(candidate.Path)!, args, token);
            if (result.ExitCode != 0 || result.Truncated) throw new ContextFailure("evaluation_failed", result.Output + result.Error, result.Truncated);
            using var json = JsonDocument.Parse(result.Output);
            var properties = json.RootElement.GetProperty("Properties");
            if (!properties.GetProperty("IsTestProject").GetString()!.Equals("true", StringComparison.OrdinalIgnoreCase)) continue;
            if (properties.GetProperty("IsTestingPlatformApplication").GetString()!.Equals("true", StringComparison.OrdinalIgnoreCase))
                throw new ContextFailure("unsupported_runner", "This command supports VSTest test projects; Microsoft.Testing.Platform applications require their own runner mode.");
            var target = properties.GetProperty("TargetPath").GetString(); var framework = properties.GetProperty("TargetFramework").GetString();
            if (string.IsNullOrWhiteSpace(target) || string.IsNullOrWhiteSpace(framework))
                throw new ContextFailure("invalid_argument", "Select test projects with one explicit TargetFramework and TargetPath.");
            selected.Add(new(candidate.Path, candidate.Configuration, candidate.Platform, framework, Path.GetFullPath(target, Path.GetDirectoryName(candidate.Path)!)));
        }
        if (selected.Count == 0) throw new ContextFailure("not_test_project", "The selected context contains no enabled test projects.");
        return selected.AsReadOnly();
    }

    public static Task<BuildResult> RunAsync(TestProject project, string directory, string? filter, string? settings, CancellationToken token)
    {
        var args = new List<string> { "vstest", project.Assembly, "/Logger:trx;LogFileName=results.trx", "/ResultsDirectory:" + directory };
        if (filter is not null) args.Add("/TestCaseFilter:" + filter);
        if (settings is not null) args.Add("/Settings:" + settings);
        return BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token);
    }

    public static async Task<(BuildResult Process, IReadOnlyList<DiscoveredTest> Tests)> DiscoverAsync(TestProject project,
        string directory, string? filter, string? settings, CancellationToken token)
    {
        RequireOutput(project);
        var output = Path.Combine(directory, "discovery.txt");
        var args = new List<string> { "vstest", project.Assembly, "/ListFullyQualifiedTests", "/ListTestsTargetPath:" + output };
        if (filter is not null) args.Add("/TestCaseFilter:" + filter);
        if (settings is not null) args.Add("/Settings:" + settings);
        var process = await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token);
        if (process.ExitCode != 0 || process.Truncated) return (process, []);
        if (!File.Exists(output) || new FileInfo(output).Length > 4_194_304)
            throw new ContextFailure("discovery_failed", "Native discovery did not produce a bounded test-name artifact.");
        var names = await File.ReadAllLinesAsync(output, token);
        return (process, Array.AsReadOnly(names.Where(name => name.Length != 0).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal)
            .Select(name => new DiscoveredTest(project.Project, name)).ToArray()));
    }

    public static void RequireOutput(TestProject project)
    {
        if (!File.Exists(project.Assembly) || !File.Exists(Path.ChangeExtension(project.Assembly, ".runtimeconfig.json")))
            throw new ContextFailure("missing_test_output", "Build the test assembly and its runtimeconfig.json before --no-build: " + project.Project);
    }
}

using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using Fpas.Workspaces.Solutions;
using Microsoft.VisualStudio.SolutionPersistence.Model;
using System.Text.Json;
using Fpas.Compiler;

namespace Fpas.Workspaces.Testing;

/// <summary>Describes an evaluated native .NET test project and its output identity.</summary>
/// <param name="Project">Absolute project path.</param>
/// <param name="Configuration">Mapped project configuration.</param>
/// <param name="Platform">Mapped project platform.</param>
/// <param name="Framework">Explicit target framework.</param>
/// <param name="Assembly">Absolute managed test output path.</param>
public sealed record TestProject(string Project, string Configuration, string Platform, string Framework, string Assembly)
{
    /// <summary>Gets whether the output is a self-hosted Microsoft.Testing.Platform application.</summary>
    public bool TestingPlatform { get; init; }
    /// <summary>Gets the evaluated framework moniker for VSTest selection.</summary>
    public string FrameworkMoniker { get; init; } = "";
    /// <summary>Gets evaluated VSTest adapter and collector directories.</summary>
    public IReadOnlyList<string> VstestExtensionPaths { get; init; } = [];
    /// <summary>Gets the captured output-content identity, when inspected.</summary>
    public string? OutputIdentity { get; init; }
    /// <summary>Gets whether portable symbols or source checksums could not be verified.</summary>
    public bool StaleSources { get; init; }
}
/// <summary>Identifies one test discovered by its native runner.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="FullyQualifiedName">Native method identity, or MTP UID when no method identity is available.</param>
public sealed record DiscoveredTest(string Project, string FullyQualifiedName)
{
    /// <summary>Gets the native MTP test UID, when provided.</summary>
    public string? Uid { get; init; }
    /// <summary>Gets the native display name, when provided.</summary>
    public string? DisplayName { get; init; }
    /// <summary>Gets the verified method source location, or null when unavailable or ambiguous.</summary>
    public SourceLocation? Location { get; init; }
}
/// <summary>Supplies native runner extensions and passthrough arguments.</summary>
/// <param name="AdapterPaths">Additional VSTest adapter directories.</param>
/// <param name="Collectors">Native VSTest collector names.</param>
/// <param name="Arguments">VSTest runsettings or MTP extension arguments.</param>
public sealed record TestRunOptions(IReadOnlyList<string> AdapterPaths, IReadOnlyList<string> Collectors, IReadOnlyList<string> Arguments);

/// <summary>Selects native test projects and invokes their VSTest or MTP runner.</summary>
public static class TestDriver
{
    /// <summary>Evaluates included test projects using the context's solution mappings.</summary>
    /// <param name="context">Project or solution path.</param>
    /// <param name="configuration">Context configuration.</param>
    /// <param name="project">Optional included project path to restrict selection.</param>
    /// <param name="token">Cancels evaluation.</param>
    /// <returns>Enabled test projects with explicit frameworks and outputs.</returns>
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

    /// <summary>Runs native tests and requests a results.trx artifact in the supplied directory.</summary>
    /// <param name="project">Evaluated test output to run.</param>
    /// <param name="directory">Invocation-owned result directory.</param>
    /// <param name="filter">Native runner filter, or null.</param>
    /// <param name="settings">Native runsettings path, or null.</param>
    /// <param name="token">Cancels execution and joins owned processes.</param>
    /// <param name="options">Optional native extensions and arguments.</param>
    /// <param name="liveOutput">Optional live output writers.</param>
    /// <returns>Native process status, including captured cancellation as exit 130.</returns>
    public static async Task<BuildResult> RunAsync(TestProject project, string directory, string? filter, string? settings, CancellationToken token,
        TestRunOptions? options = null, ProcessOutput? liveOutput = null)
    {
        if (project.TestingPlatform) return await TestingPlatformDriver.Run(project, directory, filter, settings, options, list: false, token, liveOutput);
        var assembly = await VstestDeployment.Prepare(project, directory, options, token);
        var args = new List<string> { "vstest", assembly, "/Logger:trx;LogFileName=results.trx", "/ResultsDirectory:" + directory };
        if (filter is not null) args.Add("/TestCaseFilter:" + filter);
        if (settings is not null) args.Add("/Settings:" + settings);
        VstestOptions(project, args, options);
        return await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token, captureCancellation: true, liveOutput: liveOutput);
    }

    /// <summary>Discovers native method identities without running test bodies.</summary>
    /// <param name="project">Evaluated test output to inspect.</param>
    /// <param name="directory">Invocation-owned discovery artifact directory.</param>
    /// <param name="filter">Native runner filter, or null.</param>
    /// <param name="settings">Native runsettings path, or null.</param>
    /// <param name="token">Cancels discovery and joins owned processes.</param>
    /// <param name="options">Optional native extensions and arguments.</param>
    /// <returns>Native status and bounded discovered identities.</returns>
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
        var process = await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, args, token, captureCancellation: true);
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

    /// <summary>Requires an existing managed test output and runtime configuration.</summary>
    /// <param name="project">Evaluated test output to validate.</param>
    public static void RequireOutput(TestProject project)
    {
        if (!File.Exists(project.Assembly) || !File.Exists(Path.ChangeExtension(project.Assembly, ".runtimeconfig.json")))
            throw new ContextFailure("missing_test_output", "Build the test assembly and its runtimeconfig.json before --no-build: " + project.Project);
    }
}

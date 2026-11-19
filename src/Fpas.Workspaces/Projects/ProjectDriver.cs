// [Documentation](../../../docs/pascal/program-structure/project-commands.md#build)
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Solutions;
using Microsoft.VisualStudio.SolutionPersistence.Model;
using System.Text.Json;

namespace Fpas.Workspaces.Projects;

/// <summary>Describes an evaluated project output without requiring successful compilation.</summary>
/// <param name="Project">Absolute project path.</param>
/// <param name="Configuration">Mapped project configuration.</param>
/// <param name="Platform">Mapped project platform.</param>
/// <param name="Framework">Explicit target framework.</param>
/// <param name="OutputType">Evaluated output kind.</param>
/// <param name="Assembly">Absolute managed output path.</param>
public sealed record ProjectOutput(string Project, string Configuration, string Platform, string Framework, string OutputType, string Assembly)
{
    /// <summary>Gets whether the output kind is Exe or WinExe.</summary>
    public bool Executable => OutputType.Equals("Exe", StringComparison.OrdinalIgnoreCase) || OutputType.Equals("WinExe", StringComparison.OrdinalIgnoreCase);
}

/// <summary>Builds explicit contexts and selects evaluated executable outputs.</summary>
public static class ProjectDriver
{
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    /// <summary>Requires an existing supported project or solution file.</summary>
    /// <param name="path">FPAS, C#, F# project or slnx solution path.</param>
    /// <returns>The absolute validated context path.</returns>
    public static string ValidateContext(string path)
    {
        path = Path.GetFullPath(path);
        if (!File.Exists(path) || Path.GetExtension(path).ToLowerInvariant() is not (".fpasproj" or ".csproj" or ".fsproj" or ".slnx"))
            throw new ContextFailure("invalid_argument", "Supply an existing .fpasproj, .csproj, .fsproj or .slnx file.");
        return path;
    }

    /// <summary>Builds a context with the selected dotnet SDK and owned process lifetime.</summary>
    /// <param name="context">Project or solution path.</param>
    /// <param name="configuration">Build or solution configuration.</param>
    /// <param name="noRestore">Whether to use existing restore assets.</param>
    /// <param name="token">Cancels the build and joins owned processes.</param>
    /// <param name="captureDiagnostics">Whether to attach structured MSBuild diagnostics.</param>
    /// <param name="liveOutput">Live output writers when structured capture is disabled.</param>
    /// <returns>Build status, captured output and optional structured diagnostics.</returns>
    public static async Task<BuildResult> BuildAsync(string context, string configuration, bool noRestore, CancellationToken token,
        bool captureDiagnostics = false, ProcessOutput? liveOutput = null)
    {
        context = ValidateContext(context);
        var args = new List<string> { "build", context, "--configuration", MsBuildSyntax.Escape(configuration), "--disable-build-servers", "-nr:false", "-p:UseSharedCompilation=false", "--verbosity", "minimal" };
        if (noRestore) args.Add("--no-restore");
        var directory = Path.GetDirectoryName(context)!;
        return captureDiagnostics ? await ProjectBuildDiagnostics.Run(directory, args, token) :
            await BuildProcess.Run(directory, args, token, liveOutput: liveOutput);
    }

    /// <summary>Selects one executable using evaluated outputs and solution mappings.</summary>
    /// <param name="context">Project or solution path.</param>
    /// <param name="configuration">Build or solution configuration.</param>
    /// <param name="project">Optional included project path to disambiguate.</param>
    /// <param name="token">Cancels project evaluation.</param>
    /// <returns>The unique executable output; ambiguous or absent executables are rejected.</returns>
    public static async Task<ProjectOutput> SelectExecutableAsync(string context, string configuration, string? project, CancellationToken token)
    {
        context = ValidateContext(context);
        var scope = await Scope(context, configuration, token);
        var candidates = scope.Projects;
        if (project is not null)
        {
            var selected = Path.GetFullPath(project);
            candidates = candidates.Where(candidate => Paths.Equals(candidate.Path, selected)).ToArray();
            if (candidates.Count != 1) throw new ContextFailure("invalid_argument", "--project must identify an included project in the selected context.");
        }
        var outputs = new List<ProjectOutput>();
        foreach (var candidate in candidates)
        {
            var output = await Describe(candidate, scope, token);
            if (output.Executable) outputs.Add(output);
        }
        if (outputs.Count == 1) return outputs[0];
        throw new ContextFailure(outputs.Count == 0 ? "not_executable" : "ambiguous_project",
            outputs.Count == 0 ? "The selected context contains no executable project." : "Several executable projects are available. Select one with --project.",
            candidates: outputs.Select(output => output.Project).Order(StringComparer.Ordinal).ToArray());
    }

    /// <summary>Reevaluates one included project's output in its context mapping.</summary>
    /// <param name="context">Project or solution path.</param>
    /// <param name="configuration">Context configuration.</param>
    /// <param name="project">Included absolute project path.</param>
    /// <param name="token">Cancels evaluation.</param>
    /// <returns>The current evaluated output description.</returns>
    public static async Task<ProjectOutput> DescribeAsync(string context, string configuration, string project, CancellationToken token)
    {
        var scope = await Scope(context, configuration, token);
        var selected = scope.Projects.SingleOrDefault(candidate => Paths.Equals(candidate.Path, project))
            ?? throw new ContextFailure("invalid_argument", "The selected project is not included in the build context.");
        return await Describe(selected, scope, token);
    }

    private static async Task<SolutionScope> Scope(string context, string configuration, CancellationToken token)
    {
        if (string.IsNullOrWhiteSpace(configuration)) throw new ContextFailure("invalid_argument", "Configuration must not be empty.");
        try { return await SolutionScope.Read(context, configuration, token); }
        catch (SolutionException exception) { throw new ContextFailure("invalid_argument", "Cannot read solution: " + exception.Message); }
    }

    private static async Task<ProjectOutput> Describe(ProjectSelection project, SolutionScope scope, CancellationToken token)
    {
        var args = new List<string> { "msbuild", project.Path, "-nologo", "-nr:false", "-p:UseSharedCompilation=false",
            "-p:Configuration=" + MsBuildSyntax.Escape(project.Configuration), "-p:Platform=" + MsBuildSyntax.Escape(project.Platform),
            "-t:GetTargetPath", "-getProperty:OutputType,TargetPath,TargetFramework" };
        if (scope.ConfigurationXml is not null)
            args.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + MsBuildSyntax.Escape(scope.Directory!),
                "-p:CurrentSolutionConfigurationContents=" + MsBuildSyntax.Escape(scope.ConfigurationXml)]);
        var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, args, token);
        if (result.ExitCode != 0 || result.Truncated) throw new ContextFailure("evaluation_failed", "Cannot evaluate project: " + result.Output + result.Error, result.Truncated);
        using var json = JsonDocument.Parse(result.Output);
        var properties = json.RootElement.GetProperty("Properties");
        var target = properties.GetProperty("TargetPath").GetString();
        var framework = properties.GetProperty("TargetFramework").GetString();
        if (string.IsNullOrWhiteSpace(target) || string.IsNullOrWhiteSpace(framework))
            throw new ContextFailure("invalid_argument", "Select a project with one explicit TargetFramework and TargetPath.");
        return new(project.Path, project.Configuration, project.Platform, framework, properties.GetProperty("OutputType").GetString() ?? "Library",
            Path.GetFullPath(target, Path.GetDirectoryName(project.Path)!));
    }
}

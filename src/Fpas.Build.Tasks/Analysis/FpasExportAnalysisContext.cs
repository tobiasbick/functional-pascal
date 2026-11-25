// [Documentation](../../../docs/architecture/workspaces.md#context-preparation)
using Microsoft.Build.Evaluation;
using Microsoft.Build.Framework;
using System.Security.Cryptography;
using System.Text.Json;
using Fpas.Build.Tasks.Documentation;
using Fpas.Build.Tasks.Incremental;

namespace Fpas.Build.Tasks.Analysis;

/// <summary>Exports the evaluated FPAS compilation inputs as a JSON analysis manifest for workspace loading.</summary>
public sealed class FpasExportAnalysisContext : Microsoft.Build.Utilities.Task
{
    /// <summary>Gets or sets the absolute destination of the new analysis manifest.</summary>
    [Required] public string ManifestFile { get; set; } = "";
    /// <summary>Gets or sets the path of the evaluated project.</summary>
    [Required] public string ProjectPath { get; set; } = "";
    /// <summary>Gets or sets the compiler executable or managed entry assembly.</summary>
    [Required] public string CompilerPath { get; set; } = "";
    /// <summary>Gets or sets the compiler assembly used to verify toolchain identity.</summary>
    public string CompilerAssemblyPath { get; set; } = "";
    /// <summary>Gets or sets the description or resolved-input phase.</summary>
    public string Phase { get; set; } = "description";
    /// <summary>Gets or sets the evaluated configuration.</summary>
    public string Configuration { get; set; } = "Debug";
    /// <summary>Gets or sets the evaluated platform, which must resolve to AnyCPU.</summary>
    public string Platform { get; set; } = "AnyCPU";
    /// <summary>Gets or sets the evaluated target framework, net10.0.</summary>
    public string TargetFramework { get; set; } = "";
    /// <summary>Gets or sets the name of the emitted assembly.</summary>
    public string AssemblyName { get; set; } = "";
    /// <summary>Gets or sets whether the project produces an Exe, WinExe or Library.</summary>
    public string OutputType { get; set; } = "Library";
    /// <summary>Gets or sets the restore assets path required for resolved inputs.</summary>
    public string ProjectAssetsFile { get; set; } = "";
    /// <summary>Gets or sets the destination assembly path used in the build proof.</summary>
    public string OutputAssembly { get; set; } = "";
    /// <summary>Gets or sets the path of the incremental compilation proof.</summary>
    public string BuildCache { get; set; } = "";
    /// <summary>Gets or sets the debug symbol format used in the build proof.</summary>
    public string DebugType { get; set; } = "portable";
    /// <summary>Gets or sets whether compiler optimizations are enabled.</summary>
    public bool Optimize { get; set; }
    /// <summary>Gets or sets the optional assembly signing key path.</summary>
    public string SigningKey { get; set; } = "";
    /// <summary>Gets or sets public signing without a private-key signature.</summary>
    public bool PublicSign { get; set; }
    /// <summary>Gets or sets whether signing reserves a signature for later completion.</summary>
    public bool DelaySign { get; set; }
    /// <summary>Gets or sets the optional XML documentation path.</summary>
    public string DocumentationFile { get; set; } = "";
    /// <summary>Gets or sets the optional documentation publication destination.</summary>
    public string DocumentationDestination { get; set; } = "";
    /// <summary>Gets or sets the optional documentation output copy.</summary>
    public string DocumentationOutput { get; set; } = "";
    /// <summary>Gets or sets whether analysis must validate source documentation.</summary>
    public bool GenerateDocumentationFile { get; set; }
    /// <summary>Gets or sets whether all enabled warnings are errors.</summary>
    public bool TreatWarningsAsErrors { get; set; }
    /// <summary>Gets or sets diagnostic codes promoted to errors.</summary>
    public string WarningsAsErrors { get; set; } = "";
    /// <summary>Gets or sets diagnostic codes exempt from warning promotion.</summary>
    public string WarningsNotAsErrors { get; set; } = "";
    /// <summary>Gets or sets diagnostic codes to suppress.</summary>
    public string NoWarn { get; set; } = "";
    /// <summary>Gets or sets the FPAS sources in the evaluated context.</summary>
    public ITaskItem[] Sources { get; set; } = [];
    /// <summary>Gets or sets the resolved CLR assembly references.</summary>
    public ITaskItem[] References { get; set; } = [];
    /// <summary>Gets or sets neutral resource inputs for compilation proof validation.</summary>
    public ITaskItem[] Resources { get; set; } = [];
    /// <summary>Gets or sets optional native Windows resources for compilation proof validation.</summary>
    public ITaskItem? WindowsResources { get; set; }
    /// <summary>Gets or sets project references with their configuration and platform mappings.</summary>
    public ITaskItem[] Projects { get; set; } = [];
    /// <summary>Gets or sets output assembly paths for referenced projects.</summary>
    public ITaskItem[] ProjectOutputs { get; set; } = [];
    /// <summary>Gets or sets whether checked FPAS sources can supply dependency metadata.</summary>
    public bool UseSourceReferences { get; set; }

    /// <inheritdoc/>
    public override bool Execute()
    {
        try
        {
            var platform = string.IsNullOrWhiteSpace(Platform) ? "AnyCPU" : Platform.Replace(" ", "", StringComparison.Ordinal);
            if (TargetFramework != "net10.0" || platform != "AnyCPU" || OutputType is not ("Exe" or "WinExe" or "Library"))
                throw new InvalidOperationException("Analysis requires net10.0/AnyCPU and OutputType Exe, WinExe or Library.");
            if (!Path.IsPathFullyQualified(ManifestFile)) throw new InvalidOperationException("The analysis manifest path must be absolute.");
            if (Phase == "resolved" && !File.Exists(ProjectAssetsFile))
                return MissingInput(ProjectAssetsFile, "restore_project", "Restore assets are missing.");
            var globals = ((IBuildEngine6)BuildEngine).GetGlobalProperties();
            using var collection = new ProjectCollection(globals.ToDictionary(pair => pair.Key, pair => pair.Value, StringComparer.OrdinalIgnoreCase));
            var evaluated = collection.LoadProject(ProjectPath);
            var projectFiles = evaluated.Imports.Select(import => import.ImportedProject.FullPath).Append(ProjectPath).ToArray();
            CheckPreparationHooks(evaluated);
            foreach (var output in ProjectOutputs)
                if (!File.Exists(output.GetMetadata("FullPath")) && !(UseSourceReferences && SourceOutput(output)))
                    return MissingInput(output.GetMetadata("FullPath"), "build_project", "Referenced project metadata is missing.",
                        output.GetMetadata("MSBuildSourceProjectFile"));
            var sources = Paths(Sources); var references = Phase == "resolved" ? Paths(References) : [];
            string[]? draftSources = null;
            if (Phase == "resolved" && UseSourceReferences && globals.TryGetValue("FpasDraftPaths", out var draftManifest))
            {
                var virtualPaths = (JsonSerializer.Deserialize(File.ReadAllBytes(draftManifest), ExportedContextJson.Default.StringArray) ?? [])
                    .ToHashSet(PathComparer);
                draftSources = DraftFileEvaluation.Sources(evaluated, globals, virtualPaths);
                sources = sources.Where(file => !virtualPaths.Contains(file)).ToArray();
            }
            var sourceReferences = UseSourceReferences ? References.Concat(ProjectOutputs).Where(SourceOutput).ToArray() : [];
            var sourceOutputs = UseSourceReferences ? ProjectOutputs.Where(SourceOutput).Where(output => !References.Any(reference =>
                PathComparer.Equals(reference.GetMetadata("MSBuildSourceProjectFile"), output.GetMetadata("MSBuildSourceProjectFile")))).ToArray() : [];
            if (Phase == "resolved") references = references.Concat(Paths(sourceOutputs)).Distinct(PathComparer).Order(StringComparer.Ordinal).ToArray();
            var inputs = Phase == "resolved" ? projectFiles.Concat(sources).Concat(references) : projectFiles;
            if (Phase == "resolved") inputs = inputs.Append(ProjectAssetsFile);
            var resources = ResourceInput.Read(Resources);
            var windowsResources = WindowsResourceInput.Read(WindowsResources);
            if (Phase == "resolved" && !UseSourceReferences)
            {
                inputs = inputs.Concat(resources.Select(resource => resource.SourcePath)).Concat(resources.Select(resource => resource.Path).Where(File.Exists));
                if (windowsResources is { Icon.Length: > 0 }) inputs = inputs.Append(windowsResources.Icon);
            }
            inputs = inputs.Concat(CompileFingerprint.Tools(CompilerPath));
            if (SigningKey.Length > 0) inputs = inputs.Append(SigningKey);
            var hashes = inputs.Where(path => File.Exists(path) || !sourceReferences.Any(output => PathComparer.Equals(output.GetMetadata("FullPath"), path)))
                .Select(Path.GetFullPath).Distinct(PathComparer).Order(StringComparer.Ordinal)
                .ToDictionary(path => path, Hash, PathComparer);
            var compiler = CompilerAssemblyPath.Length > 0 ? CompilerAssemblyPath : Path.Combine(Path.GetDirectoryName(CompilerPath)!, "Fpas.Compiler.dll");
            hashes[Path.GetFullPath(compiler)] = Hash(compiler);
            var result = new ExportedContext(1, Phase, Path.GetFullPath(ProjectPath), Configuration, platform, TargetFramework,
                AssemblyName, OutputType == "Library" ? "library" : "exe", Hash(compiler), GenerateDocumentationFile, TreatWarningsAsErrors,
                Codes(WarningsAsErrors), Codes(WarningsNotAsErrors), Codes(NoWarn), sources, references,
                Projects.Select(Reference).ToArray(), References.Where(item => item.GetMetadata("ReferenceSourceTarget") == "ProjectReference").Concat(sourceOutputs)
                    .Select(item => new ExportedOutput(item.GetMetadata("MSBuildSourceProjectFile"), item.GetMetadata("FullPath"))).Distinct().ToArray(),
                Path.GetFullPath(BuildCache), Phase == "resolved" && !UseSourceReferences ? BuildInputs(resources, windowsResources) : "", hashes,
                string.IsNullOrWhiteSpace(ProjectAssetsFile) ? null : Path.GetFullPath(ProjectAssetsFile), draftSources);
            using var stream = new FileStream(ManifestFile, FileMode.CreateNew, FileAccess.Write, FileShare.None);
            JsonSerializer.Serialize(stream, result, ExportedContextJson.Default.ExportedContext);
            return true;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or InvalidOperationException or ArgumentException or
            Microsoft.Build.Exceptions.InvalidProjectFileException)
        {
            Log.LogError(null, "FP5005", null, ProjectPath, 0, 0, 0, 0, "Cannot export analysis context: " + exception.Message);
            return false;
        }
    }

    private bool MissingInput(string path, string action, string message, string? owner = null)
    {
        var issue = new ExportedPreparationIssue(string.IsNullOrWhiteSpace(owner) ? ProjectPath : owner, path, action, message);
        using var stream = new FileStream(ManifestFile + ".issue", FileMode.CreateNew, FileAccess.Write, FileShare.None);
        JsonSerializer.Serialize(stream, issue, ExportedContextJson.Default.ExportedPreparationIssue);
        Log.LogError(null, "FP5005", null, ProjectPath, 0, 0, 0, 0, message + " " + path);
        return false;
    }

    private string BuildInputs(IReadOnlyList<ResourceInput> resources, WindowsResourceInput? windowsResources)
    {
        var documents = string.IsNullOrEmpty(DocumentationFile) ? [] : DocumentationOutputs.Paths(DocumentationFile, DocumentationDestination, DocumentationOutput);
        return CompileFingerprint.Compute(Sources.Select(item => item.GetMetadata("FullPath")), References.Select(item => item.GetMetadata("FullPath")),
            CompilerPath, CompileFingerprint.Options(OutputType, Configuration, TargetFramework,
            Path.GetFullPath(OutputAssembly), DebugType, Optimize, documents, TreatWarningsAsErrors, WarningsAsErrors, WarningsNotAsErrors, NoWarn) +
            SigningInputs.Identity(SigningKey, PublicSign, DelaySign), CompilerAssemblyPath, resources, windowsResources);
    }

    private static StringComparer PathComparer => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    private static bool SourceOutput(ITaskItem item) => Path.GetExtension(item.GetMetadata("MSBuildSourceProjectFile")).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase);
    private static string[] Paths(IEnumerable<ITaskItem> items) => items.Select(item => item.GetMetadata("FullPath")).Distinct(PathComparer).Order(StringComparer.Ordinal).ToArray();
    private static string Hash(string path) { using var stream = File.OpenRead(path); return Convert.ToHexString(SHA256.HashData(stream)); }
    private static string[] Codes(string text) => text.Split([';', ','], StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
    private static string Setting(ITaskItem item, string metadata, string fallback)
    {
        var text = item.GetMetadata(metadata); var separator = text.IndexOf('=');
        return separator < 0 ? fallback : text[(separator + 1)..];
    }

    private ExportedReference Reference(ITaskItem project)
    {
        if (!string.IsNullOrWhiteSpace(project.GetMetadata("AdditionalProperties")))
            throw new InvalidOperationException("Analysis does not support ProjectReference AdditionalProperties. Use configuration/platform mappings instead: " + project.ItemSpec);
        var removed = project.GetMetadata("GlobalPropertiesToRemove").Split(';', StringSplitOptions.TrimEntries);
        if (removed.Any(property => new[] { "BuildingSolutionFile", "SolutionDir", "CurrentSolutionConfigurationContents" }.Contains(property, StringComparer.OrdinalIgnoreCase)))
            throw new InvalidOperationException("Analysis does not support removing solution context properties from a ProjectReference: " + project.ItemSpec);
        return new(project.GetMetadata("FullPath"), removed.Contains("Configuration", StringComparer.OrdinalIgnoreCase) ? "" : Setting(project, "SetConfiguration", Configuration),
            removed.Contains("Platform", StringComparer.OrdinalIgnoreCase) ? "" : Setting(project, "SetPlatform", Platform));
    }

    private static void CheckPreparationHooks(Project project)
    {
        var sdk = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(typeof(Project).Assembly.Location)!, "..", "..")) + Path.DirectorySeparatorChar;
        var preparation = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        void Include(string name)
        {
            if (!preparation.Add(name) || !project.Targets.TryGetValue(name, out var target)) return;
            foreach (var dependency in project.ExpandString(target.DependsOnTargets).Split(';', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries))
                Include(dependency);
        }
        Include("FpasPrepareCompilation");
        string[] compileTargets = ["CoreCompile", "BeforeCompile", "Compile", "Build", "BeforeBuild", "CoreBuild"];
        foreach (var target in project.Targets.Values)
        {
            if (preparation.Contains(target.Name)) continue;
            if (target.FullPath.StartsWith(sdk, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal)) continue;
            if (project.ExpandString(target.BeforeTargets).Split(';', StringSplitOptions.TrimEntries)
                .Any(name => compileTargets.Contains(name, StringComparer.OrdinalIgnoreCase)))
                throw new InvalidOperationException("Custom pre-compile hooks must use FpasPrepareCompilation so check and build prepare the same inputs: " + target.Name);
        }
    }
}

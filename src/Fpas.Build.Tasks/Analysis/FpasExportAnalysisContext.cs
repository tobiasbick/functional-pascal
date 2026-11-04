using Microsoft.Build.Evaluation;
using Microsoft.Build.Framework;
using System.Security.Cryptography;
using System.Text.Json;
using Fpas.Build.Tasks.Documentation;
using Fpas.Build.Tasks.Incremental;

namespace Fpas.Build.Tasks.Analysis;

public sealed class FpasExportAnalysisContext : Microsoft.Build.Utilities.Task
{
    [Required] public string ManifestFile { get; set; } = "";
    [Required] public string ProjectPath { get; set; } = "";
    [Required] public string CompilerPath { get; set; } = "";
    public string Phase { get; set; } = "description";
    public string Configuration { get; set; } = "Debug";
    public string Platform { get; set; } = "AnyCPU";
    public string TargetFramework { get; set; } = "";
    public string AssemblyName { get; set; } = "";
    public string OutputType { get; set; } = "Library";
    public string ProjectAssetsFile { get; set; } = "";
    public string OutputAssembly { get; set; } = "";
    public string BuildCache { get; set; } = "";
    public string DebugType { get; set; } = "portable";
    public bool Optimize { get; set; }
    public string DocumentationFile { get; set; } = "";
    public string DocumentationDestination { get; set; } = "";
    public string DocumentationOutput { get; set; } = "";
    public bool GenerateDocumentationFile { get; set; }
    public bool TreatWarningsAsErrors { get; set; }
    public string WarningsAsErrors { get; set; } = "";
    public string WarningsNotAsErrors { get; set; } = "";
    public string NoWarn { get; set; } = "";
    public ITaskItem[] Sources { get; set; } = [];
    public ITaskItem[] References { get; set; } = [];
    public ITaskItem[] Projects { get; set; } = [];

    public override bool Execute()
    {
        try
        {
            var platform = string.IsNullOrWhiteSpace(Platform) ? "AnyCPU" : Platform.Replace(" ", "", StringComparison.Ordinal);
            if (TargetFramework != "net10.0" || platform != "AnyCPU" || OutputType is not ("Exe" or "Library"))
                throw new InvalidOperationException("Analysis requires net10.0/AnyCPU and OutputType Exe or Library.");
            if (!Path.IsPathFullyQualified(ManifestFile)) throw new InvalidOperationException("The analysis manifest path must be absolute.");
            var globals = ((IBuildEngine6)BuildEngine).GetGlobalProperties();
            using var collection = new ProjectCollection(globals.ToDictionary(pair => pair.Key, pair => pair.Value, StringComparer.OrdinalIgnoreCase));
            var evaluated = collection.LoadProject(ProjectPath);
            var projectFiles = evaluated.Imports.Select(import => import.ImportedProject.FullPath).Append(ProjectPath).ToArray();
            CheckPreparationHooks(evaluated);
            var sources = Paths(Sources); var references = Phase == "resolved" ? Paths(References) : [];
            var inputs = Phase == "resolved" ? projectFiles.Concat(sources).Concat(references) : projectFiles;
            if (Phase == "resolved" && File.Exists(ProjectAssetsFile)) inputs = inputs.Append(ProjectAssetsFile);
            inputs = inputs.Concat(CompileFingerprint.Tools(CompilerPath));
            var hashes = inputs.Select(Path.GetFullPath).Distinct(PathComparer).Order(StringComparer.Ordinal)
                .ToDictionary(path => path, Hash, PathComparer);
            var compiler = Path.Combine(Path.GetDirectoryName(CompilerPath)!, "Fpas.Compiler.dll");
            var result = new ExportedContext(1, Phase, Path.GetFullPath(ProjectPath), Configuration, platform, TargetFramework,
                AssemblyName, OutputType == "Exe" ? "exe" : "library", Hash(compiler), GenerateDocumentationFile, TreatWarningsAsErrors,
                Codes(WarningsAsErrors), Codes(WarningsNotAsErrors), Codes(NoWarn), sources, references,
                Projects.Select(Reference).ToArray(), References.Where(item => item.GetMetadata("ReferenceSourceTarget") == "ProjectReference")
                    .Select(item => new ExportedOutput(item.GetMetadata("MSBuildSourceProjectFile"), item.GetMetadata("FullPath"))).ToArray(),
                Path.GetFullPath(BuildCache), Phase == "resolved" ? BuildInputs(sources, references) : "", hashes);
            using var stream = new FileStream(ManifestFile, FileMode.CreateNew, FileAccess.Write, FileShare.None);
            JsonSerializer.Serialize(stream, result, ExportedContextJson.Default.ExportedContext);
            return true;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or InvalidOperationException or ArgumentException)
        {
            Log.LogError(null, "FP5005", null, ProjectPath, 0, 0, 0, 0, "Cannot export analysis context: " + exception.Message);
            return false;
        }
    }

    private string BuildInputs(string[] sources, string[] references)
    {
        var documents = string.IsNullOrEmpty(DocumentationFile) ? [] : DocumentationOutputs.Paths(DocumentationFile, DocumentationDestination, DocumentationOutput);
        return CompileFingerprint.Compute(sources, references, CompilerPath, CompileFingerprint.Options(OutputType, Configuration, TargetFramework,
            Path.GetFullPath(OutputAssembly), DebugType, Optimize, documents, TreatWarningsAsErrors, WarningsAsErrors, WarningsNotAsErrors, NoWarn));
    }

    private static StringComparer PathComparer => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
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
        foreach (var target in project.Targets.Values)
        {
            if (target.FullPath.StartsWith(sdk, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal)) continue;
            if (target.BeforeTargets.Split(';').Any(name => name.Trim() is "CoreCompile" or "BeforeCompile" or "Compile" or "Build" or "BeforeBuild" or "CoreBuild"))
                throw new InvalidOperationException("Custom pre-compile hooks must use FpasPrepareCompilation so check and build prepare the same inputs: " + target.Name);
        }
    }
}

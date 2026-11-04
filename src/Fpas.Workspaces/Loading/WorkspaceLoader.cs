using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Solutions;
using System.Text;
using System.Text.Json;

namespace Fpas.Workspaces.Loading;

public static class WorkspaceLoader
{
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    public static async Task<WorkspaceContext> LoadAsync(ContextOptions options, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var context = Path.GetFullPath(options.Context);
        if (!File.Exists(context) || Path.GetExtension(context).ToLowerInvariant() is not (".fpasproj" or ".slnx"))
            throw new ContextFailure("invalid_argument", "Supply an existing .fpasproj or .slnx context.");
        if (string.IsNullOrWhiteSpace(options.Configuration)) throw new ContextFailure("invalid_argument", "Configuration must not be empty.");
        SolutionScope scope;
        try { scope = await SolutionScope.Read(context, options.Configuration, cancellationToken); }
        catch (Microsoft.VisualStudio.SolutionPersistence.Model.SolutionException exception)
        { throw new ContextFailure("context_failed", "Cannot read the solution: " + exception.Message); }
        var scratch = Path.Combine(Path.GetTempPath(), "fpas-analysis-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(scratch);
        try { return await Load(options, context, scope, scratch, cancellationToken); }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or JsonException or DecoderFallbackException or System.ComponentModel.Win32Exception)
        { throw new ContextFailure("context_failed", "Cannot read the selected context: " + exception.Message); }
        finally { Directory.Delete(scratch, recursive: true); }
    }

    private static async Task<WorkspaceContext> Load(ContextOptions options, string context, SolutionScope scope, string scratch, CancellationToken token)
    {
        var hashes = new Dictionary<string, string>(Paths) { [context] = WorkspaceContext.HashFile(context) };
        var projects = new List<ContextProject>(); var issues = new List<ContextIssue>();
        var outputs = new List<PreparationOutput>();
        var edges = new Dictionary<string, string[]>(Paths);
        if (!options.NoRestore)
        {
            var restore = await BuildProcess.Run(Path.GetDirectoryName(context)!, ["restore", context, "--disable-parallel", "--verbosity", "quiet",
                "-nr:false", "-p:Configuration=" + Escape(options.Configuration), "-p:UseSharedCompilation=false"], token);
            if (restore.ExitCode != 0 || restore.Truncated) throw new ContextFailure("context_failed", "Restore failed or exceeded its output limit: " + restore.Output + restore.Error, restore.Truncated);
        }
        var described = new Dictionary<string, (ProjectSelection Selection, ProjectManifest Manifest)>(Paths);
        var resolved = new Dictionary<string, ProjectManifest>(Paths);
        var visited = new Dictionary<string, string>(Paths);
        var queue = new Queue<ProjectSelection>(scope.Projects);
        while (queue.TryDequeue(out var pending))
        {
            token.ThrowIfCancellationRequested();
            ProjectSelection selection;
            try { selection = await ResolveSelection(pending, scope, token); }
            catch (ContextFailure failure) { visited.TryAdd(pending.Path, pending.Configuration); issues.Add(new(pending.Path, failure.Code, failure.Message, failure.OutputTruncated)); continue; }
            if (visited.TryGetValue(selection.Path, out var configuration))
            {
                if (configuration != selection.Configuration) throw new ContextFailure("ambiguous_project", "A project is required in multiple configurations: " + selection.Path);
                continue;
            }
            visited.Add(selection.Path, selection.Configuration);
            try
            {
                if (Path.GetExtension(selection.Path).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase))
                {
                    var manifest = await Export(selection, scope, scratch, "description", token);
                    described.Add(selection.Path, (selection, manifest));
                    edges[selection.Path] = manifest.Projects.Select(reference => reference.Path).ToArray();
                    foreach (var reference in manifest.Projects)
                        queue.Enqueue(new(reference.Path, reference.Configuration, reference.Platform));
                }
                else
                {
                    var references = await ForeignReferences(selection, scope, token);
                    edges[selection.Path] = references.Select(reference => reference.Path).ToArray();
                    foreach (var reference in references)
                        queue.Enqueue(reference);
                }
            }
            catch (ContextFailure failure) { issues.Add(new(selection.Path, failure.Code, failure.Message, failure.OutputTruncated)); }
        }
        foreach (var (_, (selection, _)) in described.OrderBy(item => item.Key, StringComparer.Ordinal))
        {
            try
            {
                var manifest = await Export(selection, scope, scratch, "resolved", token);
                resolved.Add(selection.Path, manifest);
                outputs.AddRange(manifest.Outputs.Select(output => new PreparationOutput(output.Project, output.Path)));
                var compiler = typeof(FpasCompiler).Assembly.Location;
                hashes[compiler] = WorkspaceContext.HashFile(compiler);
                var sources = new List<CompilationSource>();
                foreach (var (path, hash) in manifest.InputHashes)
                {
                    if (hashes.TryGetValue(path, out var previous) && hash != previous) throw new ContextFailure("inputs_changed", "Inputs differ between project contexts: " + path);
                    hashes[path] = hash;
                }
                foreach (var file in manifest.Sources)
                {
                    token.ThrowIfCancellationRequested();
                    var content = await File.ReadAllBytesAsync(file, token);
                    if (Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(content)) != manifest.InputHashes[file])
                        throw new ContextFailure("inputs_changed", "A source changed during context capture: " + file);
                    sources.Add(new(file, new UTF8Encoding(false, true).GetString(content)));
                }
                var id = WorkspaceContext.HashText(selection.Path + "\n" + manifest.Configuration);
                var policy = new DiagnosticOptions(manifest.WarningsAsErrors, Array.AsReadOnly(manifest.WarningErrors),
                    Array.AsReadOnly(manifest.WarningExceptions), Array.AsReadOnly(manifest.NoWarn));
                var request = new AnalysisRequest(sources.AsReadOnly(), manifest.AssemblyName, Array.AsReadOnly(manifest.References),
                    manifest.Target == "exe" ? CompilationTarget.Exe : CompilationTarget.Library, manifest.ValidateDocumentation, policy);
                projects.Add(new(selection.Path, manifest.Configuration, id, request, Reachable(selection.Path)));
            }
            catch (ContextFailure failure) { issues.Add(new(selection.Path, failure.Code, failure.Message, failure.OutputTruncated)); }
        }
        if (described.Count == 0 && issues.Count == 0) throw new ContextFailure("context_failed", "The selected context contains no FPAS compilation.");
        foreach (var output in outputs)
        {
            token.ThrowIfCancellationRequested();
            if (!resolved.TryGetValue(output.Project, out var owner)) continue;
            if (!File.Exists(owner.BuildCache) || new FileInfo(owner.BuildCache).Length > 16_777_216)
                throw new ContextFailure("inputs_changed", "Prepared FPAS reference has no valid build proof: " + output.Project);
            var proof = JsonSerializer.Deserialize(await File.ReadAllBytesAsync(owner.BuildCache, token), ProjectManifestJson.Default.BuildProof);
            if (proof is null || proof.Inputs != owner.ExpectedBuildInputs || proof.AssemblyHash != WorkspaceContext.HashFile(output.Path))
                throw new ContextFailure("inputs_changed", "Prepared reference does not match the checked dependency inputs/output: " + output.Project);
            hashes[owner.BuildCache] = WorkspaceContext.HashFile(owner.BuildCache);
        }
        var result = new WorkspaceContext(context, options.Configuration, !options.NoRestore, projects, issues,
            visited.Keys.Where(path => Path.GetExtension(path).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase)), outputs, hashes);
        result.VerifyFiles(token);
        return result;

        IReadOnlyList<string> Reachable(string project)
        {
            var seen = new HashSet<string>(Paths); var pending = new Stack<string>(edges.GetValueOrDefault(project) ?? []);
            while (pending.TryPop(out var next))
                if (seen.Add(next)) foreach (var dependency in edges.GetValueOrDefault(next) ?? []) pending.Push(dependency);
            return Array.AsReadOnly(seen.Order(StringComparer.Ordinal).ToArray());
        }
    }

    private static List<string> Arguments(ProjectSelection project, SolutionScope scope)
    {
        var args = new List<string> { "msbuild", project.Path, "-nologo", "-nr:false", "-m:1", "-verbosity:quiet", "-p:UseSharedCompilation=false" };
        if (project.Configuration.Length != 0) args.Add("-p:Configuration=" + Escape(project.Configuration));
        if (project.Platform.Length != 0) args.Add("-p:Platform=" + Escape(project.Platform));
        if (scope.ConfigurationXml is not null)
            args.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + Escape(scope.Directory!), "-p:CurrentSolutionConfigurationContents=" + Escape(scope.ConfigurationXml)]);
        return args;
    }

    private static async Task<ProjectManifest> Export(ProjectSelection project, SolutionScope scope, string scratch, string phase, CancellationToken token)
    {
        var path = Path.Combine(scratch, Guid.NewGuid().ToString("N") + ".json");
        var args = Arguments(project, scope);
        args.AddRange(["-t:" + (phase == "description" ? "FpasDescribeAnalysisContext" : "FpasExportAnalysisContext"), "-p:FpasAnalysisManifest=" + Escape(path)]);
        var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, args, token);
        if (result.ExitCode != 0 || !File.Exists(path) || result.Truncated)
            throw new ContextFailure(phase == "resolved" ? "dependency_failed" : "context_failed", "MSBuild context export failed. Use a matching FPAS SDK with analysis targets. " + result.Output + result.Error, result.Truncated);
        if (new FileInfo(path).Length > 16_777_216) throw new ContextFailure("context_failed", "Context manifest exceeds 16 MiB.");
        var manifest = JsonSerializer.Deserialize(await File.ReadAllBytesAsync(path, token), ProjectManifestJson.Default.ProjectManifest)
            ?? throw new ContextFailure("context_failed", "Missing context manifest.");
        if (manifest.Version != 1 || manifest.Phase != phase || !Paths.Equals(manifest.Project, project.Path) || manifest.Framework != "net10.0" || manifest.Target is not ("exe" or "library") ||
            string.IsNullOrEmpty(manifest.AssemblyName) || manifest.Configuration != project.Configuration || manifest.Platform != project.Platform.Replace(" ", "", StringComparison.Ordinal) ||
            manifest.Sources is null || manifest.References is null || manifest.Projects is null || manifest.Outputs is null || manifest.InputHashes is null ||
            manifest.WarningErrors is null || manifest.WarningExceptions is null || manifest.NoWarn is null)
            throw new ContextFailure("context_failed", "Invalid or incompatible context manifest.");
        if (string.IsNullOrEmpty(manifest.BuildCache) || !Path.IsPathFullyQualified(manifest.BuildCache) || phase == "resolved" && (manifest.ExpectedBuildInputs is null || manifest.ExpectedBuildInputs.Length != 64))
            throw new ContextFailure("context_failed", "Invalid build proof in context manifest.");
        if (manifest.Projects.Any(reference => reference is null || !Path.IsPathFullyQualified(reference.Path) || reference.Configuration is null || reference.Platform is null) ||
            manifest.Outputs.Any(output => output is null || !Path.IsPathFullyQualified(output.Path)) ||
            manifest.InputHashes.Values.Any(hash => hash is null || hash.Length != 64 || !hash.All(char.IsAsciiHexDigit)))
            throw new ContextFailure("context_failed", "Invalid context identities or content fingerprints.");
        var currentCompiler = WorkspaceContext.HashFile(typeof(FpasCompiler).Assembly.Location);
        if (manifest.CompilerHash != currentCompiler) throw new ContextFailure("context_failed", "Selected SDK compiler differs from this CLI; rebuild/repack the matching SDK.");
        foreach (var file in (phase == "resolved" ? manifest.Sources.Concat(manifest.References) : []).Concat(manifest.InputHashes.Keys))
            if (!Path.IsPathFullyQualified(file) || !File.Exists(file)) throw new ContextFailure("context_failed", "Missing absolute context input: " + file);
        foreach (var file in phase == "resolved" ? manifest.Sources.Concat(manifest.References) : [])
            if (!manifest.InputHashes.ContainsKey(file)) throw new ContextFailure("context_failed", "Context input has no content fingerprint: " + file);
        return manifest;
    }

    private static async Task<IReadOnlyList<ProjectSelection>> ForeignReferences(ProjectSelection project, SolutionScope scope, CancellationToken token)
    {
        var args = Arguments(project, scope);
        args.AddRange(["-t:AssignProjectConfiguration", "-getItem:ProjectReferenceWithConfiguration"]);
        var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, args, token);
        if (result.ExitCode != 0 || result.Truncated) throw new ContextFailure("context_failed", "Cannot evaluate foreign project references: " + result.Output + result.Error, result.Truncated);
        using var json = JsonDocument.Parse(result.Output);
        return json.RootElement.GetProperty("Items").GetProperty("ProjectReferenceWithConfiguration").EnumerateArray().Select(item =>
        {
            if (item.TryGetProperty("AdditionalProperties", out var additional) && !string.IsNullOrWhiteSpace(additional.GetString()))
                throw new ContextFailure("context_failed", "Analysis does not support ProjectReference AdditionalProperties. Use configuration/platform mappings instead.");
            var removed = item.TryGetProperty("GlobalPropertiesToRemove", out var removal) ? (removal.GetString() ?? "").Split(';', StringSplitOptions.TrimEntries) : [];
            if (removed.Any(property => new[] { "BuildingSolutionFile", "SolutionDir", "CurrentSolutionConfigurationContents" }.Contains(property, StringComparer.OrdinalIgnoreCase)))
                throw new ContextFailure("context_failed", "Analysis does not support removing solution context properties from a ProjectReference.");
            string Setting(string name, string fallback)
            {
                var value = item.TryGetProperty(name, out var element) ? element.GetString() ?? "" : "";
                var separator = value.IndexOf('='); return separator < 0 ? fallback : value[(separator + 1)..];
            }
            return new ProjectSelection(item.GetProperty("FullPath").GetString()!, removed.Contains("Configuration", StringComparer.OrdinalIgnoreCase) ? "" : Setting("SetConfiguration", project.Configuration),
                removed.Contains("Platform", StringComparer.OrdinalIgnoreCase) ? "" : Setting("SetPlatform", project.Platform));
        }).ToArray();
    }

    private static async Task<ProjectSelection> ResolveSelection(ProjectSelection project, SolutionScope scope, CancellationToken token)
    {
        if (project.Configuration.Length != 0 && project.Platform.Length != 0) return project;
        var args = Arguments(project, scope); args.Add("-getProperty:Configuration,Platform");
        var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, args, token);
        if (result.ExitCode != 0 || result.Truncated) throw new ContextFailure("context_failed", "Cannot evaluate reference configuration: " + result.Output + result.Error, result.Truncated);
        using var json = JsonDocument.Parse(result.Output);
        var properties = json.RootElement.GetProperty("Properties");
        var configuration = properties.GetProperty("Configuration").GetString();
        var platform = properties.GetProperty("Platform").GetString();
        if (string.IsNullOrEmpty(configuration)) throw new ContextFailure("context_failed", "Reference evaluation returned no configuration: " + project.Path);
        return project with { Configuration = configuration, Platform = string.IsNullOrEmpty(platform) ? "AnyCPU" : platform };
    }

    private static string Escape(string text)
    {
        var result = new StringBuilder();
        foreach (var c in text) result.Append("%*?@$();,'\"".Contains(c) ? "%" + ((int)c).ToString("X2", System.Globalization.CultureInfo.InvariantCulture) : c);
        return result.ToString();
    }
}

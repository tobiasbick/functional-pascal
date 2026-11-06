using Fpas.Workspaces.Processes;
using System.Xml.Linq;

namespace Fpas.Workspaces.Loading;

/// <summary>Inspect evaluated imports without executing targets before allowing the fixed analysis target graph.</summary>
internal static class PassiveEvaluation
{
    internal static async Task Check(string project, List<string> arguments, string scratch, IReadOnlyDictionary<string, string> environment, CancellationToken token)
    {
        var expanded = Path.Combine(scratch, Guid.NewGuid().ToString("N") + ".xml");
        var result = await BuildProcess.Run(Path.GetDirectoryName(project)!, [.. arguments, "-preprocess:" + expanded], token, environment: environment);
        if (result.ExitCode != 0 || result.Truncated || !File.Exists(expanded))
            throw new ContextFailure("preparation_required", "Cannot evaluate existing project inputs. Check project XML and restore/build missing SDK assets explicitly. " + result.Output + result.Error, result.Truncated);
        if (new FileInfo(expanded).Length > 16_777_216) throw new ContextFailure("context_failed", "Expanded project exceeds 16 MiB.");
        var xml = XDocument.Load(expanded);
        var imports = xml.DescendantNodes().OfType<XComment>().SelectMany(comment => comment.Value.Split('\n')).Select(line => line.Trim())
            .Where(path => Path.IsPathFullyQualified(path) && File.Exists(path)).Append(project).Distinct(WorkspacePaths.Comparer).ToArray();
        var sdkProps = imports.FirstOrDefault(path => path.Replace('\\', '/').EndsWith("/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.props", StringComparison.Ordinal));
        if (sdkProps is null) throw new ContextFailure("preparation_required", "Passive evaluation requires the standard .NET SDK targets.");
        var sdkRoot = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(sdkProps)!, "..", "..", "..")) + Path.DirectorySeparatorChar;
        var installation = Path.GetFullPath(Path.Combine(sdkRoot, "..", ".."));
        var package = Path.Combine(environment["FPAS_INSTALLED_SDK_CACHE"], "fpas.net.sdk") + Path.DirectorySeparatorChar;
        var trusted = new[] { sdkRoot, Path.Combine(installation, "sdk-manifests") + Path.DirectorySeparatorChar,
            Path.Combine(installation, "packs") + Path.DirectorySeparatorChar, package };
        bool Trusted(string path) => trusted.Any(root => path.StartsWith(root, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal));
        var targets = imports.Select(path => (Path: path, Project: XDocument.Load(path).Root!)).ToArray();
        var standardNames = targets.Where(input => Trusted(input.Path)).SelectMany(input => input.Project.Elements().Where(element => element.Name.LocalName == "Target"))
            .Select(element => (string?)element.Attribute("Name") ?? "").ToHashSet(StringComparer.OrdinalIgnoreCase);
        string[] buildHooks = ["FpasPrepareCompilation", "Build", "BeforeBuild", "AfterBuild", "CoreBuild", "Compile", "BeforeCompile", "AfterCompile", "CoreCompile", "Clean", "CoreClean", "AfterClean", "BeforeClean", "Rebuild"];
        foreach (var (path, root) in targets.Where(input => !Trusted(input.Path)))
        {
            token.ThrowIfCancellationRequested();
            if (!string.IsNullOrWhiteSpace((string?)root.Attribute("InitialTargets"))) Reject(path, "custom InitialTargets");
            foreach (var property in root.Descendants().Where(element => element.Parent?.Name.LocalName == "PropertyGroup" && !element.Ancestors().Any(parent => parent.Name.LocalName == "Target")))
                if (property.Name.LocalName != "FpasPrepareCompilationDependsOn" &&
                    (property.Name.LocalName.EndsWith("DependsOn", StringComparison.OrdinalIgnoreCase) || property.Name.LocalName == "_FpasAnalysisInputs"))
                    Reject(path, "custom target dependencies: " + property.Name.LocalName);
            foreach (var element in root.Elements())
            {
                if (element.Name.LocalName != "Target") continue;
                var name = (string?)element.Attribute("Name") ?? "";
                if (standardNames.Contains(name) || name.Contains('$')) Reject(path, "overridden/dynamic SDK target: " + name);
                foreach (var attribute in new[] { "BeforeTargets", "AfterTargets" })
                    foreach (var hook in ((string?)element.Attribute(attribute) ?? "").Split(';', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries))
                        if (!buildHooks.Contains(hook, StringComparer.OrdinalIgnoreCase)) Reject(path, "custom analysis hook: " + name);
                if (element.Descendants().Any(item => item.Name.LocalName == "Compile" || item.Attribute("ItemName") is { } itemName && (itemName.Value == "Compile" || itemName.Value.Contains('$'))))
                    Reject(path, "target-added Compile items; declare generated sources in evaluated ItemGroups");
            }
        }
    }

    private static void Reject(string path, string reason) => throw new ContextFailure("preparation_required", "Passive evaluation does not support " + reason + ": " + path);
}

internal static class WorkspacePaths
{
    internal static StringComparer Comparer => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
}

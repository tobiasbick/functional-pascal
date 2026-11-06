using Fpas.Workspaces.Processes;
using System.Xml.Linq;
using static Fpas.Workspaces.Processes.MsBuildSyntax;

namespace Fpas.Workspaces.Loading;

internal static class WorkspacePreparation
{
    internal static async Task Restore(ContextOptions options, string context, CancellationToken token)
    {
        if (options.NoRestore || options.Preparation != ContextPreparation.BuildDependencies) return;
        var restore = await BuildProcess.Run(Path.GetDirectoryName(context)!, ["restore", context, "--disable-parallel", "--verbosity", "quiet",
            "-nr:false", "-p:Configuration=" + Escape(options.Configuration), "-p:UseSharedCompilation=false"], token);
        if (restore.ExitCode != 0 || restore.Truncated)
            throw new ContextFailure("context_failed", "Restore failed or exceeded its output limit: " + restore.Output + restore.Error, restore.Truncated);
    }

    internal static void ConfigureEvaluation(List<string> arguments, ContextPreparation preparation)
    {
        if (preparation != ContextPreparation.BuildDependencies)
            arguments.AddRange(["-p:FpasAnalysisPrepare=false", "-p:BuildProjectReferences=false", "-p:DesignTimeBuild=true"]);
        if (preparation == ContextPreparation.SourceSnapshots) arguments.Add("-p:FpasAnalysisSourceReferences=true");
    }

    internal static async Task<IReadOnlyDictionary<string, string>> InstalledSdkEnvironment(string context, string scratch, CancellationToken token)
    {
        var result = await BuildProcess.Run(Path.GetDirectoryName(context)!, ["nuget", "locals", "global-packages", "--list", "--force-english-output"], token);
        const string prefix = "global-packages: ";
        var line = result.Output.Split('\n').Select(line => line.Trim()).FirstOrDefault(line => line.StartsWith(prefix, StringComparison.Ordinal));
        if (result.ExitCode != 0 || result.Truncated || line is null) throw new ContextFailure("preparation_required", "Cannot locate installed package SDKs.");
        var root = Path.Combine(scratch, "resolvers"); var folder = Path.Combine(root, "Fpas.SdkResolver"); Directory.CreateDirectory(folder);
        var assembly = Path.Combine(Path.GetDirectoryName(typeof(WorkspaceLoader).Assembly.Location)!, "Fpas.SdkResolver.dll");
        if (!File.Exists(assembly)) throw new ContextFailure("preparation_required", "The installed SDK resolver is missing from this compiler distribution.");
        File.WriteAllText(Path.Combine(folder, "Fpas.SdkResolver.xml"), new XElement("SdkResolver", new XElement("Path", assembly)).ToString());
        return new Dictionary<string, string> { ["MSBUILDDISABLENUGETSDKRESOLVER"] = "1", ["MSBUILDADDITIONALSDKRESOLVERSFOLDER"] = root,
            ["FPAS_INSTALLED_SDK_CACHE"] = line[prefix.Length..] };
    }
}

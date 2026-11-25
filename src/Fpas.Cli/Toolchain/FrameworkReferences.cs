// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.Deployment;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;

namespace Fpas.Cli.Toolchain;

internal static class FrameworkReferences
{
    internal static string CorePath => DistributionAssets.Managed(typeof(Core.RecordAttribute).Assembly);

    internal static string[] Resolve()
    {
        var host = Path.GetFullPath(BuildProcess.DotnetHost());
        var installation = Path.GetDirectoryName(File.ResolveLinkTarget(host, returnFinalTarget: true)?.FullName ?? host)!;
        var packs = Path.Combine(installation, "packs", "Microsoft.NETCore.App.Ref");
        var referenceDirectory = Directory.Exists(packs) ? Directory.EnumerateDirectories(packs)
            .Select(path => (Path: path, Version: Version.TryParse(Path.GetFileName(path), out var version) ? version : null))
            .Where(pack => pack.Version is { Major: 10, Minor: 0, Build: >= 0, Revision: -1 })
            .OrderByDescending(pack => pack.Version)
            .Select(pack => Path.Combine(pack.Path, "ref", "net10.0"))
            .FirstOrDefault(path => File.Exists(Path.Combine(path, "System.Runtime.dll"))) : null;
        if (referenceDirectory is null)
            throw new ContextFailure("reference_pack_missing",
                "The selected dotnet installation has no .NET 10 reference pack. Install the .NET 10 SDK or point DOTNET_HOST_PATH to an installation with that pack.",
                inputPath: packs);
        return [.. Directory.EnumerateFiles(referenceDirectory, "*.dll").Order(StringComparer.Ordinal), CorePath];
    }
}

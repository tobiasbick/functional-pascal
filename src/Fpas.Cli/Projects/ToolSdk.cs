using System.Reflection;
using System.Xml.Linq;

namespace Fpas.Cli.Projects;

internal static class ToolSdk
{
    internal static string Version
    {
        get
        {
            var directory = Directory.GetParent(AppContext.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar));
            if (directory?.Name == "tools" && directory.Parent is { } package)
            {
                var manifests = Directory.EnumerateFiles(package.FullName, "*.nuspec")
                    .Where(path => Path.GetFileName(path).Equals("fpas.net.sdk.nuspec", StringComparison.OrdinalIgnoreCase)).Take(2).ToArray();
                if (manifests.Length > 1) throw new IOException("The FPAS SDK contains ambiguous package manifests.");
                if (manifests.Length == 1)
                {
                    if (new FileInfo(manifests[0]).Length > 1_048_576) throw new IOException("The FPAS SDK manifest exceeds 1 MiB.");
                    var manifest = XDocument.Load(manifests[0]);
                    var versions = manifest.Descendants().Where(element => element.Name.LocalName == "version").Select(element => element.Value.Trim()).Take(2).ToArray();
                    if (versions.Length != 1 || versions[0].Length == 0) throw new IOException("The FPAS SDK manifest must contain one package version.");
                    return ValidateVersion(versions[0]);
                }
            }
            return typeof(ToolSdk).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>().Single(attribute => attribute.Key == "FpasSdkVersion").Value!;
        }
    }

    internal static string ValidateVersion(string? value)
    {
        var version = value ?? Version;
        if (version.Length == 0 || !char.IsAsciiDigit(version[0]) || version.Any(character => !char.IsAsciiLetterOrDigit(character) && character is not ('.' or '-' or '+')))
            throw new ArgumentException("--sdk-version must be an explicit NuGet version, without ranges or MSBuild expressions.");
        return version;
    }
}

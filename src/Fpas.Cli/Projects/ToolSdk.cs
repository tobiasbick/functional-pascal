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
            if (directory?.Name == "tools" && directory.Parent is { } package && File.Exists(Path.Combine(package.FullName, "fpas.net.sdk.nuspec")))
            {
                var manifest = XDocument.Load(Path.Combine(package.FullName, "fpas.net.sdk.nuspec"));
                return manifest.Descendants().Single(element => element.Name.LocalName == "version").Value;
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

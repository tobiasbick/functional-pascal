using Fpas.Workspaces.Processes;
using System.Xml.Linq;

namespace Fpas.Cli.Projects;

internal static class ProjectFiles
{
    internal static string Project(string version, bool library, string? source = null, string? output = null, string? assembly = null)
    {
        var properties = new XElement("PropertyGroup", new XElement("TargetFramework", "net10.0"), new XElement("OutputType", library ? "Library" : "Exe"));
        var project = new XElement("Project", new XAttribute("Sdk", "Fpas.NET.Sdk/" + ToolSdk.ValidateVersion(version)), properties);
        if (source is not null)
        {
            properties.Add(new XElement("EnableDefaultFpasCompileItems", "false"));
            project.Add(new XElement("ItemGroup", new XElement("Compile", new XAttribute("Include", MsBuildSyntax.Escape(source)))));
        }
        if (output is not null) properties.Add(new XElement("BaseOutputPath", MsBuildSyntax.Escape(output + Path.DirectorySeparatorChar)));
        if (assembly is not null) properties.Add(new XElement("AssemblyName", MsBuildSyntax.Escape(assembly)));
        return project + Environment.NewLine;
    }

    internal static string Feed(string feed, string directory)
    {
        string source;
        if (!Path.IsPathFullyQualified(feed) && Uri.TryCreate(feed, UriKind.Absolute, out var uri))
        {
            if (uri.Scheme != "https" || uri.UserInfo.Length != 0) throw new ArgumentException("--sdk-feed URLs must use HTTPS without embedded credentials.");
            source = uri.AbsoluteUri;
        }
        else source = Path.GetRelativePath(directory, Path.GetFullPath(feed));
        return new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas"), new XAttribute("value", source)),
            new XElement("add", new XAttribute("key", "nuget.org"), new XAttribute("value", "https://api.nuget.org/v3/index.json")))) + Environment.NewLine;
    }

    internal static string Solution(IEnumerable<string> projects, string directory)
        => new XElement("Solution", new XElement("Configurations",
            new XElement("BuildType", new XAttribute("Name", "Debug")), new XElement("BuildType", new XAttribute("Name", "Release")),
            new XElement("Platform", new XAttribute("Name", "Any CPU")),
            new XElement("ProjectType", new XAttribute("Name", "FPAS"), new XAttribute("Extension", ".fpasproj"),
                new XAttribute("TypeId", "{5D4B15AB-1024-4803-9015-4F069B13DE5A}"), new XElement("Platform", new XAttribute("Project", "AnyCPU")))),
            projects.Select(path => new XElement("Project", new XAttribute("Path", Path.GetRelativePath(directory, path).Replace('\\', '/'))))) + Environment.NewLine;

    internal static string ValidateName(string name)
    {
        var identifier = Fpas.Compiler.FpasIdentifiers.SpellQualified(name);
        if (new[] { "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9" }.Contains(name.Split('.')[0], StringComparer.OrdinalIgnoreCase))
            throw new ArgumentException("Use a Pascal qualified identifier as --name (letters, digits, underscores and dots; each part starts with a letter or underscore).");
        return identifier;
    }
}

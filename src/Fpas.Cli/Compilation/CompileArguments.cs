// [Documentation](../../../docs/pascal/program-structure/cli.md#inspect-and-invoke-the-compiler)
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal sealed record WindowsResourceArguments(string FileVersion, string ProductVersion, string Title,
    string Product, string Company, string Copyright, string? Icon);

internal sealed record CompileArguments(IReadOnlyList<string> Sources, string Output, IReadOnlyList<string> References, bool Json, CompilationTarget Target, bool EmitDebug,
    string? Documentation, DiagnosticOptions DiagnosticOptions, bool Optimize, string? KeyFile, bool PublicSign, bool DelaySign,
    IReadOnlyList<(string Name, string Path)> Resources, WindowsResourceArguments? WindowsResources)
{
    internal CompileArguments Validate()
    {
        var output = Path.GetFullPath(Output);
        var documentation = Documentation is null ? null : Path.GetFullPath(Documentation);
        if (Sources.Count == 0 || References.Count == 0)
            throw new ArgumentException("compile requires sources, --output, and --reference or --reference-dir.");
        if (!string.Equals(Path.GetExtension(output), ".dll", StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("--output must name a .dll file.");
        if (string.IsNullOrWhiteSpace(Path.GetFileNameWithoutExtension(output)))
            throw new ArgumentException("--output requires an assembly name before '.dll'.");
        if (documentation is not null && !Path.GetExtension(documentation).Equals(".xml", StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("--documentation must name an .xml file.");
        var comparison = OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal;
        var unique = new HashSet<string>(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        foreach (var source in Sources)
        {
            if (!unique.Add(source == "-" ? "-" : Path.GetFullPath(source)))
                throw new ArgumentException("Specify each source only once; stdin may be read at most once.");
            if (source != "-" && (string.Equals(Path.GetFullPath(source), Path.GetFullPath(output), comparison) ||
                string.Equals(Path.GetFullPath(source), Path.ChangeExtension(Path.GetFullPath(output), ".runtimeconfig.json"), comparison) ||
                string.Equals(Path.GetFullPath(source), Path.ChangeExtension(Path.GetFullPath(output), ".pdb"), comparison)))
                throw new ArgumentException("Source paths must differ from assembly, runtime configuration, and PDB output paths.");
            if (source != "-" && string.Equals(Path.GetFullPath(source), documentation, comparison))
                throw new ArgumentException("Source paths must differ from XML documentation output paths.");
        }
        var artifacts = new[] { Path.GetFullPath(output), Path.ChangeExtension(Path.GetFullPath(output), ".pdb"),
            Path.ChangeExtension(Path.GetFullPath(output), ".runtimeconfig.json"), documentation };
        if (KeyFile is not null && artifacts.Any(artifact => string.Equals(Path.GetFullPath(KeyFile), artifact, comparison)))
            throw new ArgumentException("The signing key must differ from all output paths.");
        if (References.Any(reference => artifacts.Any(artifact => string.Equals(reference, artifact, comparison))))
            throw new ArgumentException("Reference paths must differ from assembly, runtime configuration, PDB, and XML documentation output paths.");
        if (Resources.Any(resource => artifacts.Any(artifact => string.Equals(resource.Path, artifact, comparison))))
            throw new ArgumentException("Embedded resource paths must differ from all output paths.");
        if (WindowsResources?.Icon is { } icon && artifacts.Any(artifact => string.Equals(icon, artifact, comparison)))
            throw new ArgumentException("ApplicationIcon must differ from all output paths.");
        return this with { Output = output, Documentation = documentation };
    }
}

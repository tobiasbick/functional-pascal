using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal sealed record CompileArguments(IReadOnlyList<string> Sources, string Output, IReadOnlyList<string> References, bool Json, CompilationTarget Target, bool EmitDebug,
    string? Documentation, DiagnosticOptions DiagnosticOptions)
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
        if (References.Any(reference => artifacts.Any(artifact => string.Equals(reference, artifact, comparison))))
            throw new ArgumentException("Reference paths must differ from assembly, runtime configuration, PDB, and XML documentation output paths.");
        return this with { Output = output, Documentation = documentation };
    }
}

using Fpas.Compiler;
using System.Text.Json;

namespace Fpas.Cli.Compilation;

internal sealed record CompileArguments(IReadOnlyList<string> Sources, string Output, IReadOnlyList<string> References, bool Json, CompilationTarget Target, bool EmitDebug,
    string? Documentation, DiagnosticOptions DiagnosticOptions)
{
    internal static string[] Expand(string[] args)
    {
        if (args is ["--response-file", var response])
        {
            try { args = JsonSerializer.Deserialize<string[]>(File.ReadAllText(response)) ?? throw new JsonException("Missing arguments."); }
            catch (JsonException exception) { throw new ArgumentException("Invalid JSON argument array in response file.", exception); }
        }
        if (args.Any(argument => argument is null))
            throw new ArgumentException("Response file arguments must be non-null strings.");
        return args;
    }

    internal static CompileArguments Parse(string[] args)
    {
        var sources = new List<string>();
        string? output = null;
        var references = new List<string>();
        var json = false;
        var target = CompilationTarget.Exe;
        var emitDebug = true;
        var emitDocumentation = false;
        string? documentation = null;
        var warnAsError = false;
        var warningsAsErrors = new List<string>(); var warningsNotAsErrors = new List<string>(); var noWarn = new List<string>();
        for (var index = 0; index < args.Length; index++)
        {
            var argument = args[index];
            switch (argument)
            {
                case "--generate-documentation": emitDocumentation = true; break;
                case "--documentation":
                    if (documentation is not null) throw new ArgumentException("Specify --documentation only once.");
                    documentation = Value(args, ref index); emitDocumentation = true; break;
                case "--warn-as-error": warnAsError = true; break;
                case "--warnings-as-errors": warningsAsErrors.AddRange(Codes(Value(args, ref index))); break;
                case "--warnings-not-as-errors": warningsNotAsErrors.AddRange(Codes(Value(args, ref index))); break;
                case "--no-warn": noWarn.AddRange(Codes(Value(args, ref index))); break;
                case "--debug":
                    emitDebug = Value(args, ref index) switch
                    {
                        "portable" => true, "none" => false,
                        _ => throw new ArgumentException("--debug must be portable or none.")
                    };
                    break;
                case "--target":
                    target = Value(args, ref index) switch
                    {
                        "exe" => CompilationTarget.Exe,
                        "library" => CompilationTarget.Library,
                        _ => throw new ArgumentException("--target must be exe or library.")
                    };
                    break;
                case "--output":
                    if (output is not null)
                        throw new ArgumentException("Specify --output only once.");
                    output = Value(args, ref index);
                    break;
                case "--reference":
                    references.Add(Path.GetFullPath(Value(args, ref index)));
                    break;
                case "--reference-dir":
                    var directory = Path.GetFullPath(Value(args, ref index));
                    references.AddRange(Directory.EnumerateFiles(directory, "*.dll").Order(StringComparer.Ordinal));
                    break;
                case "--diagnostics":
                    if (Value(args, ref index) != "json")
                        throw new ArgumentException("The supported diagnostics format is 'json'.");
                    json = true;
                    break;
                default:
                    if (argument.StartsWith('-') && argument != "-")
                        throw new ArgumentException($"Unexpected argument '{argument}'.");
                    sources.Add(argument);
                    break;
            }
        }
        if (sources.Count == 0 || output is null || references.Count == 0)
            throw new ArgumentException("compile requires sources, --output, and --reference or --reference-dir.");
        if (!string.Equals(Path.GetExtension(output), ".dll", StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("--output must name a .dll file.");
        if (string.IsNullOrWhiteSpace(Path.GetFileNameWithoutExtension(output)))
            throw new ArgumentException("--output requires an assembly name before '.dll'.");
        documentation = emitDocumentation ? Path.GetFullPath(documentation ?? Path.ChangeExtension(output, ".xml")) : null;
        if (documentation is not null && !Path.GetExtension(documentation).Equals(".xml", StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("--documentation must name an .xml file.");
        var comparison = OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal;
        var unique = new HashSet<string>(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        foreach (var source in sources)
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
        if (references.Any(reference => artifacts.Any(artifact => string.Equals(reference, artifact, comparison))))
            throw new ArgumentException("Reference paths must differ from assembly, runtime configuration, PDB, and XML documentation output paths.");
        return new(sources, Path.GetFullPath(output), references, json, target, emitDebug, documentation, new(warnAsError, warningsAsErrors, warningsNotAsErrors, noWarn));
    }

    private static string[] Codes(string value) => value.Split([';', ','], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

    private static string Value(string[] args, ref int index)
    {
        if (index + 1 == args.Length || args[index + 1].StartsWith("--", StringComparison.Ordinal))
            throw new ArgumentException($"Missing value for '{args[index]}'.");
        return args[++index];
    }
}

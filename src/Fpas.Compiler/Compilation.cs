using Fpas.Compiler.Binding;
using Fpas.Compiler.Emission;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Documentation;

namespace Fpas.Compiler;

public sealed record SourceLocation(string Path, int Line, int Column, int Length);

public sealed record CompilerDiagnostic(string Code, string Message, SourceLocation Location, DiagnosticSeverity Severity = DiagnosticSeverity.Error);

public enum CompilationTarget { Exe, Library }

public sealed record CompilationSource(string Path, string Text);

public sealed record CompilationRequest(
    IReadOnlyList<CompilationSource> Sources, string AssemblyName, IReadOnlyList<string> References,
    CompilationTarget Target = CompilationTarget.Exe, bool EmitDebugInformation = false, bool EmitDocumentation = false, DiagnosticOptions? DiagnosticOptions = null);

public sealed record CompiledProgram(byte[] AssemblyImage, string? RuntimeConfiguration, byte[]? PortablePdb = null, byte[]? XmlDocumentation = null);

public sealed record CompilationResult(CompiledProgram? Program, IReadOnlyList<CompilerDiagnostic> Diagnostics)
{
    public bool Success => Program is not null && Diagnostics.All(diagnostic => diagnostic.Severity != DiagnosticSeverity.Error);
}

/// <summary>Compiles programs and units using explicitly supplied assembly metadata.</summary>
public static class FpasCompiler
{
    public static CompilationResult Compile(CompilationRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentException.ThrowIfNullOrWhiteSpace(request.AssemblyName);
        ArgumentNullException.ThrowIfNull(request.Sources);
        ArgumentNullException.ThrowIfNull(request.References);
        try
        {
            var sources = request.Sources.OrderBy(input => input.Path, StringComparer.Ordinal).Select(input =>
            {
                var text = new SourceText(input.Path, input.Text);
                return new SourceUnit(text, new Parser(text, new Lexer(text).ReadTokens()).ParseCompilationUnit());
            }).ToArray();
            ValidateTarget(sources, request.Target);
            using var references = ReferenceCatalog.Open(request.References, sources[0].Source);
            var catalog = SourceCatalog.Declare(sources, references);
            var compilation = new BoundCompilation(references.Runtime, request.Target == CompilationTarget.Exe,
                catalog.Units.Select(unit => new Binder(unit, catalog, references).Bind()).ToArray());
            byte[]? documentation = null;
            IReadOnlyList<CompilerDiagnostic> diagnostics = [];
            if (request.EmitDocumentation)
            {
                var result = DocumentationWriter.Write(catalog, compilation, request.AssemblyName);
                documentation = result.Image;
                diagnostics = (request.DiagnosticOptions ?? new()).Apply(result.Diagnostics);
                if (diagnostics.Any(diagnostic => diagnostic.Severity == DiagnosticSeverity.Error)) return new(null, diagnostics);
            }
            var output = AssemblyEmitter.Emit(compilation, request.AssemblyName, catalog.Units.Select(unit => unit.Source), references, request.EmitDebugInformation);
            return new(output with { XmlDocumentation = documentation }, diagnostics);
        }
        catch (CompilationFailure failure)
        {
            return new(null, [failure.Diagnostic]);
        }
        catch (BadImageFormatException exception)
        {
            return new(null, [new("FP4001", exception.Message, new(request.Sources.FirstOrDefault()?.Path ?? "<references>", 1, 1, 0))]);
        }
    }

    private static void ValidateTarget(IReadOnlyList<SourceUnit> sources, CompilationTarget target)
    {
        var programs = sources.Where(unit => unit.Syntax.IsProgram).ToArray();
        if (sources.Count != 0 && (target == CompilationTarget.Exe && programs.Length == 1 || target == CompilationTarget.Library && programs.Length == 0))
            return;
        var invalid = programs.Skip(target == CompilationTarget.Exe ? 1 : 0).FirstOrDefault() ?? sources.FirstOrDefault();
        throw (invalid?.Source ?? new SourceText("<compilation>", "")).Error("FP2004",
            "Executable output requires exactly one program; library output requires one or more units and no program.", invalid?.Syntax.Name.Position ?? 0);
    }
}

internal sealed class CompilationFailure(CompilerDiagnostic diagnostic) : Exception(diagnostic.Message)
{
    internal CompilerDiagnostic Diagnostic { get; } = diagnostic;
}

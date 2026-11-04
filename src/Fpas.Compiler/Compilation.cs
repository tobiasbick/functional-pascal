using Fpas.Compiler.Analysis;
using Fpas.Compiler.Emission.Metadata;

namespace Fpas.Compiler;

public sealed record SourceLocation(string Path, int Line, int Column, int Length);

public sealed record CompilerDiagnostic(string Code, string Message, SourceLocation Location, DiagnosticSeverity Severity = DiagnosticSeverity.Error);

public enum CompilationTarget { Exe, Library }

public sealed record CompilationSource(string Path, string Text);

public sealed record CompilationRequest(
    IReadOnlyList<CompilationSource> Sources, string AssemblyName, IReadOnlyList<string> References,
    CompilationTarget Target = CompilationTarget.Exe, bool EmitDebugInformation = false, bool EmitDocumentation = false, DiagnosticOptions? DiagnosticOptions = null,
    bool Optimize = false);

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
        try
        {
            using var prepared = PreparedCompilation.Create(new(request.Sources, request.AssemblyName, request.References,
                request.Target, request.EmitDocumentation, request.DiagnosticOptions), CancellationToken.None);
            if (!prepared.Success) return new(null, prepared.Diagnostics);
            var documentation = prepared.Documentation?.Serialize(request.AssemblyName);
            var output = AssemblyEmitter.Emit(prepared, request.AssemblyName, request.EmitDebugInformation, request.Optimize);
            return new(output with { XmlDocumentation = documentation }, prepared.Diagnostics);
        }
        catch (CompilationFailure failure)
        {
            return new(null, [failure.Diagnostic]);
        }
        catch (BadImageFormatException exception)
        {
            return new(null, [InvalidReference(exception, request.Sources)]);
        }
    }

    /// <summary>Validates explicit source/metadata inputs without generating PE, PDB or XML output.</summary>
    public static AnalysisResult Analyze(AnalysisRequest request, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        try
        {
            using var prepared = PreparedCompilation.Create(request, cancellationToken, collectSemantics: true);
            cancellationToken.ThrowIfCancellationRequested();
            return new(prepared.Success, prepared.Diagnostics, prepared.Success ? prepared.Analysis!.Finish() : null);
        }
        catch (CompilationFailure failure) { cancellationToken.ThrowIfCancellationRequested(); return new(false, [failure.Diagnostic]); }
        catch (BadImageFormatException exception) { cancellationToken.ThrowIfCancellationRequested(); return new(false, [InvalidReference(exception, request.Sources)]); }
    }

    private static CompilerDiagnostic InvalidReference(BadImageFormatException exception, IReadOnlyList<CompilationSource> sources)
        => new("FP4001", exception.Message, new(sources.FirstOrDefault()?.Path ?? "<references>", 1, 1, 0));
}

internal sealed class CompilationFailure(CompilerDiagnostic diagnostic) : Exception(diagnostic.Message)
{
    internal CompilerDiagnostic Diagnostic { get; } = diagnostic;
}

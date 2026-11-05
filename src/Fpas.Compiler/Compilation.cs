using Fpas.Compiler.Analysis;
using Fpas.Compiler.Emission.Metadata;

namespace Fpas.Compiler;

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
            return new(null, failure.Diagnostics);
        }
        catch (BadImageFormatException exception)
        {
            return new(null, [InvalidReference(exception)]);
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
        catch (CompilationFailure failure) { cancellationToken.ThrowIfCancellationRequested(); return new(false, failure.Diagnostics); }
        catch (BadImageFormatException exception) { cancellationToken.ThrowIfCancellationRequested(); return new(false, [InvalidReference(exception)]); }
    }

    private static CompilerDiagnostic InvalidReference(BadImageFormatException exception)
        => new("FP4001", exception.Message, null);
}

internal sealed class CompilationFailure : Exception
{
    internal IReadOnlyList<CompilerDiagnostic> Diagnostics { get; }
    internal CompilerDiagnostic Diagnostic => Diagnostics[0];
    internal CompilationFailure(CompilerDiagnostic diagnostic) : this([diagnostic]) { }
    internal static CompilationFailure Many(IReadOnlyList<CompilerDiagnostic> diagnostics) => new(diagnostics);
    private CompilationFailure(IReadOnlyList<CompilerDiagnostic> diagnostics) : base(diagnostics[0].Message)
        => Diagnostics = Array.AsReadOnly(diagnostics.ToArray());
}

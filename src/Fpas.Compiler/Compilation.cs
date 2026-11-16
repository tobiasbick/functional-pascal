using Fpas.Compiler.Analysis;
using Fpas.Compiler.Emission.Metadata;

namespace Fpas.Compiler;

/// <summary>The kind of assembly a compilation produces.</summary>
public enum CompilationTarget
{
    /// <summary>An executable with exactly one program and a runtime configuration.</summary>
    Exe,
    /// <summary>A library built from one or more units and no program.</summary>
    Library
}

/// <summary>One FPAS source file supplied to the compiler.</summary>
/// <param name="Path">The logical path reported in diagnostics and debug information.</param>
/// <param name="Text">The complete source text.</param>
public sealed record CompilationSource(string Path, string Text);

/// <summary>Explicit inputs and output options for one assembly compilation.</summary>
/// <param name="Sources">The FPAS sources forming the assembly.</param>
/// <param name="AssemblyName">The simple name of the produced assembly.</param>
/// <param name="References">Paths of the reference assemblies, including the net10.0 System.Runtime reference.</param>
/// <param name="Target">The kind of assembly to produce.</param>
/// <param name="EmitDebugInformation">Whether to produce a portable PDB.</param>
/// <param name="EmitDocumentation">Whether to validate source documentation and produce an XML documentation file.</param>
/// <param name="DiagnosticOptions">Warning suppression and promotion rules, or <see langword="null"/> for the defaults.</param>
/// <param name="Optimize">Whether to mark the assembly as optimized for the debugger.</param>
/// <param name="ReferenceImages">Captured bytes for some of <paramref name="References"/>, used instead of reading those files.</param>
/// <param name="StrongNameKey">A strong-name key blob, or <see langword="null"/> for an unsigned assembly.</param>
/// <param name="PublicSign">Whether to public-sign with the public key in <paramref name="StrongNameKey"/>.</param>
/// <param name="DelaySign">Whether to reserve the signature for later delay signing.</param>
public sealed record CompilationRequest(
    IReadOnlyList<CompilationSource> Sources, string AssemblyName, IReadOnlyList<string> References,
    CompilationTarget Target = CompilationTarget.Exe, bool EmitDebugInformation = false, bool EmitDocumentation = false, DiagnosticOptions? DiagnosticOptions = null,
    bool Optimize = false, IReadOnlyList<MetadataImage>? ReferenceImages = null,
    byte[]? StrongNameKey = null, bool PublicSign = false, bool DelaySign = false);

/// <summary>The generated outputs of a successful compilation.</summary>
/// <param name="AssemblyImage">The PE image of the produced assembly.</param>
/// <param name="RuntimeConfiguration">The runtimeconfig.json text for an executable, or <see langword="null"/> for a library.</param>
/// <param name="PortablePdb">The portable PDB image, when debug information was requested.</param>
/// <param name="XmlDocumentation">The XML documentation file, when documentation was requested.</param>
public sealed record CompiledProgram(byte[] AssemblyImage, string? RuntimeConfiguration, byte[]? PortablePdb = null, byte[]? XmlDocumentation = null);

/// <summary>The outputs and diagnostics of one compilation.</summary>
/// <param name="Program">The generated outputs, or <see langword="null"/> when compilation failed.</param>
/// <param name="Diagnostics">All reported errors and warnings.</param>
public sealed record CompilationResult(CompiledProgram? Program, IReadOnlyList<CompilerDiagnostic> Diagnostics)
{
    /// <summary>Whether outputs were produced and no diagnostic is an error.</summary>
    public bool Success => Program is not null && Diagnostics.All(diagnostic => diagnostic.Severity != DiagnosticSeverity.Error);
}

/// <summary>Compiles programs and units using explicitly supplied assembly metadata.</summary>
public static class FpasCompiler
{
    /// <summary>Compiles the requested sources into an assembly.</summary>
    /// <param name="request">The sources, references, and output options.</param>
    /// <returns>The generated outputs, or the diagnostics that prevented them.</returns>
    public static CompilationResult Compile(CompilationRequest request)
        => Compile(request, CancellationToken.None);

    /// <summary>Compiles the requested sources into an assembly.</summary>
    /// <param name="request">The sources, references, and output options.</param>
    /// <param name="cancellationToken">Cancels the compilation with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The generated outputs, or the diagnostics that prevented them.</returns>
    public static CompilationResult Compile(CompilationRequest request, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(request);
        try
        {
            using var prepared = PreparedCompilation.Create(new(request.Sources, request.AssemblyName, request.References,
                request.Target, request.EmitDocumentation, request.DiagnosticOptions, request.ReferenceImages), cancellationToken);
            if (!prepared.Success) return new(null, prepared.Diagnostics);
            var documentation = prepared.Documentation?.Serialize(request.AssemblyName);
            var output = AssemblyEmitter.Emit(prepared, request.AssemblyName, request.EmitDebugInformation, request.Optimize,
                request.StrongNameKey, request.PublicSign, request.DelaySign);
            cancellationToken.ThrowIfCancellationRequested();
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
        catch (System.Security.Cryptography.CryptographicException)
        { return new(null, [new("FP4003", "Invalid strong-name signing key or signing mode.", null)]); }
    }

    /// <summary>Validates explicit source/metadata inputs without generating PE, PDB or XML output.</summary>
    /// <param name="request">The sources, references, and validation options.</param>
    /// <param name="cancellationToken">Cancels the analysis with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The diagnostics and, when binding started, a semantic snapshot of the checked regions.</returns>
    public static AnalysisResult Analyze(AnalysisRequest request, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        try
        {
            using var prepared = PreparedCompilation.Create(request, cancellationToken, collectSemantics: true);
            cancellationToken.ThrowIfCancellationRequested();
            var snapshot = prepared.Analysis!.Finish();
            return new(prepared.Success && snapshot.Coverage.IsComplete, prepared.Diagnostics, snapshot);
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

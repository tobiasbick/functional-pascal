namespace Fpas.Compiler.Analysis;

/// <summary>Explicit source and metadata inputs for semantic validation without emitted output.</summary>
public sealed record AnalysisRequest(
    IReadOnlyList<CompilationSource> Sources, string AssemblyName, IReadOnlyList<string> References,
    CompilationTarget Target = CompilationTarget.Exe, bool ValidateDocumentation = false,
    DiagnosticOptions? DiagnosticOptions = null);

/// <summary>Validation completion and diagnostics, independent of assembly generation.</summary>
public sealed record AnalysisResult(bool IsComplete, IReadOnlyList<CompilerDiagnostic> Diagnostics, SemanticSnapshot? Snapshot = null)
{
    public bool Success => IsComplete && Diagnostics.All(diagnostic => diagnostic.Severity != DiagnosticSeverity.Error);
}

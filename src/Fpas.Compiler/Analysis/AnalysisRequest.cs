namespace Fpas.Compiler.Analysis;

/// <summary>Explicit source and metadata inputs for semantic validation without emitted output.</summary>
/// <param name="Sources">The FPAS sources forming the assembly.</param>
/// <param name="AssemblyName">The simple name of the analyzed assembly.</param>
/// <param name="References">Paths of the reference assemblies, including the net10.0 System.Runtime reference.</param>
/// <param name="Target">The kind of assembly the sources form.</param>
/// <param name="ValidateDocumentation">Whether to validate source documentation.</param>
/// <param name="DiagnosticOptions">Warning suppression and promotion rules, or <see langword="null"/> for the defaults.</param>
/// <param name="ReferenceImages">Captured bytes for some of <paramref name="References"/>, used instead of reading those files.</param>
public sealed record AnalysisRequest(
    IReadOnlyList<CompilationSource> Sources, string AssemblyName, IReadOnlyList<string> References,
    CompilationTarget Target = CompilationTarget.Exe, bool ValidateDocumentation = false,
    DiagnosticOptions? DiagnosticOptions = null, IReadOnlyList<MetadataImage>? ReferenceImages = null);

/// <summary>Validation completion and diagnostics, independent of assembly generation.</summary>
/// <param name="IsComplete">Whether analysis succeeded and every source region was checked.</param>
/// <param name="Diagnostics">All reported errors and warnings.</param>
/// <param name="Snapshot">The semantic facts of the checked regions, when binding started.</param>
public sealed record AnalysisResult(bool IsComplete, IReadOnlyList<CompilerDiagnostic> Diagnostics, SemanticSnapshot? Snapshot = null)
{
    /// <summary>Whether analysis is complete and no diagnostic is an error.</summary>
    public bool Success => IsComplete && Diagnostics.All(diagnostic => diagnostic.Severity != DiagnosticSeverity.Error);
    /// <summary>"complete", "partial", or "unavailable", describing how much of the source has semantic facts.</summary>
    public string Availability => Snapshot is null || Snapshot.Coverage.Regions.All(region => region.State == "unavailable") ? "unavailable" : IsComplete ? "complete" : "partial";
}

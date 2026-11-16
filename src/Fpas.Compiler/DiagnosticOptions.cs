namespace Fpas.Compiler;

/// <summary>The severity of a compiler diagnostic.</summary>
public enum DiagnosticSeverity
{
    /// <summary>The diagnostic prevents output.</summary>
    Error,
    /// <summary>The diagnostic is reported without preventing output.</summary>
    Warning
}

/// <summary>Warning suppression and promotion rules, matching the MSBuild warning properties.</summary>
/// <param name="TreatWarningsAsErrors">Whether every warning not listed in <paramref name="WarningsNotAsErrors"/> becomes an error.</param>
/// <param name="WarningsAsErrors">Warning codes promoted to errors; bare numbers mean FP codes.</param>
/// <param name="WarningsNotAsErrors">Warning codes kept as warnings when <paramref name="TreatWarningsAsErrors"/> is set.</param>
/// <param name="NoWarn">Warning codes that are not reported.</param>
public sealed record DiagnosticOptions(bool TreatWarningsAsErrors = false, IReadOnlyList<string>? WarningsAsErrors = null,
    IReadOnlyList<string>? WarningsNotAsErrors = null, IReadOnlyList<string>? NoWarn = null)
{
    internal IReadOnlyList<CompilerDiagnostic> Apply(IEnumerable<CompilerDiagnostic> diagnostics)
    {
        static string Normalize(string code) => code.All(char.IsAsciiDigit) ? "FP" + code : code.ToUpperInvariant();
        bool Contains(IReadOnlyList<string>? codes, string code) => codes?.Any(value => Normalize(value) == Normalize(code)) == true;
        return diagnostics.Where(diagnostic => diagnostic.Severity != DiagnosticSeverity.Warning || !Contains(NoWarn, diagnostic.Code))
            .Select(diagnostic => diagnostic.Severity == DiagnosticSeverity.Warning &&
                (Contains(WarningsAsErrors, diagnostic.Code) || TreatWarningsAsErrors && !Contains(WarningsNotAsErrors, diagnostic.Code))
                    ? diagnostic with { Severity = DiagnosticSeverity.Error } : diagnostic).ToArray();
    }
}

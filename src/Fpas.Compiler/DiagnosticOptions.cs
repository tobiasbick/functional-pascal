namespace Fpas.Compiler;

public enum DiagnosticSeverity { Error, Warning }

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

namespace Fpas.Compiler.Diagnostics;

internal sealed class DiagnosticBag
{
    internal const int Limit = 50;
    private readonly List<CompilerDiagnostic> diagnostics = [];
    private int errors;
    internal int Count => diagnostics.Count;
    internal bool HasErrors => errors != 0;
    internal IReadOnlyList<CompilerDiagnostic> Snapshot => Array.AsReadOnly(diagnostics.ToArray());

    internal void Add(CompilerDiagnostic diagnostic)
    {
        if (diagnostic.Severity == DiagnosticSeverity.Error && errors == Limit - 1)
        {
            diagnostics.Add(new("FP0003", "The diagnostic limit was reached; further errors were not analyzed.", diagnostic.Location)
                { Help = "Correct the reported errors and check again. At most 50 errors are returned." });
            ThrowIfErrors();
        }
        diagnostics.Add(diagnostic);
        if (diagnostic.Severity == DiagnosticSeverity.Error) errors++;
    }

    internal void ThrowIfErrors()
    {
        if (errors != 0) throw CompilationFailure.Many(diagnostics);
    }
}

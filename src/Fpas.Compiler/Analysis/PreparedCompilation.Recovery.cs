using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Semantics;
using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

internal sealed partial class PreparedCompilation
{
    private static SourceCatalog RecoverCatalog(AnalysisRequest request, IReadOnlyList<SourceUnit> parsed, IReadOnlyList<SourceText> sources,
        IReadOnlyDictionary<SourceText, IReadOnlyList<Token>> tokens, IReadOnlyDictionary<SourceText, IReadOnlyList<(int Start, int End)>> invalid,
        IReadOnlyDictionary<SourceText, string?> names,
        ReferenceCatalog references, DiagnosticBag diagnostics, CancellationToken token, out SemanticRecorder recorder)
    {
        var excluded = sources.Where(source => parsed.All(unit => unit.Source != source)).ToDictionary(source => source, _ => "syntax_error");
        while (true)
        {
            token.ThrowIfCancellationRequested();
            // An unavailable declaration scope cannot resolve to an older DLL or a
            // different namespace/type. Remove its source consumers transitively.
            bool changed;
            do
            {
                changed = false;
                foreach (var unit in parsed.Where(unit => !excluded.ContainsKey(unit.Source)))
                    if (excluded.Keys.Any(source => names[source] is not { } name || unit.Syntax.Imports.Any(import => Related(import.Target.Text, name))))
                    { excluded.Add(unit.Source, "declaration_dependency"); changed = true; }
            } while (changed);
            recorder = new(request.AssemblyName, references, SemanticRecorder.Fingerprint(request, references, token), token);
            foreach (var source in sources)
            {
                var syntax = parsed.FirstOrDefault(unit => unit.Source == source)?.Syntax;
                recorder.Source(source, tokens.GetValueOrDefault(source) ?? [], syntax?.Declarations ?? new Dictionary<int, DeclarationSyntax>());
                if (excluded.TryGetValue(source, out var reason)) recorder.Unavailable(source, 0, source.Text.Length, reason);
                else foreach (var (start, end) in invalid[source]) recorder.Unavailable(source, start, end, "syntax_error");
            }
            var available = parsed.Where(unit => !excluded.ContainsKey(unit.Source)).Select(unit => new SourceUnit(unit.Source, unit.Syntax)).ToArray();
            try { return SourceCatalog.Declare(available, references, recorder); }
            catch (CompilationFailure failure) when (failure.Diagnostics.Count == 1)
            {
                var offender = available.FirstOrDefault(unit => unit.Source.Path == failure.Diagnostic.Location?.Path);
                if (offender is null) throw;
                diagnostics.Add(failure.Diagnostic); excluded.Add(offender.Source, "declaration_error");
                if (failure.Diagnostic.Code == "FP2005")
                    foreach (var duplicate in available.Where(unit => unit.Syntax.Name.Text == offender.Syntax.Name.Text && unit.Source != offender.Source))
                        excluded.TryAdd(duplicate.Source, "declaration_error");
            }
        }
    }

    private static bool Related(string import, string name) => import == name || import.StartsWith(name + ".", StringComparison.Ordinal) || name.StartsWith(import + ".", StringComparison.Ordinal);
}

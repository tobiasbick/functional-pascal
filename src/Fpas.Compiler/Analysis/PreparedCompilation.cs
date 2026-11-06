using Fpas.Compiler.Binding.Bodies;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Semantics;
using Fpas.Compiler.Documentation;
using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Lowering.Async;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

internal sealed partial class PreparedCompilation : IDisposable
{
    internal SourceCatalog Catalog { get; }
    internal ReferenceCatalog References { get; }
    internal BoundCompilation Program { get; }
    internal IReadOnlyDictionary<BoundRoutine, AsyncPlan> AsyncPlans { get; }
    internal DocumentationModel? Documentation { get; }
    internal IReadOnlyList<CompilerDiagnostic> Diagnostics { get; }
    internal SemanticRecorder? Analysis { get; }
    internal bool Success => Diagnostics.All(diagnostic => diagnostic.Severity != DiagnosticSeverity.Error);

    private PreparedCompilation(SourceCatalog catalog, ReferenceCatalog references, BoundCompilation program,
        IReadOnlyDictionary<BoundRoutine, AsyncPlan> asyncPlans, DocumentationModel? documentation, IReadOnlyList<CompilerDiagnostic> diagnostics, SemanticRecorder? analysis)
    {
        Catalog = catalog; References = references; Program = program; AsyncPlans = asyncPlans;
        Documentation = documentation; Diagnostics = Array.AsReadOnly(diagnostics.ToArray());
        Analysis = analysis;
    }

    internal static PreparedCompilation Create(AnalysisRequest request, CancellationToken cancellationToken, bool collectSemantics = false)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentException.ThrowIfNullOrWhiteSpace(request.AssemblyName);
        ArgumentNullException.ThrowIfNull(request.Sources);
        ArgumentNullException.ThrowIfNull(request.References);
        cancellationToken.ThrowIfCancellationRequested();
        var diagnosticsBag = new DiagnosticBag();
        var sourceTokens = new Dictionary<SourceText, IReadOnlyList<Token>>();
        var sources = new List<SourceUnit>();
        var allSources = new List<SourceText>();
        var invalid = new Dictionary<SourceText, IReadOnlyList<(int Start, int End)>>();
        var names = new Dictionary<SourceText, string?>();
        foreach (var input in request.Sources.OrderBy(input => input.Path, StringComparer.Ordinal))
        {
            var text = new SourceText(input.Path, input.Text, cancellationToken, diagnosticsBag);
            allSources.Add(text);
            var before = diagnosticsBag.Count;
            Parser? parser = null;
            try
            {
                var tokens = new Lexer(text).ReadTokens(); sourceTokens.Add(text, tokens);
                parser = new Parser(text, tokens, recover: collectSemantics);
                sources.Add(new(text, parser.ParseCompilationUnit())); invalid[text] = parser.InvalidRegions;
            }
            catch (CompilationFailure failure)
            {
                if (failure.Diagnostics.Count > 1) throw;
                // A parser recovery may already have explained an unusable enclosing structure.
                if (diagnosticsBag.Count == before) diagnosticsBag.Add(failure.Diagnostic);
            }
            finally { names[text] = parser?.ContainerName; }
        }
        if (!collectSemantics) diagnosticsBag.ThrowIfErrors();
        if (!diagnosticsBag.HasErrors) ValidateTarget(sources, request.Target);
        var references = ReferenceCatalog.Open(request.References, allSources.FirstOrDefault() ?? new SourceText("", "", cancellationToken), request.ReferenceImages, captureFingerprints: collectSemantics);
        try
        {
            SemanticRecorder? analysis = null;
            var catalog = collectSemantics ? RecoverCatalog(request, sources, allSources, sourceTokens, invalid, names, references, diagnosticsBag, cancellationToken, out analysis)
                : SourceCatalog.Declare(sources, references);
            analysis?.Declarations(catalog);
            var units = new List<BoundUnit>();
            foreach (var unit in catalog.Units)
            {
                try { units.Add(new Binder(unit, catalog, references).Bind()); }
                catch (CompilationFailure failure)
                { if (failure.Diagnostics.Count > 1) throw; analysis?.Unavailable(unit.Source, 0, unit.Source.Text.Length, "binding_error"); diagnosticsBag.Add(failure.Diagnostic); }
            }
            analysis?.PropagateUnavailableScopes(catalog);
            if (!collectSemantics) diagnosticsBag.ThrowIfErrors();
            var program = new BoundCompilation(references.Runtime, request.Target == CompilationTarget.Exe, units);
            var documentation = request.ValidateDocumentation && !diagnosticsBag.HasErrors ? DocumentationWriter.Analyze(catalog, program) : null;
            var diagnostics = (request.DiagnosticOptions ?? new()).Apply(documentation?.Diagnostics ?? []);
            foreach (var diagnostic in diagnostics) diagnosticsBag.Add(diagnostic);
            cancellationToken.ThrowIfCancellationRequested();
            var plans = diagnosticsBag.HasErrors
                ? new Dictionary<BoundRoutine, AsyncPlan>(ReferenceEqualityComparer.Instance)
                : AsyncPreparation.Prepare(program, references, cancellationToken);
            if (!diagnosticsBag.HasErrors) ReferenceRequirements.Validate(program, references);
            return new(catalog, references, program, plans, documentation, diagnosticsBag.Snapshot, analysis);
        }
        catch { references.Dispose(); throw; }
    }

    private static void ValidateTarget(IReadOnlyList<SourceUnit> sources, CompilationTarget target)
    {
        var programs = sources.Where(unit => unit.Syntax.IsProgram).ToArray();
        if (sources.Count != 0 && (target == CompilationTarget.Exe && programs.Length == 1 || target == CompilationTarget.Library && programs.Length == 0)) return;
        var invalid = programs.Skip(target == CompilationTarget.Exe ? 1 : 0).FirstOrDefault() ?? sources.FirstOrDefault();
        const string message = "Executable output requires exactly one program; library output requires one or more units and no program.";
        if (invalid is null) throw new CompilationFailure(new("FP2004", message, null));
        throw invalid.Source.Error("FP2004", message, invalid.Syntax.Name.Position);
    }

    public void Dispose() => References.Dispose();
}

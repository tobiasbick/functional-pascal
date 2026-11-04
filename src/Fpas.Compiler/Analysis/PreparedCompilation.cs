using Fpas.Compiler.Binding.Bodies;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Semantics;
using Fpas.Compiler.Documentation;
using Fpas.Compiler.Lowering.Async;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

internal sealed class PreparedCompilation : IDisposable
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
        var sourceTokens = new Dictionary<SourceText, IReadOnlyList<Token>>();
        var sources = request.Sources.OrderBy(input => input.Path, StringComparer.Ordinal).Select(input =>
        {
            var text = new SourceText(input.Path, input.Text, cancellationToken);
            var tokens = new Lexer(text).ReadTokens(); sourceTokens.Add(text, tokens);
            return new SourceUnit(text, new Parser(text, tokens).ParseCompilationUnit());
        }).ToArray();
        ValidateTarget(sources, request.Target);
        var references = ReferenceCatalog.Open(request.References, sources[0].Source, captureFingerprints: collectSemantics);
        try
        {
            var analysis = collectSemantics ? new SemanticRecorder(request.AssemblyName, references, SemanticRecorder.Fingerprint(request, references, cancellationToken), cancellationToken) : null;
            foreach (var unit in sources) analysis?.Source(unit.Source, sourceTokens[unit.Source], unit.Syntax.Declarations);
            var catalog = SourceCatalog.Declare(sources, references, analysis);
            analysis?.Declarations(catalog);
            var program = new BoundCompilation(references.Runtime, request.Target == CompilationTarget.Exe,
                catalog.Units.Select(unit => new Binder(unit, catalog, references).Bind()).ToArray());
            var documentation = request.ValidateDocumentation ? DocumentationWriter.Analyze(catalog, program) : null;
            var diagnostics = (request.DiagnosticOptions ?? new()).Apply(documentation?.Diagnostics ?? []);
            cancellationToken.ThrowIfCancellationRequested();
            var plans = diagnostics.Any(diagnostic => diagnostic.Severity == DiagnosticSeverity.Error)
                ? new Dictionary<BoundRoutine, AsyncPlan>(ReferenceEqualityComparer.Instance)
                : AsyncPreparation.Prepare(program, references, cancellationToken);
            if (!diagnostics.Any(diagnostic => diagnostic.Severity == DiagnosticSeverity.Error)) ReferenceRequirements.Validate(program, references);
            return new(catalog, references, program, plans, documentation, diagnostics, analysis);
        }
        catch { references.Dispose(); throw; }
    }

    private static void ValidateTarget(IReadOnlyList<SourceUnit> sources, CompilationTarget target)
    {
        var programs = sources.Where(unit => unit.Syntax.IsProgram).ToArray();
        if (sources.Count != 0 && (target == CompilationTarget.Exe && programs.Length == 1 || target == CompilationTarget.Library && programs.Length == 0)) return;
        var invalid = programs.Skip(target == CompilationTarget.Exe ? 1 : 0).FirstOrDefault() ?? sources.FirstOrDefault();
        throw (invalid?.Source ?? new SourceText("<compilation>", "")).Error("FP2004",
            "Executable output requires exactly one program; library output requires one or more units and no program.", invalid?.Syntax.Name.Position ?? 0);
    }

    public void Dispose() => References.Dispose();
}

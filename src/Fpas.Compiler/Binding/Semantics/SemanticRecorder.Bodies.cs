using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private sealed record BodyEntry(SourceBody Body, SymbolEntry? Declaration);
    private readonly Dictionary<object, BodyEntry> bodies = new(ReferenceEqualityComparer.Instance);
    private BodyEntry? activeBody;
    private string operationPhase = "body";

    internal IDisposable Body(SourceText source, RoutineSymbol routine, CompilationUnitSyntax unit, LambdaSyntax? lambda = null)
    {
        if (lambda is not null) return Enter(routine, "lambda", Span(source, lambda.SpanStart, lambda.SpanEnd - lambda.SpanStart), null, []);
        var name = routine.Declaration?.Name ?? (routine.IsEntryPoint ? unit.Name.Parts[^1] : null);
        if (name is null || !sourceDeclarations[source.Path].TryGetValue(name.Position, out var declaration) || declaration.BodyStart is not { } start)
            return new BodyScope(this, null);
        var span = Span(source, start, declaration.BodyEnd!.Value - start);
        var symbol = routine.Declaration is null ? null : symbols[routine];
        var contracts = (routine.Declaration?.Contracts ?? []).Select(clause =>
            Span(source, clause.Condition.SpanStart, clause.Condition.SpanEnd - clause.Condition.SpanStart)).ToArray();
        return Enter(routine, routine.IsEntryPoint ? "program_entry" : "routine", span, symbol, contracts);
    }

    internal IDisposable Initializer(SourceText source, object identity, ExpressionSyntax expression, string kind)
        => Enter(identity, kind, Span(source, expression.SpanStart, expression.SpanEnd - expression.SpanStart), null, []);

    internal IDisposable Phase(string phase)
    {
        var previous = operationPhase; operationPhase = phase;
        return new Restore(() => operationPhase = previous);
    }

    private IDisposable Enter(object identity, string kind, SourceSpan span, SymbolEntry? symbol, IEnumerable<SourceSpan> additional)
    {
        var id = snapshotId + ":b" + bodies.Count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        var body = new BodyEntry(new(id, kind, span, Array.AsReadOnly(additional.Prepend(span).ToArray()), symbol?.Id, activeBody?.Body.Id), symbol);
        bodies.Add(identity, body);
        return new BodyScope(this, body);
    }

    private sealed class BodyScope : IDisposable
    {
        private readonly SemanticRecorder owner;
        private readonly BodyEntry? previous;
        private readonly string phase;
        internal BodyScope(SemanticRecorder owner, BodyEntry? body)
        {
            this.owner = owner; previous = owner.activeBody; phase = owner.operationPhase;
            owner.activeBody = body; owner.operationPhase = "body";
        }
        public void Dispose() { owner.activeBody = previous; owner.operationPhase = phase; }
    }

    private sealed class Restore(Action restore) : IDisposable { public void Dispose() => restore(); }

    internal void Exclude(SourceText source, string kind, int position, int length = 1)
    {
        if (activeBody is not null) excludedOperations.Add(new(activeBody.Body.Id, kind, Span(source, position, length)));
    }
}

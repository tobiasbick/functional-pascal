using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private readonly List<SemanticRegion> invalid = [];
    internal void Unavailable(SourceText source, int start, int end, string reason)
        => invalid.Add(new(Span(source, start, Math.Max(0, end - start)), "unavailable", reason));

    internal bool CanBind(SourceText source, RoutineSymbol routine)
    {
        var declaration = RoutineDeclaration(source, routine);
        return declaration?.BodyStart is not { } start || !Blocked(Span(source, start, declaration.BodyEnd!.Value - start));
    }

    internal void UnavailableBody(SourceText source, RoutineSymbol routine)
    {
        var syntax = routine.Declaration;
        var declaration = RoutineDeclaration(source, routine);
        if (declaration?.BodyStart is { } start)
        {
            start = Math.Min(start, syntax?.Contracts?.FirstOrDefault()?.Condition.SpanStart ?? start);
            Unavailable(source, start, declaration.BodyEnd!.Value, "binding_error");
            if (syntax?.Attributes is { Count: > 0 }) Unavailable(source, declaration.Start, syntax.Name.Position, "binding_error");
        }
        else Unavailable(source, 0, source.Text.Length, "binding_error");
    }

    private DeclarationSyntax? RoutineDeclaration(SourceText source, RoutineSymbol routine) => routine.Declaration is { } syntax ? sourceDeclarations[source.Path].GetValueOrDefault(syntax.Name.Position) :
        routine.IsEntryPoint ? sourceDeclarations[source.Path].Values.FirstOrDefault(item => item.Owner is null) : null;

    internal void PropagateUnavailableScopes(SourceCatalog catalog)
    {
        var unavailable = catalog.Units.Where(unit => invalid.Any(region => region.Span.Path == unit.Source.Path && region.Span.Offset == 0 && region.Span.Length == unit.Source.Text.Length)).ToHashSet();
        bool changed;
        do
        {
            changed = false;
            foreach (var unit in catalog.Units.Where(unit => !unavailable.Contains(unit)))
                if (unavailable.Any(other => unit.Syntax.Imports.Any(import => import.Target.Text == other.Syntax.Name.Text ||
                    import.Target.Text.StartsWith(other.Syntax.Name.Text + ".", StringComparison.Ordinal) || other.Syntax.Name.Text.StartsWith(import.Target.Text + ".", StringComparison.Ordinal))))
                { unavailable.Add(unit); Unavailable(unit.Source, 0, unit.Source.Text.Length, "declaration_dependency"); changed = true; }
        } while (changed);
    }

    private bool Blocked(SourceSpan span) => invalid.Any(region => region.Span.Path == span.Path &&
        (region.Span.Length == 0 ? span.Contains(region.Span.Offset) || span.Length == 0 && span.Offset == region.Span.Offset : span.Offset < (long)region.Span.Offset + region.Span.Length && region.Span.Offset < (long)span.Offset + span.Length));

    private (SemanticSymbol[] Symbols, IEnumerable<SemanticType> Types) RetainedFacts(SemanticSymbol[] candidates, SourceOperation[] checkedOperations)
    {
        if (invalid.Count == 0) return (candidates, types.Values);
        var byId = candidates.ToDictionary(symbol => symbol.Id, StringComparer.Ordinal);
        var keptSymbols = new HashSet<string>(StringComparer.Ordinal); var keptTypes = new HashSet<string>(StringComparer.Ordinal);
        var pendingSymbols = new Stack<string>(candidates.Where(symbol => symbol.Declaration is not null).Select(symbol => symbol.Id));
        var pendingTypes = new Stack<string>();
        foreach (var occurrence in occurrences.Values) pendingSymbols.Push(occurrence.Symbol.Id);
        foreach (var relation in relations)
            if (byId.ContainsKey(relation.Implementation.Id) && byId.ContainsKey(relation.Contract.Id))
            { pendingSymbols.Push(relation.Implementation.Id); pendingSymbols.Push(relation.Contract.Id); }
        foreach (var expression in expressions.Values) { pendingTypes.Push(expression.Type); if (expression.DeclaredType is { } declared) pendingTypes.Push(declared); }
        foreach (var operation in checkedOperations)
        {
            if (operation.Target is { } target) pendingSymbols.Push(target);
            if (operation.Receiver?.Symbol is { } receiver) pendingSymbols.Push(receiver);
            if (operation.Receiver?.Type is { } receiverType) pendingTypes.Push(receiverType);
            if (operation.OwnerType is { } ownerType) pendingTypes.Push(ownerType);
            if (operation.ResultType is { } resultType) pendingTypes.Push(resultType);
            Signature(operation.Signature);
        }
        while (pendingSymbols.Count != 0 || pendingTypes.Count != 0)
        {
            token.ThrowIfCancellationRequested();
            while (pendingSymbols.TryPop(out var id))
            {
                if (!keptSymbols.Add(id)) continue;
                var symbol = byId[id];
                if (symbol.Type is { } type) pendingTypes.Push(type);
                Signature(symbol.Callable);
                if (symbol.Context is not { } context) continue;
                if (context.Owner is { } owner) pendingSymbols.Push(owner);
                foreach (var surface in context.SurfaceSymbols) pendingSymbols.Push(surface);
                foreach (var surface in context.SurfaceTypes) pendingTypes.Push(surface);
            }
            while (pendingTypes.TryPop(out var id))
            {
                if (!keptTypes.Add(id)) continue;
                var type = types[id];
                if (type.Parameter is { } parameter) pendingSymbols.Push(parameter);
                if (type.Element is { } element) pendingTypes.Push(element);
                foreach (var argument in type.Arguments) pendingTypes.Push(argument);
                Signature(type.Callable);
            }
        }
        return (candidates.Where(symbol => keptSymbols.Contains(symbol.Id)).ToArray(), types.Values.Where(type => keptTypes.Contains(type.Id)));

        void Signature(CallableSignature? signature)
        {
            if (signature is null) return;
            pendingTypes.Push(signature.Result);
            foreach (var parameter in signature.Parameters) pendingTypes.Push(parameter.Type);
            foreach (var parameter in signature.TypeParameters) pendingTypes.Push(parameter);
        }
    }
}

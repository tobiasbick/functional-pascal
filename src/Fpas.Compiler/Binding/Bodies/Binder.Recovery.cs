// [Documentation](../../../../docs/architecture/compiler.md#syntax-and-binding-boundaries)
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    // A preceding diagnostic already explains why this binding is unavailable.
    private sealed class UnavailableBinding : Exception;

    private bool recoverStatements = parent?.recoverStatements ?? false;
    private int bindingFaults;

    private BoundRoutine? BindCheckedRoutine(RoutineSymbol symbol, IReadOnlyList<StatementSyntax> body, int position,
        IReadOnlyList<ContractSyntax>? contracts = null)
    {
        recoverStatements = true;
        var routine = Attempt(() => BindRoutine(symbol, body, position, contracts));
        if (routine is null) catalog.Analysis?.UnavailableBody(source, symbol);
        return routine;
    }

    private T? Attempt<T>(Func<T> bind) where T : class
    {
        if (!recoverStatements) return bind();
        try { return bind(); }
        catch (CompilationFailure failure) when (failure.Diagnostics.Count == 1)
        {
            bindingFaults++;
            source.Diagnostics.Add(failure.Diagnostic);
            return null;
        }
        catch (UnavailableBinding)
        {
            bindingFaults++;
            return null;
        }
    }

    private void ReserveFailedBinding(Token token)
    {
        var name = token.Text;
        var scope = scopes.Peek();
        var position = token.Position;
        if (scope.TryGetValue(name, out var previous))
        {
            position = declarationPositions[previous];
            nonNull.Remove(previous);
        }
        var local = new LocalSymbol(name, TypeSymbol.Unsupported, -1) { Unavailable = true };
        scope[name] = local;
        declarationPositions.Add(local, position);
    }
}

using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private IReadOnlyList<ImportedMethod> SourceMethods(IReadOnlyList<RoutineSymbol> group, TypeList? arguments, int position, BoundExpression? receiver, bool delegateGroup = false)
    {
        var methods = group.Where(method => !method.IsInstance || receiver is not null)
            .Where(method => arguments is null ? !delegateGroup || method.GenericParameters.Count == 0 : method.GenericParameters.Count == arguments.Count)
            .Select(method => references.InstantiateSource(method, method.Owner?.Type ?? TypeSymbol.Object,
                arguments ?? (method.GenericParameters.Count == 0 ? null : new(method.GenericParameters.Select(parameter => parameter.Type)))))
            .ToArray();
        if (methods.Length == 0 && group.Any(method => method.GenericParameters.Count != 0))
            throw source.Error("FP3026", "Generic routine groups require matching explicit type arguments when used as delegates.", position);
        return methods;
    }

    private BoundExpression? ImplicitReceiver(IReadOnlyList<RoutineSymbol> group, int position)
        => group.Any(method => method.IsInstance) && HasLexicalValue("self") ? Value("self", position) : null;

    private BoundExpression CallRoutine(IReadOnlyList<RoutineSymbol> group, IReadOnlyList<Syntax.ExpressionSyntax> arguments, TypeList? typeArguments, bool check, int position, BoundExpression? receiver = null)
        => BindInvocation(SourceMethods(group, typeArguments, position, receiver), arguments, receiver, check, position, inferGenerics: typeArguments is null);

    private BoundExpression RoutineDelegate(IReadOnlyList<RoutineSymbol> group, TypeSymbol? expected, int position, TypeList? typeArguments = null, BoundExpression? receiver = null)
    {
        var methods = SourceMethods(group, typeArguments, position, receiver, delegateGroup: true).Where(references.ValidMethodArguments).ToArray();
        if (expected is null && methods.Length > 1) return new BoundMethodGroup(methods, receiver);
        var method = expected is null ? methods.SingleOrDefault() : references.DelegateMethod(expected, methods);
        if (method is null) throw source.Error("FP3019", "A routine group needs one compatible delegate signature.", position);
        return new BoundMethodDelegate(method, method.IsStatic ? null : receiver,
            NaturalDelegate(expected, method.DelegateParameters, method.Result, position, method.ParameterInfo));
    }

    private IReadOnlyList<RoutineSymbol> AccessibleSource(SourceUnit owner, string name, int position)
    {
        if (!owner.Routines.TryGetValue(name, out var group)) throw source.Error("FP3001", $"Unknown routine '{name}'.", position);
        var accessible = group.Where(method => method.IsPublic || owner == unit).ToArray();
        if (accessible.Length == 0) throw source.Error("FP3011", $"Routine '{name}' is private.", position);
        return accessible;
    }
}

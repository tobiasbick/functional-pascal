using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private sealed class NestedScope(ClosureSymbol environment)
    {
        internal ClosureSymbol Environment { get; } = environment;
        internal Dictionary<string, List<RoutineSymbol>> Methods { get; } = new(StringComparer.Ordinal);
    }

    private readonly Dictionary<Dictionary<string, LocalSymbol>, NestedScope> nestedScopes = new(ReferenceEqualityComparer.Instance);
    private HashSet<string> potentialNestedCaptures = new(StringComparer.Ordinal);

    private NestedScope? ScopeRoutines(Dictionary<string, LocalSymbol> scope) => nestedScopes.GetValueOrDefault(scope);

    private (NestedScope Scope, IReadOnlyList<RoutineSymbol> Methods)? FindNested(string name)
    {
        foreach (var scope in scopes)
        {
            if (scope.ContainsKey(name)) return null;
            if (ScopeRoutines(scope) is { } nested && nested.Methods.TryGetValue(name, out var methods)) return (nested, methods);
        }
        return parameters.ContainsKey(name) ? null : parent?.FindNested(name);
    }

    private NestedScope? PrepareNestedRoutines(IReadOnlyList<StatementSyntax> statements)
    {
        var declarations = statements.OfType<NestedRoutineSyntax>().ToArray();
        if (declarations.Length == 0 || nestedScopes.ContainsKey(scopes.Peek())) return null;
        var environment = new ClosureSymbol(catalog.ClosureCount++, recordContext, TypeParameters.Values.ToArray());
        closures.Add(environment);
        var scope = new NestedScope(environment); nestedScopes.Add(scopes.Peek(), scope);
        foreach (var declaration in declarations)
        {
            var syntax = declaration.Routine;
            if (imports.ContainsKey(syntax.Name.Text) || scopes.Peek().ContainsKey(syntax.Name.Text) ||
                scopes.Count == 1 && parameters.ContainsKey(syntax.Name.Text) ||
                currentRoutine.IsEntryPoint && syntax.Name.Text == "args" || recordContext is not null && syntax.Name.Text == "self")
                throw source.Error("FP3004", "A nested routine name conflicts with a binding or reserved name.", syntax.Name.Position);
            var method = catalog.DeclareRoutine(unit, syntax, references, -1 - environment.Id, enclosing: environment.GenericParameters) with { IsInstance = true };
            if (recordContext is not null && method.Parameters.Any(parameter => parameter.Name == "self"))
                throw source.Error("FP3004", "Nested parameters cannot shadow the enclosing record's self.", syntax.Name.Position);
            SourceCatalog.AddOverload(scope.Methods, method, source, syntax.Name.Position);
            var binder = Child(environment);
            binder.BindDefaults(method, syntax.Parameters);
        }
        return scope;
    }

    private Binder Child(ClosureSymbol environment)
    {
        var binder = new Binder(unit, catalog, references, this, environment, recordContext) { arithmeticChecked = arithmeticChecked };
        foreach (var import in imports) binder.imports.Add(import.Key, import.Value);
        return binder;
    }

    private void BindNestedBodies(NestedScope scope)
    {
        var facts = nonNull.ToArray(); nonNull.Clear();
        try
        {
            foreach (var method in scope.Methods.Values.SelectMany(group => group))
            {
                var syntax = method.Declaration!;
                var body = Child(scope.Environment).BindRoutine(method, syntax.Statements, syntax.Name.Position, syntax.Contracts);
                scope.Environment.NestedBodies.Add(body);
                scope.Environment.Body ??= body;
            }
        }
        finally { SetFacts(facts); }
    }

    private TypeSymbol NestedOwner(NestedScope scope) => scope.Environment.GenericParameters.Count == 0 ? scope.Environment.Type :
        scope.Environment.Type with { Arguments = new(scope.Environment.GenericParameters.Select(parameter => parameter.Type)) };

    private IReadOnlyList<ImportedMethod> NestedMethods(NestedScope scope, IReadOnlyList<RoutineSymbol> methods, TypeList? arguments, bool delegateGroup)
        => methods.Where(method => arguments is null ? !delegateGroup || method.GenericParameters.Count == 0 : method.GenericParameters.Count == arguments.Count)
            .Select(method => references.InstantiateSource(method, NestedOwner(scope), arguments ??
                (method.GenericParameters.Count == 0 ? null : new(method.GenericParameters.Select(parameter => parameter.Type))))).ToArray();

    private BoundExpression CallNested((NestedScope Scope, IReadOnlyList<RoutineSymbol> Methods) group, CallSyntax syntax, TypeList? arguments, bool check)
        => BindInvocation(NestedMethods(group.Scope, group.Methods, arguments, false), syntax.Arguments,
            NestedReceiver(group.Scope.Environment, syntax.Position), check, syntax.Position, inferGenerics: arguments is null);

    private BoundExpression NestedDelegate((NestedScope Scope, IReadOnlyList<RoutineSymbol> Methods) group, TypeSymbol? expected, int position, TypeList? arguments = null)
    {
        var methods = NestedMethods(group.Scope, group.Methods, arguments, true).Where(references.ValidMethodArguments).ToArray();
        var receiver = NestedReceiver(group.Scope.Environment, position);
        if (methods.Length > 1 && expected is null) return new BoundMethodGroup(methods, receiver);
        var method = expected is null ? methods.SingleOrDefault() : references.DelegateMethod(expected, methods);
        if (method is null) throw source.Error("FP3019", "A nested routine group needs a compatible delegate signature and type arguments.", position);
        return new BoundMethodDelegate(method, receiver, NaturalDelegate(expected, method.DelegateParameters, method.Result, position, method.ParameterInfo));
    }

    private static IEnumerable<string> NestedCaptureNames(IEnumerable<StatementSyntax> body)
    {
        foreach (var statement in body)
        {
            if (statement is NestedRoutineSyntax nested)
                foreach (var name in RoutineNames(nested.Routine)) yield return name;
            foreach (var name in NestedCaptureNames(Children(statement))) yield return name;
        }
    }

    private static IEnumerable<string> RoutineNames(RoutineSyntax routine)
        => StatementNames(routine.Statements).Concat((routine.Contracts ?? []).SelectMany(clause => ExpressionNames(clause.Condition)));

    private static IEnumerable<string> StatementNames(IEnumerable<StatementSyntax> statements)
    {
        foreach (var statement in statements)
        {
            if (statement is NestedRoutineSyntax nested)
                foreach (var name in RoutineNames(nested.Routine)) yield return name;
            foreach (var name in Expressions(statement).SelectMany(ExpressionNames)) yield return name;
            foreach (var name in StatementNames(Children(statement))) yield return name;
        }
    }

    private static IEnumerable<string> ExpressionNames(ExpressionSyntax expression)
    {
        if (expression is NameExpressionSyntax identifier) yield return identifier.Name.Text;
        if (expression is LambdaSyntax { Body: { } body })
            foreach (var name in StatementNames(body)) yield return name;
        foreach (var name in Children(expression).SelectMany(ExpressionNames)) yield return name;
    }
}

using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private bool HasLexicalValue(string name)
    {
        foreach (var scope in scopes)
        {
            if (scope.ContainsKey(name)) return true;
            if (ScopeRoutines(scope)?.Methods.ContainsKey(name) == true) return false;
        }
        return parameters.ContainsKey(name) || parent?.HasLexicalValue(name) == true;
    }
    private bool HasValue(string name) => HasLexicalValue(name) || FindNested(name) is not null || unit.Globals.ContainsKey(name) && recordContext?.UserMethods.ContainsKey(name) != true;

    private BoundExpression Value(string name, int position, TypeSymbol? expected = null)
    {
        if (enumInitializer?.Members.FirstOrDefault(member => member.Name.Text == name) is { } enumMember)
        {
            catalog.Analysis?.EnumMemberUse(source, position, enumMember);
            return BoundNumber.Create(enumInitializer.Base, enumMember.Value);
        }
        if (FindLocal(name) is { } local) return new BoundLocal(local, nonNull.Contains(local) ? local.Type.NonNullable : null);
        if (FindNested(name) is { } nested) return NestedDelegate(nested, expected, position);
        if (parameters.TryGetValue(name, out var parameter)) return new BoundParameter(parameter, nonNull.Contains(parameter) ? parameter.Type.NonNullable : null);
        if (parent?.HasLexicalValue(name) == true)
        {
            var original = parent.Value(name, position);
            var symbol = original switch
            {
                BoundLocal bound => (object)bound.Local,
                BoundParameter bound => bound.Parameter,
                BoundCapture bound => bound.Capture.Original,
                _ => throw new InvalidOperationException("Invalid lexical capture.")
            };
            return CaptureValue(symbol, original, position);
        }
        if (recordContext is not null && recordContext.UserMethods.TryGetValue(name, out var recordMethod))
            return RoutineDelegate(recordMethod, expected, position, receiver: ImplicitReceiver(recordMethod, position));
        if (unit.Globals.TryGetValue(name, out var global)) return GlobalValue(global, position);
        if (routines.TryGetValue(name, out var routine)) return RoutineDelegate(routine, expected, position);
        throw source.Error("FP3001", $"Unknown initialized value '{name}'.", position);
    }

    private BoundExpression BindLambda(LambdaSyntax syntax, TypeSymbol? expected, bool check)
    {
        var signature = syntax.Parameters.Select((parameter, index) => new ParameterSymbol(parameter.Name.Text, BindType(parameter.Type), index, parameter.Mode)).ToArray();
        SourceCatalog.ValidateParameterModes(signature, syntax.Parameters, source);
        if (syntax.Parameters.Any(parameter => parameter.Default is not null || parameter.Mode == "params"))
            throw source.Error("FP3028", "Anonymous functions cannot declare optional or params parameters.", syntax.Position);
        if (signature.Select(parameter => parameter.Name).Distinct(StringComparer.Ordinal).Count() != signature.Length || signature.Any(parameter => imports.ContainsKey(parameter.Name) || recordContext is not null && parameter.Name == "self"))
            throw source.Error("FP3004", "Lambda parameters must be unique and cannot shadow import aliases.", syntax.Position);
        var result = syntax.Result is null ? TypeSymbol.Void : BindType(syntax.Result);
        var symbol = new ClosureSymbol(catalog.ClosureCount++, recordContext, TypeParameters.Values.ToArray());
        closures.Add(symbol);
        var nested = new Binder(unit, catalog, references, this, symbol, recordContext) { arithmeticChecked = check };
        foreach (var import in imports) nested.imports.Add(import.Key, import.Value);
        var routine = new RoutineSymbol("<Invoke>", result, signature, false, false, -1, IsInstance: true, IsAsync: syntax.IsAsync);
        catalog.Analysis?.Parameters(signature, syntax.Parameters, source);
        var body = syntax.Body ?? [new ReturnSyntax(syntax.Expression!.Position, syntax.Expression)];
        symbol.Body = nested.BindRoutine(routine, body, syntax.Position, syntax.Contracts);
        var value = new BoundLambda(symbol, NaturalDelegate(expected, signature.Select(parameter => parameter.SignatureType).ToArray(), result, syntax.Position, signature));
        return expected is null ? value : Retarget(value, expected) ?? value;
    }

    private TypeSymbol NaturalDelegate(TypeSymbol? expected, IReadOnlyList<TypeSymbol> parameters, TypeSymbol result, int position, IReadOnlyList<ParameterSymbol>? info = null)
        => expected is not null && references.DelegateCompatible(expected, parameters, result, info) ? expected.NonNullable : references.Callable(parameters, result, source, position);

    private BoundExpression SourceDelegate(SourceUnit owner, string name, int position, TypeSymbol? expected, TypeList? arguments = null)
    {
        return RoutineDelegate(AccessibleSource(owner, name, position), expected, position, arguments);
    }

    private BoundDelegate? Retarget(BoundDelegate value, TypeSymbol target)
    {
        if (value is BoundMethodGroup group)
        {
            var method = references.DelegateMethod(target, group.Methods);
            if (method is not null) catalog.Analysis?.Retarget(group, method);
            return method is null ? null : new BoundMethodDelegate(method, method.IsStatic ? null : group.Receiver, target.NonNullable);
        }
        var (parameters, result) = DelegateShape(value);
        var info = value switch { BoundLambda lambda => lambda.Closure.Body.Symbol.Parameters, BoundMethodDelegate method => method.Method.ParameterInfo, _ => null };
        if (!references.DelegateCompatible(target, parameters, result, info)) return null;
        return value switch
        {
            BoundLambda lambda => new BoundLambda(lambda.Closure, target.NonNullable),
            BoundMethodDelegate method => new BoundMethodDelegate(method.Method, method.Receiver, target.NonNullable),
            _ => throw new InvalidOperationException("Unknown delegate creation.")
        };
    }

    private static (IReadOnlyList<TypeSymbol> Parameters, TypeSymbol Result) DelegateShape(BoundDelegate value) => value switch
    {
            BoundLambda lambda => (lambda.Closure.Body.Symbol.Parameters.Select(parameter => parameter.SignatureType).ToArray(), lambda.Closure.Body.Symbol.Result),
            BoundMethodDelegate method => (method.Method.DelegateParameters, method.Method.Result),
        _ => throw new InvalidOperationException("Unknown delegate creation.")
    };

    private bool Accepts(BoundExpression argument, TypeSymbol target) => argument switch
    {
        BoundArrayLiteral array when LiteralVector(target) is not null => AcceptsArrayLiteral(array, target),
        BoundDelegate value when references.DelegateSignature(target) is not null => Retarget(value, target) is not null,
        _ => argument is not BoundMethodGroup && (references.Applicable(argument.Type, target) || ConstantFits(argument, target))
    };

    private static bool IsMutableCapture(object symbol) => symbol is LocalSymbol { Mutable: true, Captured: true } || symbol is CaptureSymbol { Mutable: true } || symbol is ParameterSymbol { Mode: "ref" or "out" };

    private void ForgetCapturedFacts()
    {
        nonNull.RemoveWhere(IsMutableCapture);
    }
}

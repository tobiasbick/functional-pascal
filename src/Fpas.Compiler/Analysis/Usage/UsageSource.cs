// [Documentation](../../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

internal static class UsageSource
{
    internal static ApiUsage Call(UsageUnit unit, ReferenceCatalog catalog, ImportedType owner, ImportedMethod method, bool namedArguments)
    {
        bool constructor = method.Name == ".ctor";
        var target = unit.Receiver(owner, method.Owner, method.IsStatic || constructor, constructor);
        if (constructor)
            target = "new " + target;
        else if (method.Name != "Invoke" || method.IsStatic || catalog.DelegateSignature(method.Owner) is null)
            target += "." + FpasIdentifiers.SpellQualified(method.Name);
        if (method.MethodArguments is { Count: > 0 } typeArguments)
            target = FpasIdentifiers.ApplyTypeArguments(target, typeArguments.Select(unit.Names.Type).ToArray());

        var arguments = new List<string>();
        for (int i = 0; i < method.Parameters.Count; i++)
        {
            var parameter = method.Parameter(i);
            var mode = parameter.ByRef ? parameter.Mode : "";
            var input = unit.Input(parameter.Name, NamedCallArguments.InputType(method.Parameters[i]), mode);
            var value = mode.Length == 0 ? input : mode + " " + input;
            arguments.Add(namedArguments ? IdentifierFacts.Spell(parameter.Name) + " := " + value : value);
        }

        var result = constructor ? method.Owner : method.Result;
        bool taskResult = result.Definition?.FullName is "System.Threading.Tasks.Task" or "System.Threading.Tasks.Task`1"
            or "System.Threading.Tasks.ValueTask" or "System.Threading.Tasks.ValueTask`1";
        bool awaitCall = !constructor && taskResult && !unit.HasByRefInputs;
        if (awaitCall && result.IsNullable)
            return ApiUsage.Unavailable("The method can return a null awaitable. Decide how to handle absence before awaiting it.");
        if (awaitCall) result = result.TypeArguments.Count == 0 ? TypeSymbol.Void : result.TypeArguments[0];
        var hasResult = result != TypeSymbol.Void;
        var canScope = hasResult && !catalog.Derives(result, catalog.Symbol(catalog.Required("System.Threading.Tasks.Task")));
        var syncDispose = canScope && catalog.Derives(result, catalog.Symbol(catalog.Required("System.IDisposable")));
        var asyncDispose = canScope && !syncDispose && !unit.HasByRefInputs && catalog.Derives(result, catalog.Symbol(catalog.Required("System.IAsyncDisposable")));
        var isAsync = awaitCall || asyncDispose;
        var returnsValue = hasResult && !syncDispose && !asyncDispose;
        var returnType = hasResult ? unit.Names.Type(result) : null;
        var signature = returnsValue ? returnType : null;
        if (isAsync) signature = FpasIdentifiers.ApplyTypeArguments(unit.Names.Named(catalog.Required("System.Threading.Tasks.Task"), []),
            returnsValue ? [returnType!] : []);
        var call = (awaitCall ? "await " : "") + target + "(" + string.Join(", ", arguments) + ")";
        var statement = (returnsValue ? "return " : "") + call + ";";
        if (syncDispose || asyncDispose)
        {
            var binding = asyncDispose ? "await using const" : "using const";
            statement = $"// Use this scope when the API transfers ownership to the caller.\n  {binding} Resource: {returnType} := {call};";
        }
        return unit.Render(statement, signature, isAsync);
    }

    internal static ApiUsage Property(UsageUnit unit, ImportedType owner, string name, ImportedMethod accessor, bool write, TypeSymbol valueType)
    {
        var receiver = unit.Receiver(owner, accessor.Owner, accessor.IsStatic);
        var indexCount = accessor.Parameters.Count - (write ? 1 : 0);
        var indices = Enumerable.Range(0, indexCount).Select(i => unit.Input(accessor.Parameter(i).Name, accessor.Parameters[i]));
        var target = indexCount == 0 ? receiver + "." + FpasIdentifiers.SpellQualified(name) : receiver + "[" + string.Join(", ", indices) + "]";
        return Access(unit, target, valueType, write);
    }

    internal static ApiUsage Field(UsageUnit unit, ImportedType owner, ImportedField field, bool write)
    {
        var receiver = unit.Receiver(owner, field.Owner, field.IsStatic);
        return Access(unit, receiver + "." + FpasIdentifiers.SpellQualified(field.Name), field.Type, write);
    }

    private static ApiUsage Access(UsageUnit unit, string target, TypeSymbol valueType, bool write) => write
        ? unit.Render(target + " := " + unit.Input("value", valueType) + ";")
        : unit.Render("return " + target + ";", unit.Names.Type(valueType));
}

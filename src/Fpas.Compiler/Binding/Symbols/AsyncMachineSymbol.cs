using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Symbols;

internal sealed class AsyncMachineSymbol(BoundRoutine routine, ClosureSymbol? closure, int id, string ownerName)
{
    internal BoundRoutine Routine { get; } = routine;
    internal ClosureSymbol? Closure { get; } = closure;
    internal int Id { get; } = id;
    internal string OwnerName { get; } = ownerName;
    internal TypeSymbol? HostType => Closure?.Type ?? Routine.Symbol.Owner?.Type;
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; } = (closure?.GenericParameters ?? routine.Symbol.Owner?.GenericParameters ?? []).Concat(routine.Symbol.GenericParameters).ToArray();
    internal int ParentArity => Closure?.GenericParameters.Count ?? Routine.Symbol.Owner?.GenericParameters.Count ?? 0;
    internal string MetadataName => "<" + Routine.Symbol.Name + ">d__" + Id + (GenericParameters.Count == ParentArity ? "" : "`" + (GenericParameters.Count - ParentArity));
    internal TypeSymbol Type => new(TypeKind.Async, Arguments: GenericParameters.Count == 0 ? null : new(GenericParameters.Select(parameter => parameter.Type)), Machine: this);
    internal IReadOnlyDictionary<LocalSymbol, string> LocalNames { get; } = ChooseLocalNames(routine);

    private static Dictionary<LocalSymbol, string> ChooseLocalNames(BoundRoutine routine)
    {
        var used = routine.Symbol.Parameters.Select(parameter => parameter.Name).ToHashSet(StringComparer.Ordinal);
        if (routine.Symbol.IsInstance) used.Add("this");
        var names = new Dictionary<LocalSymbol, string>();
        foreach (var local in routine.Locals.Where(local => !local.Name.StartsWith('<')))
        {
            var name = local.Name;
            for (var suffix = 1; !used.Add(name); suffix++)
                name = local.Name + "_" + suffix.ToString(System.Globalization.CultureInfo.InvariantCulture);
            names.Add(local, name);
        }
        return names;
    }
}

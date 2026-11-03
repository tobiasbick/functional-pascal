using Fpas.Compiler.Lowering;

namespace Fpas.Compiler.Binding;

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
    internal AsyncPlan Plan { get; set; } = null!;
}

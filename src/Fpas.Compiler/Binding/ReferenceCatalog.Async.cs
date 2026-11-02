using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal TypeSymbol AsyncBuilder(AsyncMachineSymbol machine)
    {
        var result = machine.Routine.Symbol.BodyResult;
        return result == TypeSymbol.Void ? Symbol(Required("System.Runtime.CompilerServices.AsyncTaskMethodBuilder")) :
            Symbol(Required("System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1")) with { Arguments = new([result]) };
    }

    internal ImportedField MachineField(AsyncMachineSymbol machine, string name, TypeSymbol type)
        => new(machine.Type, name, type, false, false, SignatureType: SourceSignature(type, machine.GenericParameters));

    internal ImportedField MachineLocal(AsyncMachineSymbol machine, LocalSymbol local)
        => MachineField(machine, "<" + local.Name + ">5__" + (local.Slot + 1), local.Captured && local.Mutable ? CellType(local.Type) : local.Type);

    internal ImportedField MachineParameter(AsyncMachineSymbol machine, ParameterSymbol parameter)
        => parameter.Index < 0 ? MachineField(machine, "<>4__this", machine.HostType!) : MachineField(machine, parameter.Name, parameter.Type);

    internal BoundAwait Await(BoundExpression operand, SourceText source, int position)
    {
        var name = operand.Type.Definition?.FullName;
        if (name is not ("System.Threading.Tasks.Task" or "System.Threading.Tasks.Task`1" or "System.Threading.Tasks.ValueTask" or "System.Threading.Tasks.ValueTask`1" or
            "System.Runtime.CompilerServices.ConfiguredTaskAwaitable" or "System.Runtime.CompilerServices.ConfiguredTaskAwaitable`1" or
            "System.Runtime.CompilerServices.ConfiguredValueTaskAwaitable" or "System.Runtime.CompilerServices.ConfiguredValueTaskAwaitable`1"))
            throw source.Error("FP3030", "await supports Task, ValueTask, and their standard ConfigureAwait awaitables.", position);
        var getAwaiter = FindMethod(operand.Type, "GetAwaiter", [], position, source, false);
        var completed = Property(getAwaiter.Result, "IsCompleted", false)?.Getter ?? throw source.Error("FP3030", "Missing awaiter completion metadata.", position);
        return new(operand, getAwaiter, completed, FindMethod(getAwaiter.Result, "GetResult", [], position, source, false), source.StatementLocation(position));
    }
}

using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Emission.StateMachines;

internal static class AsyncMachineFields
{
    internal static TypeSymbol AsyncBuilder(this ReferenceCatalog catalog, AsyncMachineSymbol machine)
    {
        var result = machine.Routine.Symbol.BodyResult;
        return result == TypeSymbol.Void ? catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.AsyncTaskMethodBuilder")) :
            catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1")) with { Arguments = new([result]) };
    }

    internal static ImportedField MachineField(this ReferenceCatalog catalog, AsyncMachineSymbol machine, string name, TypeSymbol type)
        => new(machine.Type, name, type, false, false, SignatureType: catalog.SourceSignature(type, machine.GenericParameters));

    internal static ImportedField MachineLocal(this ReferenceCatalog catalog, AsyncMachineSymbol machine, LocalSymbol local)
        => catalog.MachineField(machine, (local.Name.StartsWith('<') ? "<>s__" : "<" + machine.LocalNames[local] + ">5__") + (local.Slot + 1),
            local.Captured && local.Mutable ? catalog.CellType(local.Type) : local.Type);

    internal static ImportedField MachineParameter(this ReferenceCatalog catalog, AsyncMachineSymbol machine, ParameterSymbol parameter)
        => parameter.Index < 0 ? catalog.MachineField(machine, "<>4__this", machine.HostType!) : catalog.MachineField(machine, parameter.Name, parameter.Type);
}

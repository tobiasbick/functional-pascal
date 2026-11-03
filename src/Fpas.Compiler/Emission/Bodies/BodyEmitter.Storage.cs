using Fpas.Compiler.Emission.StateMachines;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Bodies;

internal sealed partial class BodyEmitter
{
    private AsyncMachineSymbol? asyncMachine;
    private readonly Dictionary<LocalSymbol, int> asyncSlots = [];

    private bool Hoisted(LocalSymbol local) => asyncPlan?.Locals.Contains(local) == true && local.Type.Kind != TypeKind.ByRef;
    private int LocalSlot(LocalSymbol local) => asyncSlots.GetValueOrDefault(local, local.Slot);

    private void LoadStorage(LocalSymbol local)
    {
        if (Hoisted(local)) { il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.MachineLocal(asyncMachine!, local))); }
        else il.LoadLocal(LocalSlot(local));
    }

    private void StoreStorage(LocalSymbol local, Action value)
    {
        if (Hoisted(local))
        {
            il.LoadArgument(0); value(); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(catalog.MachineLocal(asyncMachine!, local)));
        }
        else { value(); il.StoreLocal(LocalSlot(local)); }
    }
}

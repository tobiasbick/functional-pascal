using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;
using Fpas.Compiler.Lowering;

namespace Fpas.Compiler.Emission;

internal sealed partial class BodyEmitter
{
    internal List<(int Yield, int Resume)> AwaitPoints { get; } = [];

    internal (InstructionEncoder Code, int MaxStack, IReadOnlyList<TypeSymbol> Locals) BuildAsync(AsyncMachineSymbol machine)
    {
        asyncMachine = machine;
        il = new(new BlobBuilder(), controlFlow);
        foreach (var local in machine.Plan.Locals.Where(local => local.Type.Kind == TypeKind.ByRef)) asyncSlots.Add(local, Temporary(local.Type));
        var self = Temporary(machine.Type);
        var exceptionType = catalog.Symbol(catalog.Required("System.Exception"));
        var exception = Temporary(exceptionType);
        var builder = catalog.AsyncBuilder(machine);
        var stateField = catalog.MachineField(machine, "<>1__state", TypeSymbol.Int32);
        var builderField = catalog.MachineField(machine, "<>t__builder", builder);
        var exceptionField = catalog.MachineField(machine, "<>exception", exceptionType);
        var loop = il.DefineLabel(); var start = il.DefineLabel(); var end = il.DefineLabel();
        var handler = il.DefineLabel(); var handlerEnd = il.DefineLabel();
        var finished = il.DefineLabel(); var failed = il.DefineLabel(); var returnedAsync = il.DefineLabel();
        var labels = machine.Plan.Blocks.Select(_ => il.DefineLabel()).ToArray();
        var offsets = new int[labels.Length];
        var yields = new List<(int Offset, int Resume)>();
        void State(int target) { DebugPoint(null); il.LoadArgument(0); il.LoadConstantI4(target); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(stateField)); }
        void Go(int target) { State(target); il.Branch(ILOpCode.Leave, loop); }
        il.LoadArgument(0); il.StoreLocal(self);
        il.MarkLabel(loop); DebugPoint(null); il.OpCode(ILOpCode.Nop);
        il.MarkLabel(start);
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(stateField));
        var dispatch = il.Switch(labels.Length);
        foreach (var label in labels) dispatch.Branch(label);
        il.Branch(ILOpCode.Leave, returnedAsync);
        foreach (var block in machine.Plan.Blocks.AsEnumerable().Reverse())
        {
            il.MarkLabel(labels[block.Id]); offsets[block.Id] = il.Offset;
            DebugPoint(block.Location);
            foreach (var statement in block.Statements) EmitStatement(statement);
            switch (block.Exit)
            {
                case AsyncGoto jump: Go(jump.Target); break;
                case AsyncBranch branch:
                    var no = il.DefineLabel(); Evaluate(branch.Condition); il.Branch(ILOpCode.Brfalse, no);
                    Go(branch.True); il.MarkLabel(no); Go(branch.False); break;
                case AsyncRaise raised: Evaluate(raised.Exception); il.OpCode(ILOpCode.Throw); break;
                case AsyncPropagate propagated:
                    il.LoadArgument(0); Evaluate(propagated.Exception); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(exceptionField)); Go(propagated.Target); break;
                case AsyncComplete: State(-2); il.Branch(ILOpCode.Leave, finished); break;
                case AsyncFail: State(-2); il.Branch(ILOpCode.Leave, failed); break;
                case AsyncSuspend awaited:
                    EmitCall(awaited.IsCompleted, [], new BoundLocal(awaited.Awaiter));
                    var incomplete = il.DefineLabel(); il.Branch(ILOpCode.Brfalse, incomplete); Go(awaited.Resume); il.MarkLabel(incomplete);
                    State(awaited.Resume);
                    yields.Add((il.Offset, awaited.Resume)); il.OpCode(ILOpCode.Nop);
                    il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(builderField));
                    EmitAddress(new BoundLocal(awaited.Awaiter), force: true);
                    il.LoadLocalAddress(self);
                    var schedule = catalog.FindMethod(builder, "AwaitUnsafeOnCompleted", [awaited.Awaiter.Type, machine.Type], 0, machine.Routine.Source!, false,
                        methodArguments: new([awaited.Awaiter.Type, machine.Type]));
                    il.Call(reference(schedule));
                    il.Branch(ILOpCode.Leave, returnedAsync);
                    break;
            }
        }
        il.MarkLabel(end); il.MarkLabel(handler);
        il.StoreLocal(exception); il.LoadArgument(0); il.LoadLocal(exception); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(exceptionField));
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(stateField));
        var exceptionLabels = machine.Plan.Blocks.Select(_ => il.DefineLabel()).ToArray();
        var handlers = il.Switch(exceptionLabels.Length); foreach (var label in exceptionLabels) handlers.Branch(label);
        il.Branch(ILOpCode.Leave, failed);
        foreach (var block in machine.Plan.Blocks)
        {
            il.MarkLabel(exceptionLabels[block.Id]);
            if (block.OnException < 0) il.Branch(ILOpCode.Leave, failed); else Go(block.OnException);
        }
        il.MarkLabel(handlerEnd);
        il.MarkLabel(finished);
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(builderField));
        if (machine.Plan.Result is not null) LoadStorage(machine.Plan.Result);
        var setResult = catalog.FindMethod(builder, "SetResult", machine.Routine.Symbol.BodyResult == TypeSymbol.Void ? [] : [machine.Routine.Symbol.BodyResult], 0, machine.Routine.Source!, false);
        il.Call(reference(setResult)); il.Branch(ILOpCode.Br, returnedAsync);
        il.MarkLabel(failed);
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(builderField));
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(exceptionField));
        il.Call(reference(catalog.FindMethod(builder, "SetException", [exceptionType], 0, machine.Routine.Source!, false)));
        il.MarkLabel(returnedAsync); il.OpCode(ILOpCode.Ret);
        controlFlow.AddCatchRegion(start, end, handler, handlerEnd, typeHandle(exceptionType));
        foreach (var point in yields) AwaitPoints.Add((point.Offset, offsets[point.Resume]));
        return (il, Math.Max(8, maxStack), localTypes);
    }
}

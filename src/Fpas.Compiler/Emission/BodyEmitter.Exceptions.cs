using System.Reflection.Metadata;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class BodyEmitter
{
    private void EmitFinally(Action body, Action cleanup)
    {
        var start = il.DefineLabel(); var end = il.DefineLabel(); var handler = il.DefineLabel();
        var handlerEnd = il.DefineLabel(); var after = il.DefineLabel();
        il.MarkLabel(start); il.OpCode(ILOpCode.Nop);
        protectedDepth++; body(); il.Branch(ILOpCode.Leave, after); protectedDepth--;
        il.MarkLabel(end); il.MarkLabel(handler);
        protectedDepth++; cleanup(); il.OpCode(ILOpCode.Endfinally); protectedDepth--;
        il.MarkLabel(handlerEnd); il.MarkLabel(after);
        controlFlow.AddFinallyRegion(start, end, handler, handlerEnd);
    }

    private void EmitCatches(BoundTry guarded)
    {
        if (guarded.Catches.Count == 0) { EmitStatement(guarded.Body); return; }
        var start = il.DefineLabel(); var end = il.DefineLabel(); var after = il.DefineLabel();
        il.MarkLabel(start); il.OpCode(ILOpCode.Nop);
        protectedDepth++; EmitStatement(guarded.Body); il.Branch(ILOpCode.Leave, after); protectedDepth--;
        il.MarkLabel(end);
        foreach (var caught in guarded.Catches)
        {
            var handler = il.DefineLabel(); var handlerEnd = il.DefineLabel();
            var filter = il.DefineLabel();
            if (caught.Filter is not null)
            {
                il.MarkLabel(filter);
                il.OpCode(ILOpCode.Isinst); il.Token(typeHandle(caught.ExceptionType)); il.OpCode(ILOpCode.Dup);
                var matches = il.DefineLabel(); var filterEnd = il.DefineLabel();
                il.Branch(ILOpCode.Brtrue, matches); il.OpCode(ILOpCode.Pop); il.LoadConstantI4(0); il.Branch(ILOpCode.Br, filterEnd);
                il.MarkLabel(matches);
                if (caught.Local is null) il.OpCode(ILOpCode.Pop); else il.StoreLocal(caught.Local.Slot);
                Evaluate(caught.Filter);
                il.MarkLabel(filterEnd); il.OpCode(ILOpCode.Endfilter);
            }
            il.MarkLabel(handler); protectedDepth++;
            if (caught.Local is null || caught.Filter is not null) il.OpCode(ILOpCode.Pop); else il.StoreLocal(caught.Local.Slot);
            EmitStatement(caught.Body); il.Branch(ILOpCode.Leave, after);
            protectedDepth--; il.MarkLabel(handlerEnd);
            if (caught.Filter is null) controlFlow.AddCatchRegion(start, end, handler, handlerEnd, typeHandle(caught.ExceptionType));
            else controlFlow.AddFilterRegion(start, end, handler, handlerEnd, filter);
        }
        il.MarkLabel(after);
    }

    private void EmitDispose(BoundUsing resource)
    {
        var done = il.DefineLabel();
        var local = new BoundLocal(resource.Local);
        if (resource.ConditionalDispose)
        {
            EmitExpression(local); il.OpCode(ILOpCode.Isinst); il.Token(typeHandle(resource.Dispose.Owner));
            var slot = Temporary(resource.Dispose.Owner);
            il.StoreLocal(slot); local = new BoundLocal(new("<disposable>", resource.Dispose.Owner, slot));
        }
        if (local.Type.CanHaveNull || local.Type.NullableValue) { EmitHasValue(local); il.Branch(ILOpCode.Brfalse, done); }
        if (local.Type.NullableValue)
        {
            var slot = Temporary(local.Type.NonNullable);
            EmitCall(NullableMethod(local.Type, "GetValueOrDefault"), [], local); il.StoreLocal(slot);
            local = new BoundLocal(new("<dispose>", local.Type.NonNullable, slot));
        }
        if (local.Type.IsValue || local.Type.IsParameter)
        {
            EmitAddress(local, force: true);
            il.OpCode(ILOpCode.Constrained); il.Token(typeHandle(local.Type));
        }
        else EmitExpression(local);
        il.OpCode(ILOpCode.Callvirt); il.Token(reference(resource.Dispose));
        il.MarkLabel(done);
        il.OpCode(ILOpCode.Nop);
    }
}

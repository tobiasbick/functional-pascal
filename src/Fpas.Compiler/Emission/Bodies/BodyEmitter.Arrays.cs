using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Bodies;

internal sealed partial class BodyEmitter
{
    private static ImportedMethod ArrayMethod(TypeSymbol array, string name)
        => new(array, name, name is "Get" or "Address" ? array.Element! : TypeSymbol.Void,
            Enumerable.Repeat(TypeSymbol.Int32, array.Rank).Concat(name == "Set" ? [array.Element!] : Array.Empty<TypeSymbol>()).ToArray(),
            IsStatic: false, ReturnsByRef: name == "Address");

    private void EmitNewArray(BoundNewArray array)
    {
        foreach (var length in array.Lengths) EmitExpression(length);
        if (array.Type.Rank == 1) { il.OpCode(ILOpCode.Newarr); il.Token(typeHandle(array.Type.Element!)); }
        else { il.OpCode(ILOpCode.Newobj); il.Token(reference(ArrayMethod(array.Type, ".ctor"))); }
    }

    private void EmitArrayLiteral(BoundArrayLiteral array)
    {
        il.LoadConstantI4(array.Elements.Count); il.OpCode(ILOpCode.Newarr); il.Token(typeHandle(array.Type.Element!));
        for (var index = 0; index < array.Elements.Count; index++)
        {
            il.OpCode(ILOpCode.Dup); il.LoadConstantI4(index); EmitExpression(array.Elements[index]);
            il.OpCode(ILOpCode.Stelem); il.Token(typeHandle(array.Type.Element!));
        }
    }

    private void EmitArrayOperands(BoundArrayIndex index)
    {
        EmitExpression(index.Receiver);
        foreach (var coordinate in index.Indices) EmitExpression(coordinate);
    }

    private void EmitArrayIndex(BoundArrayIndex index)
    {
        EmitArrayOperands(index);
        if (index.Receiver.Type.Rank == 1) { il.OpCode(ILOpCode.Ldelem); il.Token(typeHandle(index.Type)); }
        else il.Call(reference(ArrayMethod(index.Receiver.Type, "Get")));
    }

    private void EmitArrayAddress(BoundArrayIndex index)
    {
        EmitArrayOperands(index);
        if (index.Receiver.Type.Rank == 1) { il.OpCode(ILOpCode.Ldelema); il.Token(typeHandle(index.Type)); }
        else il.Call(reference(ArrayMethod(index.Receiver.Type, "Address")));
    }

    private void EmitArrayStore(BoundArrayIndex index, BoundExpression value)
    {
        maxStack = Math.Max(maxStack, index.Indices.Count + 1 + ExpressionStack.Depth(value));
        EmitArrayOperands(index); EmitExpression(value);
        if (index.Receiver.Type.Rank == 1) { il.OpCode(ILOpCode.Stelem); il.Token(typeHandle(index.Type)); }
        else il.Call(reference(ArrayMethod(index.Receiver.Type, "Set")));
    }
}

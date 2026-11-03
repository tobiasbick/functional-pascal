using Fpas.Compiler.Emission.StateMachines;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Bodies;

internal sealed partial class BodyEmitter
{
    private void EmitCaptureStorage(CaptureSymbol capture)
    {
        il.LoadArgument(0);
        if (asyncMachine is not null) { il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.MachineField(asyncMachine, "<>4__this", asyncMachine.HostType!))); }
        il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.CaptureField(capture)));
    }

    private void EmitCapture(BoundCapture capture)
    {
        EmitCaptureStorage(capture.Capture);
        if (capture.Capture.Mutable) { il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.CellValue(capture.Capture.Type))); }
    }

    private void EmitDelegate(BoundDelegate value)
    {
        switch (value)
        {
            case BoundLambda lambda:
                foreach (var capture in lambda.Closure.Captures) EmitExpression(capture.Value);
                il.OpCode(ILOpCode.Newobj); il.Token(reference(catalog.ClosureMethod(lambda.Closure, constructor: true)));
                il.OpCode(ILOpCode.Ldftn); il.Token(reference(catalog.ClosureMethod(lambda.Closure, constructor: false)));
                break;
            case BoundMethodDelegate method:
                if (method.Receiver is null) il.OpCode(ILOpCode.Ldnull);
                else
                {
                    EmitExpression(method.Receiver);
                    if (method.Receiver.Type.IsValue || method.Receiver.Type.IsParameter) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(method.Receiver.Type)); }
                }
                if (method.Receiver is not null && method.Method.IsVirtual)
                {
                    il.OpCode(ILOpCode.Dup); il.OpCode(ILOpCode.Ldvirtftn); il.Token(reference(method.Method));
                }
                else
                {
                    if (method.Method.ConstrainedType is { } implementor) { il.OpCode(ILOpCode.Constrained); il.Token(typeHandle(implementor)); }
                    il.OpCode(ILOpCode.Ldftn); il.Token(reference(method.Method));
                }
                break;
        }
        var constructor = new ImportedMethod(value.Type, ".ctor", TypeSymbol.Void, [TypeSymbol.Object, TypeSymbol.NativeInt], false);
        il.OpCode(ILOpCode.Newobj); il.Token(reference(constructor));
    }
}

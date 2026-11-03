using Fpas.Compiler.Emission.StateMachines;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private int EmitAsyncStart(AsyncMachineSymbol machine, MethodBodyStreamEncoder bodies)
    {
        var il = new InstructionEncoder(new BlobBuilder());
        var builder = catalog.AsyncBuilder(machine);
        var builderField = catalog.MachineField(machine, "<>t__builder", builder);
        il.OpCode(ILOpCode.Newobj); il.Token(Reference(new ImportedMethod(machine.Type, ".ctor", TypeSymbol.Void, [], false)));
        il.StoreLocal(0);
        il.LoadLocal(0);
        il.Call(Reference(catalog.FindMethod(builder, "Create", [], 0, machine.Routine.Source!, true)));
        il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(builderField));
        if (machine.Routine.Symbol.IsInstance)
        {
            il.LoadLocal(0); il.LoadArgument(0);
            il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(catalog.MachineField(machine, "<>4__this", machine.HostType!)));
        }
        foreach (var parameter in machine.Routine.Symbol.Parameters)
        {
            il.LoadLocal(0); il.LoadArgument(parameter.Index + (machine.Routine.Symbol.IsInstance ? 1 : 0));
            il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(catalog.MachineParameter(machine, parameter)));
        }
        il.LoadLocal(0); il.LoadConstantI4(layout.AsyncPlans[machine].Entry);
        il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(catalog.MachineField(machine, "<>1__state", TypeSymbol.Int32)));
        il.LoadLocal(0); il.OpCode(ILOpCode.Ldflda); il.Token(FieldReference(builderField));
        il.LoadLocalAddress(0);
        il.Call(Reference(catalog.FindMethod(builder, "Start", [machine.Type], 0, machine.Routine.Source!, false, methodArguments: new([machine.Type]))));
        il.LoadLocal(0); il.OpCode(ILOpCode.Ldflda); il.Token(FieldReference(builderField));
        il.Call(Reference(catalog.Property(builder, "Task", false)!.Getter!));
        il.OpCode(ILOpCode.Ret);
        return bodies.AddMethodBody(il, 3, LocalSignature([machine.Type]));
    }

    private void EmitEntryBridge(BoundRoutine routine, MethodDefinitionHandle handle, MethodBodyStreamEncoder bodies,
        ref int parameterCount, AssemblyIdentity runtime)
    {
        var il = new InstructionEncoder(new BlobBuilder());
        foreach (var parameter in routine.Symbol.Parameters) il.LoadArgument(parameter.Index);
        il.Call(layout.SourceMethods[routine.Symbol]);
        var awaitable = catalog.Await(new BoundLocal(new("<task>", routine.Symbol.Result, 0)), routine.Source!, 0);
        il.Call(Reference(awaitable.GetAwaiter)); il.StoreLocal(0); il.LoadLocalAddress(0);
        il.Call(Reference(awaitable.GetResult)); il.OpCode(ILOpCode.Ret);
        metadata.AddMethodDefinition(MethodAttributes.Private | MethodAttributes.Static | MethodAttributes.HideBySig,
            MethodImplAttributes.IL, metadata.GetOrAddString("<Main>"),
            MethodSignature(TypeSymbol.Void, routine.Symbol.Parameters.Select(parameter => parameter.Type).ToArray()),
            bodies.AddMethodBody(il, 2, LocalSignature([awaitable.GetAwaiter.Result])),
            AddParameters(routine.Symbol with { Name = "<Main>", Result = TypeSymbol.Void }, ref parameterCount, runtime));
    }

    private StandaloneSignatureHandle LocalSignature(IReadOnlyList<TypeSymbol> locals)
    {
        if (locals.Count == 0) return default;
        var blob = new BlobBuilder();
        var signature = new BlobEncoder(blob).LocalVariableSignature(locals.Count);
        foreach (var type in locals)
            EncodeType(signature.AddVariable().Type(isByRef: type.Kind == TypeKind.ByRef), type.Kind == TypeKind.ByRef ? type.Element! : type);
        return metadata.AddStandaloneSignature(metadata.GetOrAddBlob(blob));
    }

    private void AddAsyncAttribute(MethodDefinitionHandle method, AsyncMachineSymbol machine)
    {
        var attribute = catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.AsyncStateMachineAttribute"));
        var constructor = Reference(new ImportedMethod(attribute, ".ctor", TypeSymbol.Void, [catalog.Symbol(catalog.Required("System.Type"))], false));
        var blob = new BlobBuilder(); blob.WriteUInt16(1);
        blob.WriteSerializedString(machine.OwnerName + "+" + machine.MetadataName); blob.WriteUInt16(0);
        metadata.AddCustomAttribute(method, constructor, metadata.GetOrAddBlob(blob));
    }
}

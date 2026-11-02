using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class AssemblyEmitter
{
    private void EmitAsyncMachines(MethodBodyStreamEncoder bodies, ref int parameterCount, TypeReferenceHandle objectType, AssemblyIdentity runtime)
    {
        var fieldCount = layout.Records.Sum(record => record.Fields.Count) + layout.Closures.Sum(closure => closure.Captures.Count);
        var stateMachine = catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.IAsyncStateMachine"));
        foreach (var machine in layout.Machines)
        {
            genericTypeContext = machine.GenericParameters;
            var type = metadata.AddTypeDefinition(TypeAttributes.NestedPrivate | TypeAttributes.Sealed | TypeAttributes.BeforeFieldInit,
                default, metadata.GetOrAddString(machine.MetadataName), objectType,
                MetadataTokens.FieldDefinitionHandle(fieldCount + 1), layout.AsyncConstructors[machine]);
            metadata.AddNestedType(type, layout.AsyncOwners[machine]);
            metadata.AddInterfaceImplementation(type, TypeHandle(stateMachine));
            RegisterGenerics(type, machine.GenericParameters, genericTypeContext);
            AddNullabilityContext(type, runtime);
            var generated = catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.CompilerGeneratedAttribute"));
            metadata.AddCustomAttribute(type, Reference(new ImportedMethod(generated, ".ctor", TypeSymbol.Void, [], false)), metadata.GetOrAddBlob(new byte[] { 1, 0, 0, 0 }));
            void Field(ImportedField field)
            {
                var blob = new BlobBuilder(); EncodeType(new BlobEncoder(blob).FieldSignature(), field.Type);
                metadata.AddFieldDefinition(FieldAttributes.Assembly, metadata.GetOrAddString(field.Name), metadata.GetOrAddBlob(blob)); fieldCount++;
            }
            var builder = catalog.AsyncBuilder(machine);
            var builderField = catalog.MachineField(machine, "<>t__builder", builder);
            Field(catalog.MachineField(machine, "<>1__state", TypeSymbol.Int32)); Field(builderField);
            Field(catalog.MachineField(machine, "<>exception", catalog.Symbol(catalog.Required("System.Exception"))));
            if (machine.Routine.Symbol.IsInstance) Field(catalog.MachineField(machine, "<>4__this", machine.HostType!));
            foreach (var parameter in machine.Routine.Symbol.Parameters) Field(catalog.MachineParameter(machine, parameter));
            foreach (var local in machine.Plan.Locals.Where(local => local.Type.Kind != TypeKind.ByRef)) Field(catalog.MachineLocal(machine, local));

            var il = new InstructionEncoder(new BlobBuilder());
            il.LoadArgument(0); il.Call(Reference(new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false))); il.OpCode(ILOpCode.Ret);
            metadata.AddMethodDefinition(MethodAttributes.Assembly | MethodAttributes.HideBySig | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName,
                MethodImplAttributes.IL, metadata.GetOrAddString(".ctor"), MethodSignature(TypeSymbol.Void, [], instance: true),
                bodies.AddMethodBody(il, 1), MetadataTokens.ParameterHandle(parameterCount + 1));

            var emitter = new BodyEmitter(Reference, FieldReference, TypeHandle, catalog, metadata.GetOrAddUserString, debug is not null);
            var body = emitter.BuildAsync(machine);
            var signature = LocalSignature(body.Locals);
            var attributes = MethodAttributes.Public | MethodAttributes.Final | MethodAttributes.Virtual | MethodAttributes.NewSlot | MethodAttributes.HideBySig;
            var move = metadata.AddMethodDefinition(attributes, MethodImplAttributes.IL, metadata.GetOrAddString("MoveNext"),
                MethodSignature(TypeSymbol.Void, [], instance: true), bodies.AddMethodBody(body.Code, body.MaxStack, signature), MetadataTokens.ParameterHandle(parameterCount + 1));
            debug?.AddMethod(move, emitter.SequencePoints, [], signature, body.Code.Offset);
            var kickoff = machine.Closure is { } closure ? layout.ClosureInvocations[closure] : layout.SourceMethods[machine.Routine.Symbol];
            debug?.AddAsync(move, kickoff, emitter.AwaitPoints, machine.Plan.Locals, body.Code.Offset);

            il = new(new BlobBuilder());
            il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(FieldReference(builderField)); il.LoadArgument(1);
            il.Call(Reference(catalog.FindMethod(builder, "SetStateMachine", [stateMachine], 0, machine.Routine.Source!, false))); il.OpCode(ILOpCode.Ret);
            var setter = new RoutineSymbol("SetStateMachine", TypeSymbol.Void, [new("stateMachine", stateMachine, 0)], false, false, -1, IsInstance: true);
            metadata.AddMethodDefinition(attributes, MethodImplAttributes.IL, metadata.GetOrAddString(setter.Name), MethodSignature(TypeSymbol.Void, [stateMachine], instance: true),
                bodies.AddMethodBody(il, 2), AddParameters(setter, ref parameterCount, runtime));
        }
        genericTypeContext = null;
    }
}

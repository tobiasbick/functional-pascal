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
    private ParameterHandle AddParameters(RoutineSymbol routine, ref int count, AssemblyIdentity runtime)
    {
        var first = MetadataTokens.ParameterHandle(count + 1);
        if (NullableFlags(routine.Result).Contains((byte)2))
        {
            var parameter = metadata.AddParameter(ParameterAttributes.None, default, 0);
            AddNullableAnnotation(parameter, runtime, routine.Result); count++;
        }
        foreach (var parameter in routine.Parameters)
        {
            var attributes = parameter.Mode == "out" ? ParameterAttributes.Out : parameter.Mode == "in" ? ParameterAttributes.In : ParameterAttributes.None;
            if (parameter.Default is not null) attributes |= ParameterAttributes.Optional | (parameter.Type.NonNullable == TypeSymbol.Decimal ? ParameterAttributes.None : ParameterAttributes.HasDefault);
            var handle = metadata.AddParameter(attributes, metadata.GetOrAddString(parameter.Name), parameter.Index + 1);
            if (parameter.Default is not null)
            {
                var value = ConstantValue.Evaluate(parameter.Default);
                if (value is decimal number) AddDecimalConstant(handle, number);
                else metadata.AddConstant(handle, value switch { nint signed => (object)checked((int)signed), nuint unsigned => checked((uint)unsigned), _ => value });
            }
            if (parameter.Mode == "in") AddParameterMarker(handle, "System.Runtime.CompilerServices.IsReadOnlyAttribute");
            if (parameter.Mode == "params") AddParameterMarker(handle, "System.ParamArrayAttribute");
            if (NullableFlags(parameter.Type).Contains((byte)2)) AddNullableAnnotation(handle, runtime, parameter.Type);
            count++;
        }
        return first;
    }

    private void EmitClosures(BoundCompilation program, MethodBodyStreamEncoder bodies, ref int parameterCount, TypeReferenceHandle objectType)
    {
        var fieldCount = metadata.GetRowCount(TableIndex.Field);
        foreach (var closure in layout.Closures)
        {
            genericTypeContext = closure.GenericParameters.Count == 0 ? null : closure.GenericParameters;
            var arity = closure.GenericParameters.Count - (closure.RecordContext?.GenericParameters.Count ?? 0);
            var type = metadata.AddTypeDefinition(TypeAttributes.NestedPrivate | TypeAttributes.Sealed,
                default, metadata.GetOrAddString("<Closure" + closure.Id + ">" + (arity == 0 ? "" : "`" + arity)), objectType,
                MetadataTokens.FieldDefinitionHandle(fieldCount + 1), layout.ClosureConstructors[closure]);
            RegisterGenerics(type, closure.GenericParameters, genericTypeContext);
            metadata.AddNestedType(type, layout.ClosureOwners[closure]);
            AddNullabilityContext(type, program.Runtime);
            AddDebugAttribute(type, GeneratedAttribute);
            AddDebuggerDisplay(type, "function(" + string.Join(", ", closure.Captures.Select(capture => capture.FieldName + " = {" + capture.FieldName + (capture.Mutable ? ".Value" : "") + "}")) + ")");
            foreach (var capture in closure.Captures)
            {
                var signature = new BlobBuilder();
                EncodeType(new BlobEncoder(signature).FieldSignature(), CaptureStorage(capture));
                var captureField = metadata.AddFieldDefinition(FieldAttributes.Assembly | FieldAttributes.InitOnly,
                    metadata.GetOrAddString(capture.FieldName), metadata.GetOrAddBlob(signature));
                AddGeneratedField(captureField, hidden: false);
                debug?.AddStorage(captureField, capture.Original switch
                {
                    LocalSymbol local => local.Name, ParameterSymbol parameter => parameter.Name,
                    _ => throw new InvalidOperationException("A capture must refer to a source binding.")
                }, capture.Mutable);
                fieldCount++;
            }
            var constructor = new RoutineSymbol(".ctor", TypeSymbol.Void,
                closure.Captures.Select((capture, index) => new ParameterSymbol("capture" + index, CaptureStorage(capture), index)).ToArray(),
                false, false, -1, IsInstance: true);
            var firstParameter = AddParameters(constructor, ref parameterCount, program.Runtime);
            var constructorHandle = metadata.AddMethodDefinition(MethodAttributes.Assembly | MethodAttributes.HideBySig | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName,
                MethodImplAttributes.IL, metadata.GetOrAddString(".ctor"), MethodSignature(TypeSymbol.Void, constructor.Parameters.Select(parameter => parameter.Type).ToArray(), instance: true),
                EmitClosureConstructor(closure, bodies), firstParameter);
            AddGeneratedMethod(constructorHandle, hidden: true);
            foreach (var routine in closure.Bodies)
            {
                firstParameter = AddParameters(routine.Symbol, ref parameterCount, program.Runtime);
                var invocation = metadata.AddMethodDefinition(MethodAttributes.Assembly | MethodAttributes.HideBySig, SourceMethodFlags(routine.Attributes),
                    metadata.GetOrAddString(routine.Symbol.Name), MethodSignature(routine.Symbol.Result,
                        routine.Symbol.Parameters.Select(parameter => parameter.SignatureType).ToArray(), instance: true, genericArity: routine.Symbol.GenericParameters.Count),
                    EmitBody(routine, bodies), firstParameter);
                RegisterGenerics(invocation, routine.Symbol.GenericParameters, genericTypeContext);
                if (routine.Symbol.IsAsync) AddAsyncAttribute(invocation, layout.AsyncRoutines[routine], routine.Attributes);
                AddSourceAttributes(invocation, routine.Attributes);
            }
        }
        genericTypeContext = null;
    }

    private TypeSymbol CaptureStorage(CaptureSymbol capture) => capture.Mutable ? catalog.CellType(capture.Type) : capture.Type;

    private void AddParameterMarker(ParameterHandle parameter, string name)
    {
        var attribute = catalog.Symbol(catalog.Required(name));
        var constructor = Reference(new ImportedMethod(attribute, ".ctor", TypeSymbol.Void, [], false));
        metadata.AddCustomAttribute(parameter, constructor, metadata.GetOrAddBlob(new byte[] { 1, 0, 0, 0 }));
    }

    private int EmitClosureConstructor(ClosureSymbol closure, MethodBodyStreamEncoder bodies)
    {
        var il = new InstructionEncoder(new BlobBuilder());
        il.LoadArgument(0);
        il.Call(Reference(new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false)));
        for (var index = 0; index < closure.Captures.Count; index++)
        {
            il.LoadArgument(0); il.LoadArgument(index + 1);
            il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(catalog.CaptureField(closure.Captures[index])));
        }
        il.OpCode(ILOpCode.Ret);
        return bodies.AddMethodBody(il, maxStack: 2);
    }
}

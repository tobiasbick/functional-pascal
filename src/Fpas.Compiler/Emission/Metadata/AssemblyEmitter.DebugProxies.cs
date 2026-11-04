using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void EmitDebuggerProxies(MethodBodyStreamEncoder bodies, ref int parameterCount, TypeReferenceHandle objectType, AssemblyIdentity runtime)
    {
        foreach (var record in layout.Records)
        {
            if (record.Attributes.Concat(record.UnionCase?.Owner.Attributes ?? []).Any(attribute => AttributeIs(attribute, ProxyAttribute))) continue;
            genericTypeContext = record.GenericParameters.Count == 0 ? null : record.GenericParameters;
            var proxy = metadata.AddTypeDefinition(TypeAttributes.NestedPrivate | TypeAttributes.Sealed,
                default, metadata.GetOrAddString("<DebugView>"), objectType,
                MetadataTokens.FieldDefinitionHandle(metadata.GetRowCount(TableIndex.Field) + 1),
                MetadataTokens.MethodDefinitionHandle(metadata.GetRowCount(TableIndex.MethodDef) + 1));
            metadata.AddNestedType(proxy, layout.RecordTypes[record]);
            RegisterGenerics(proxy, record.GenericParameters, genericTypeContext);
            AddDebugAttribute(proxy, GeneratedAttribute);
            AddNullabilityContext(proxy, runtime);

            var signature = new BlobBuilder(); EncodeType(new BlobEncoder(signature).FieldSignature(), record.Type);
            var field = metadata.AddFieldDefinition(FieldAttributes.Private | FieldAttributes.InitOnly,
                metadata.GetOrAddString("_value"), metadata.GetOrAddBlob(signature));
            AddGeneratedField(field);
            EntityHandle receiver = field;
            if (record.GenericParameters.Count != 0)
            {
                var self = new BlobBuilder();
                var arguments = new BlobEncoder(self).TypeSpecificationSignature().GenericInstantiation(proxy, record.GenericParameters.Count, false);
                for (var index = 0; index < record.GenericParameters.Count; index++) arguments.AddArgument().GenericTypeParameter(index);
                receiver = metadata.AddMemberReference(metadata.AddTypeSpecification(metadata.GetOrAddBlob(self)),
                    metadata.GetOrAddString("_value"), metadata.GetOrAddBlob(signature));
            }

            var constructor = new RoutineSymbol(".ctor", TypeSymbol.Void, [new("value", record.Type, 0)], false, false, -1, IsInstance: true);
            var firstParameter = AddParameters(constructor, ref parameterCount, runtime);
            var il = new InstructionEncoder(new BlobBuilder());
            il.LoadArgument(0); il.Call(Reference(new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false)));
            il.LoadArgument(0); il.LoadArgument(1); il.OpCode(ILOpCode.Stfld); il.Token(receiver); il.OpCode(ILOpCode.Ret);
            var method = metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.HideBySig | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName,
                MethodImplAttributes.IL, metadata.GetOrAddString(".ctor"), MethodSignature(TypeSymbol.Void, [record.Type], instance: true),
                bodies.AddMethodBody(il, 2), firstParameter);
            AddGeneratedMethod(method, hidden: true);

            PropertyDefinitionHandle firstProperty = default;
            foreach (var payload in record.Fields)
            {
                var property = EmitDebuggerProperty(payload.Name, payload.Type, record.Method(payload.Getter), receiver, bodies, parameterCount, runtime);
                if (firstProperty.IsNil) firstProperty = property;
                AddSourceAttributes(property, payload.Attributes.Where(attribute => AttributeIs(attribute, "System.Diagnostics.DebuggerBrowsableAttribute")).ToArray());
            }
            if (record.UnionCase is not null)
            {
                var tag = record.Method(record.Methods.Single(candidate => candidate.Generated == GeneratedMethodKind.Tag));
                var property = EmitDebuggerProperty("Tag", TypeSymbol.Int32, tag, receiver, bodies, parameterCount, runtime);
                if (firstProperty.IsNil) firstProperty = property;
            }
            if (!firstProperty.IsNil) metadata.AddPropertyMap(proxy, firstProperty);

            var parent = record.UnionCase is { } member ? member.Owner.Name + Arity(member.Owner.GenericParameters.Count) + "+" + record.Name :
                record.Name + Arity(record.GenericParameters.Count);
            var name = record.Owner.Syntax.Name.Text + "+" + parent + "+<DebugView>, " + sourceAssemblyName;
            AddDebugAttribute(layout.RecordTypes[record], ProxyAttribute, TypeSymbol.String, blob => blob.WriteSerializedString(name));
        }
        genericTypeContext = null;
    }

    private PropertyDefinitionHandle EmitDebuggerProperty(string name, TypeSymbol type, ImportedMethod source, EntityHandle receiver,
        MethodBodyStreamEncoder bodies, int parameterCount, AssemblyIdentity runtime)
    {
        var il = new InstructionEncoder(new BlobBuilder());
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(receiver); il.Call(Reference(source)); il.OpCode(ILOpCode.Ret);
        var method = metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.HideBySig | MethodAttributes.SpecialName,
            MethodImplAttributes.IL, metadata.GetOrAddString("get_" + name), MethodSignature(type, [], instance: true),
            bodies.AddMethodBody(il, 1), MetadataTokens.ParameterHandle(parameterCount + 1));
        AddGeneratedMethod(method, hidden: true);
        return AddReadOnlyProperty(name, type, true, method, runtime);
    }

    private static string Arity(int count) => count == 0 ? "" : "`" + count;
}

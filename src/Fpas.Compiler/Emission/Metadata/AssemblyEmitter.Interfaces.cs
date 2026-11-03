using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void EmitInterfaces(AssemblyIdentity runtime, ref int parameterCount)
    {
        foreach (var contract in layout.Interfaces)
        {
            genericTypeContext = contract.GenericParameters.Count == 0 ? null : contract.GenericParameters;
            var name = contract.Name + (contract.GenericParameters.Count == 0 ? "" : "`" + contract.GenericParameters.Count);
            var type = metadata.AddTypeDefinition(TypeAttributes.Interface | TypeAttributes.Abstract |
                (contract.IsPublic ? TypeAttributes.NestedPublic : TypeAttributes.NestedPrivate), default, metadata.GetOrAddString(name), default,
                MetadataTokens.FieldDefinitionHandle(metadata.GetRowCount(TableIndex.Field) + 1),
                MetadataTokens.MethodDefinitionHandle(metadata.GetRowCount(TableIndex.MethodDef) + 1));
            metadata.AddNestedType(type, layout.InterfaceOwners[contract]);
            RegisterGenerics(type, contract.GenericParameters, genericTypeContext);
            AddNullabilityContext(type, runtime);
            AddSourceAttributes(type, contract.Attributes);
            foreach (var parent in contract.Bases) metadata.AddInterfaceImplementation(type, TypeHandle(parent));
            foreach (var method in contract.Methods)
            {
                var first = AddParameters(method, ref parameterCount, runtime);
                var flags = MethodAttributes.Public | MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot | MethodAttributes.HideBySig;
                if (method.Generated == GeneratedMethodKind.Getter) flags |= MethodAttributes.SpecialName;
                var handle = metadata.AddMethodDefinition(flags, MethodImplAttributes.IL, metadata.GetOrAddString(method.Name),
                    MethodSignature(method.Result, SourceSignatureParameters(method), instance: true, genericArity: method.GenericParameters.Count), -1, first);
                RegisterGenerics(handle, method.GenericParameters, genericTypeContext);
                AddSourceAttributes(handle, contract.MethodAttributes.GetValueOrDefault(method));
            }
            PropertyDefinitionHandle firstProperty = default;
            foreach (var property in contract.Properties)
            {
                var signature = new BlobBuilder();
                new BlobEncoder(signature).PropertySignature(isInstanceProperty: true).Parameters(0,
                    result => EncodeType(result.Type(), property.Getter.Result), _ => { });
                var handle = metadata.AddProperty(PropertyAttributes.None, metadata.GetOrAddString(property.Syntax.Name.Text), metadata.GetOrAddBlob(signature));
                if (firstProperty.IsNil) firstProperty = handle;
                metadata.AddMethodSemantics(handle, MethodSemanticsAttributes.Getter, layout.SourceMethods[property.Getter]);
                if (NullableFlags(property.Getter.Result).Contains((byte)2)) AddNullableAnnotation(handle, runtime, property.Getter.Result);
                AddSourceAttributes(handle, property.Attributes);
            }
            if (!firstProperty.IsNil) metadata.AddPropertyMap(type, firstProperty);
        }
        genericTypeContext = null;
    }
}

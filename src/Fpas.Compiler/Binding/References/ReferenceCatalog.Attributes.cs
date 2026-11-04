using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal bool AttributeType(TypeSymbol type, bool vector = true)
    {
        if (type.NullableValue || type.IsParameter) return false;
        if (type.Kind == TypeKind.Array) return vector && type.Rank == 1 && AttributeType(type.Element!, vector: false);
        if (type.Kind is TypeKind.Boolean or TypeKind.Char or TypeKind.SByte or TypeKind.Byte or TypeKind.Int16 or TypeKind.UInt16 or
            TypeKind.Int32 or TypeKind.UInt32 or TypeKind.Int64 or TypeKind.UInt64 or TypeKind.Single or TypeKind.Double or TypeKind.String or TypeKind.Object) return true;
        return type.Definition?.FullName == "System.Type" || ScalarInfo(type)?.Category == ScalarCategory.Enum;
    }

    internal (AttributeTargets Targets, bool Multiple) AttributeUsage(TypeSymbol type, SourceText source, int position)
    {
        if (type.IsNullable || type.Definition is null || type.Definition.Definition.Attributes.HasFlag(TypeAttributes.Abstract) ||
            !Derives(type, Symbol(Required("System.Attribute"))))
            throw source.Error("FP3035", "An attribute requires an accessible, concrete CLR attribute class.", position);
        var name = type.Definition.FullName;
        if (type.Definition.Assembly.Identity.Name == "Fpas.Core" || name is
            "System.Runtime.CompilerServices.NullableAttribute" or "System.Runtime.CompilerServices.NullableContextAttribute" or
            "System.Runtime.InteropServices.DllImportAttribute" or "System.Runtime.InteropServices.StructLayoutAttribute" or
            "System.Runtime.InteropServices.FieldOffsetAttribute" or "System.Runtime.InteropServices.MarshalAsAttribute" or
            "System.Runtime.InteropServices.ComImportAttribute" or "System.Runtime.InteropServices.PreserveSigAttribute" or "System.NonSerializedAttribute")
            throw source.Error("FP3035", "Compiler-owned or structural CLR attributes cannot be authored as ordinary custom attributes.", position);
        foreach (var owner in Hierarchy(type))
        {
            var definition = Definition(owner); var reader = definition.Assembly.Reader;
            foreach (var handle in definition.Definition.GetCustomAttributes())
            {
                var attribute = reader.GetCustomAttribute(handle);
                if (MetadataAttributes.Name(reader, attribute) != "System.AttributeUsageAttribute") continue;
                var blob = reader.GetBlobReader(attribute.Value);
                if (blob.ReadUInt16() != 1) throw new BadImageFormatException("Invalid AttributeUsage metadata.");
                var targets = (AttributeTargets)blob.ReadInt32(); var multiple = false;
                var count = blob.ReadUInt16();
                for (var index = 0; index < count; index++)
                {
                    if (blob.ReadByte() != 0x54 || blob.ReadByte() != 0x02) throw new BadImageFormatException("Invalid AttributeUsage property metadata.");
                    var property = blob.ReadSerializedString(); var value = blob.ReadByte() != 0;
                    if (property == "AllowMultiple") multiple = value;
                    else if (property != "Inherited") throw new BadImageFormatException("Unknown AttributeUsage property.");
                }
                if (blob.RemainingBytes != 0) throw new BadImageFormatException("Trailing AttributeUsage metadata.");
                return (targets, multiple);
            }
        }
        return (AttributeTargets.All, false);
    }

    internal (TypeSymbol Type, bool Field, TypeSymbol Owner) AttributeMember(TypeSymbol owner, string name, SourceText source, int position)
    {
        foreach (var type in Hierarchy(owner))
        {
            var definition = Definition(type); var reader = definition.Assembly.Reader;
            foreach (var handle in definition.Definition.GetFields())
            {
                var field = reader.GetFieldDefinition(handle);
                if (reader.GetString(field.Name) != name) continue;
                if ((field.Attributes & FieldAttributes.FieldAccessMask) == FieldAttributes.Public &&
                    !field.Attributes.HasFlag(FieldAttributes.Static) && !field.Attributes.HasFlag(FieldAttributes.InitOnly) && !field.Attributes.HasFlag(FieldAttributes.Literal) &&
                    Field(type, name, false) is { } imported && AttributeType(imported.Type)) return (imported.Type, true, type);
                throw source.Error("FP3035", "A named attribute field must be public, writable, instance, and have a legal attribute type.", position);
            }
            foreach (var handle in definition.Definition.GetProperties())
            {
                var property = reader.GetPropertyDefinition(handle);
                if (reader.GetString(property.Name) != name) continue;
                var setter = property.GetAccessors().Setter;
                if (!setter.IsNil && (reader.GetMethodDefinition(setter).Attributes & MethodAttributes.MemberAccessMask) == MethodAttributes.Public &&
                    ReadMethod(type, setter) is { IsStatic: false, Result.Kind: TypeKind.Void, Parameters.Count: 1 } method &&
                    !method.Parameter(0).ByRef && AttributeType(method.Parameters[0])) return (method.Parameters[0], false, type);
                throw source.Error("FP3035", "A named attribute property requires a public instance setter, no indexes, and a legal attribute type.", position);
            }
        }
        throw source.Error("FP3035", $"No writable attribute field/property '{name}'.", position);
    }
}

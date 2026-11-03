using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private string sourceAssemblyName = "";

    private void AddSourceAttributes(EntityHandle target, IReadOnlyList<BoundAttribute>? attributes)
    {
        foreach (var attribute in attributes ?? [])
        {
            if (AttributeIs(attribute, "System.SerializableAttribute") || AttributeIs(attribute, "System.Runtime.CompilerServices.MethodImplAttribute")) continue;
            var blob = new BlobBuilder(); blob.WriteUInt16(1);
            for (var index = 0; index < attribute.Arguments.Count; index++)
                WriteAttributeValue(blob, attribute.Constructor.Parameters[index], attribute.Arguments[index]);
            blob.WriteUInt16(checked((ushort)attribute.Named.Count));
            foreach (var named in attribute.Named)
            {
                blob.WriteByte(named.IsField ? (byte)0x53 : (byte)0x54);
                WriteAttributeType(blob, named.Type);
                blob.WriteSerializedString(named.Name);
                WriteAttributeValue(blob, named.Type, named.Value);
            }
            metadata.AddCustomAttribute(target, Reference(attribute.Constructor), metadata.GetOrAddBlob(blob));
        }
    }

    private void WriteAttributeType(BlobBuilder blob, TypeSymbol type)
    {
        if (catalog.ScalarInfo(type) is { Category: Syntax.ScalarCategory.Enum })
        {
            blob.WriteByte(0x55); blob.WriteSerializedString(AttributeTypeName(type)); return;
        }
        if (type.Kind == TypeKind.Array)
        {
            blob.WriteByte(0x1d); WriteAttributeType(blob, type.Element!); return;
        }
        blob.WriteByte((byte)(type.Definition?.FullName == "System.Type" ? 0x50 : type.Kind switch
        {
            TypeKind.Boolean => 0x02, TypeKind.Char => 0x03, TypeKind.SByte => 0x04, TypeKind.Byte => 0x05,
            TypeKind.Int16 => 0x06, TypeKind.UInt16 => 0x07, TypeKind.Int32 => 0x08, TypeKind.UInt32 => 0x09,
            TypeKind.Int64 => 0x0a, TypeKind.UInt64 => 0x0b, TypeKind.Single => 0x0c, TypeKind.Double => 0x0d,
            TypeKind.String => 0x0e, TypeKind.Object => 0x51,
            _ => throw new InvalidOperationException("Unsupported attribute field type.")
        }));
    }

    private void WriteAttributeValue(BlobBuilder blob, TypeSymbol declared, AttributeValue value)
    {
        if (declared.Kind == TypeKind.Object)
        {
            WriteAttributeType(blob, value.Type); WriteAttributeValue(blob, value.Type, value); return;
        }
        if (declared.Kind == TypeKind.Array)
        {
            if (value.Value is not AttributeValue[] elements) { blob.WriteInt32(-1); return; }
            blob.WriteInt32(elements.Length);
            foreach (var element in elements) WriteAttributeValue(blob, declared.Element!, element);
            return;
        }
        if (catalog.ScalarInfo(declared) is { Category: Syntax.ScalarCategory.Enum } enumeration)
        {
            WriteAttributeValue(blob, enumeration.Base, value); return;
        }
        if (declared.Definition?.FullName == "System.Type")
        {
            blob.WriteSerializedString(value.Value is TypeSymbol type ? AttributeTypeName(type) : null); return;
        }
        switch (declared.Kind)
        {
            case TypeKind.Boolean: blob.WriteBoolean((bool)value.Value!); break;
            case TypeKind.Char: blob.WriteUInt16((char)value.Value!); break;
            case TypeKind.SByte: blob.WriteSByte(Convert.ToSByte(value.Value)); break;
            case TypeKind.Byte: blob.WriteByte(Convert.ToByte(value.Value)); break;
            case TypeKind.Int16: blob.WriteInt16(Convert.ToInt16(value.Value)); break;
            case TypeKind.UInt16: blob.WriteUInt16(Convert.ToUInt16(value.Value)); break;
            case TypeKind.Int32: blob.WriteInt32(Convert.ToInt32(value.Value)); break;
            case TypeKind.UInt32: blob.WriteUInt32(Convert.ToUInt32(value.Value)); break;
            case TypeKind.Int64: blob.WriteInt64(Convert.ToInt64(value.Value)); break;
            case TypeKind.UInt64: blob.WriteUInt64(Convert.ToUInt64(value.Value)); break;
            case TypeKind.Single: blob.WriteSingle(Convert.ToSingle(value.Value)); break;
            case TypeKind.Double: blob.WriteDouble(Convert.ToDouble(value.Value)); break;
            case TypeKind.String: blob.WriteSerializedString((string?)value.Value); break;
            default: throw new InvalidOperationException("Unsupported attribute value.");
        }
    }

    private string AttributeTypeName(TypeSymbol type)
    {
        (string Name, string Assembly) Describe(TypeSymbol current)
        {
            if (current.NullableValue) current = catalog.Symbol(catalog.Required("System.Nullable`1")) with { Arguments = new([current.NonNullable]) };
            if (current.Kind == TypeKind.Array)
            {
                var element = Describe(current.Element!);
                return (element.Name + "[" + new string(',', current.Rank - 1) + "]", element.Assembly);
            }
            var name = current.Record is { } record ? record.Owner.Syntax.Name.Text + "+" +
                (record.UnionCase is { } member ? member.Owner.Name + Arity(member.Owner.GenericParameters.Count) + "+" + record.Name : record.Name + Arity(record.GenericParameters.Count)) :
                current.Union is { } union ? union.Owner.Syntax.Name.Text + "+" + union.Name + Arity(union.GenericParameters.Count) :
                current.Scalar is { } scalar ? scalar.Owner.Syntax.Name.Text + "+" + scalar.Name : catalog.Definition(current).FullName;
            var assembly = current.Record is not null || current.Union is not null || current.Scalar is not null ? sourceAssemblyName : catalog.Definition(current).Assembly.Identity.Name;
            if (current.Arguments is { Count: > 0 }) name += "[[" + string.Join("],[", current.Arguments.Select(AttributeTypeName)) + "]]";
            return (name, assembly);
        }
        static string Arity(int count) => count == 0 ? "" : "`" + count;
        var result = Describe(type);
        return result.Name + ", " + result.Assembly;
    }
}

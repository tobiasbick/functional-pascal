using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal static class MetadataConstants
{
    internal static object? Read(MetadataReader reader, ConstantHandle handle)
    {
        var constant = reader.GetConstant(handle); var blob = reader.GetBlobReader(constant.Value);
        return constant.TypeCode switch
        {
            ConstantTypeCode.SByte => blob.ReadSByte(), ConstantTypeCode.Byte => blob.ReadByte(),
            ConstantTypeCode.Int16 => blob.ReadInt16(), ConstantTypeCode.UInt16 => blob.ReadUInt16(),
            ConstantTypeCode.Int32 => blob.ReadInt32(), ConstantTypeCode.UInt32 => blob.ReadUInt32(),
            ConstantTypeCode.Int64 => blob.ReadInt64(), ConstantTypeCode.UInt64 => blob.ReadUInt64(),
            ConstantTypeCode.Single => blob.ReadSingle(), ConstantTypeCode.Double => blob.ReadDouble(),
            ConstantTypeCode.Boolean => blob.ReadBoolean(), ConstantTypeCode.Char => (char)blob.ReadUInt16(),
            ConstantTypeCode.String => blob.ReadUTF16(blob.Length), ConstantTypeCode.NullReference => null,
            _ => throw new BadImageFormatException("Unsupported constant type.")
        };
    }

    internal static decimal? Decimal(MetadataReader reader, CustomAttributeHandleCollection attributes)
    {
        foreach (var handle in attributes)
        {
            var attribute = reader.GetCustomAttribute(handle);
            if (MetadataAttributes.Name(reader, attribute) != "System.Runtime.CompilerServices.DecimalConstantAttribute") continue;
            var blob = reader.GetBlobReader(attribute.Value);
            if (blob.ReadUInt16() != 1) throw new BadImageFormatException("Invalid decimal constant metadata.");
            var scale = blob.ReadByte(); var sign = blob.ReadByte();
            var high = blob.ReadInt32(); var middle = blob.ReadInt32(); var low = blob.ReadInt32();
            if (scale > 28 || blob.ReadUInt16() != 0 || blob.RemainingBytes != 0) throw new BadImageFormatException("Invalid decimal constant metadata.");
            return new decimal(low, middle, high, sign != 0, scale);
        }
        return null;
    }
}

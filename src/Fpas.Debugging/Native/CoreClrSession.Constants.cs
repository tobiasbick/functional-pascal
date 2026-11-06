using System.Reflection.Metadata;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static DebugBinding ConstantBinding(string name, MetadataImage metadata, FieldDefinition field)
    {
        try
        {
            var handle = field.GetDefaultValue();
            if (handle.IsNil) throw new BadImageFormatException();
            var constant = metadata.Metadata.GetConstant(handle); var blob = metadata.Metadata.GetBlobReader(constant.Value);
            if (constant.TypeCode == ConstantTypeCode.NullReference) return new(name, "object", "null", null);
            if (constant.TypeCode == ConstantTypeCode.String)
            {
                var length = blob.Length;
                if ((length & 1) != 0) throw new BadImageFormatException();
                var bytes = blob.ReadBytes(Math.Min(length, 16384));
                var text = StoredText.Read(MemoryMarshal.Cast<byte, char>(bytes), length / 2);
                return new(name, "string", "string", text.Text, text.Status, text.Truncated, text.RawUtf16);
            }
            var kind = (uint)constant.TypeCode;
            if (ScalarValue.Size(kind) is not (> 0 and var size) || blob.Length != size) throw new BadImageFormatException();
            return ScalarValue.Read(name, kind, blob.ReadBytes(size));
        }
        catch (BadImageFormatException)
        { return new(name, "[unavailable]", "unknown", null, "unavailable", Error: new("invalid_metadata", "The literal's stored metadata is invalid or unsupported.")); }
    }
}

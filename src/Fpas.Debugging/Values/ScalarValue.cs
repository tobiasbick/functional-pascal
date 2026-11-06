using System.Buffers.Binary;
using System.Globalization;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Values;

internal static class ScalarValue
{
    internal static string? TypeName(uint kind) => kind switch
    {
        2 => "bool", 3 => "char", 4 => "sbyte", 5 => "byte", 6 => "short", 7 => "ushort", 8 => "int", 9 => "uint",
        10 => "long", 11 => "ulong", 12 => "float", 13 => "double", 14 => "string", 24 => "nint", 25 => "nuint", 28 => "object", _ => null
    };

    internal static int Size(uint kind) => kind switch { 2 or 4 or 5 => 1, 3 or 6 or 7 => 2, 8 or 9 or 12 => 4, 10 or 11 or 13 or 24 or 25 => 8, _ => 0 };

    internal static DebugBinding Read(string name, uint kind, ReadOnlySpan<byte> bytes)
    {
        var type = TypeName(kind) ?? throw new ArgumentException("Unsupported scalar kind.");
        if (bytes.Length != Size(kind)) throw new ArgumentException("Invalid scalar storage size.");
        var format = CultureInfo.InvariantCulture;
        if (kind == 3)
        {
            var character = (char)BinaryPrimitives.ReadUInt16LittleEndian(bytes);
            return new(name, type, "character", char.IsSurrogate(character) ? "\uFFFD" : character.ToString(),
                RawUtf16: char.IsSurrogate(character) ? ((ushort)character).ToString("X4", format) : null);
        }
        var text = kind switch
        {
            2 => bytes[0] == 0 ? "false" : "true",
            4 => unchecked((sbyte)bytes[0]).ToString(format), 5 => bytes[0].ToString(format),
            6 => BinaryPrimitives.ReadInt16LittleEndian(bytes).ToString(format), 7 => BinaryPrimitives.ReadUInt16LittleEndian(bytes).ToString(format),
            8 => BinaryPrimitives.ReadInt32LittleEndian(bytes).ToString(format), 9 => BinaryPrimitives.ReadUInt32LittleEndian(bytes).ToString(format),
            10 or 24 => BinaryPrimitives.ReadInt64LittleEndian(bytes).ToString(format), 11 or 25 => BinaryPrimitives.ReadUInt64LittleEndian(bytes).ToString(format),
            12 => BinaryPrimitives.ReadSingleLittleEndian(bytes).ToString("R", format),
            13 => BinaryPrimitives.ReadDoubleLittleEndian(bytes).ToString("R", format), _ => throw new ArgumentException("Unsupported scalar kind.")
        };
        return new(name, type, kind == 2 ? "boolean" : kind is 12 or 13 ? "floating" : "integer", text);
    }
}

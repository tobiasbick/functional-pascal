using System.Globalization;
using System.Numerics;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static DebugBinding DecodeDomain(DebugBinding storage, SourceShape shape)
    {
        if (storage.Status != "available") return storage;
        if (shape.Lower is null)
        {
            var expected = shape.Primitive switch
            {
                "System.Boolean" => "bool", "System.Char" => "char", "System.SByte" => "sbyte", "System.Byte" => "byte", "System.Int16" => "short", "System.UInt16" => "ushort",
                "System.Int32" => "int", "System.UInt32" => "uint", "System.Int64" => "long", "System.UInt64" => "ulong", "System.IntPtr" => "nint",
                "System.UIntPtr" => "nuint", "System.Single" => "float", "System.Double" => "double", "System.Decimal" => "decimal", "System.String" => "string", _ => null
            };
            return expected is not null && storage.Type == expected ? storage : Invalid();
        }
        var kind = shape.Primitive switch
        {
            "System.SByte" => 4u, "System.Byte" => 5u, "System.Int16" => 6u, "System.UInt16" => 7u, "System.Int32" => 8u,
            "System.UInt32" => 9u, "System.Int64" => 10u, "System.UInt64" => 11u, _ => 0u
        };
        if (kind == 0 || storage.Type != "ulong" || !ulong.TryParse(storage.Scalar, CultureInfo.InvariantCulture, out var offset)
            || !BigInteger.TryParse(shape.Lower, CultureInfo.InvariantCulture, out var lower) || !BigInteger.TryParse(shape.Upper, CultureInfo.InvariantCulture, out var upper)) return Invalid();
        var width = ScalarValue.Size(kind); var modulus = BigInteger.One << (width * 8); var signed = kind is 4 or 6 or 8 or 10;
        var minimum = signed ? -modulus / 2 : BigInteger.Zero; var maximum = signed ? modulus / 2 - 1 : modulus - 1;
        var number = lower + offset;
        if (lower < minimum || upper > maximum || lower > upper || number < lower || number > upper) return Invalid();
        var unsigned = (number % modulus + modulus) % modulus;
        var bytes = new byte[width];
        if (!unsigned.TryWriteBytes(bytes, out _, isUnsigned: true, isBigEndian: false)) return Invalid();
        return ScalarValue.Read(storage.Name, kind, bytes) with { RuntimeType = new(ScalarValue.TypeName(kind)!, kind) };

        DebugBinding Invalid() => new(storage.Name, "[unavailable]", "unknown", null, "unavailable", Error: new("invalid_metadata", "The encoded domain value or its bounds are invalid."));
    }
}

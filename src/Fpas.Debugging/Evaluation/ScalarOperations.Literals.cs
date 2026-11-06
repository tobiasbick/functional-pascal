using System.Globalization;
using System.Numerics;
using Fpas.Debugging.Core;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Evaluation;

internal static partial class ScalarOperations
{
    internal static DebugBinding Literal(ExpressionToken token, bool negative = false)
    {
        var text = token.Text;
        if (text.StartsWith('\'')) return Text(text[1..]) with { Constant = true };
        if (text is "true" or "false") return Boolean(text == "true") with { Constant = true };
        if (text == "null") return new("", "null", "null", null, Constant: true);
        text = text.Replace("_", "", StringComparison.Ordinal);
        if (text.StartsWith('#'))
        {
            if (!ushort.TryParse(text.AsSpan(1), NumberStyles.None, CultureInfo.InvariantCulture, out var code)) throw Error("A character literal must fit one UTF-16 code unit.");
            return Integer(3, code) with { Constant = true };
        }
        var hex = text.StartsWith('$'); var suffix = char.ToLowerInvariant(text[^1]);
        var real = !hex && (text.Contains('.') || text.Contains('e', StringComparison.OrdinalIgnoreCase) || suffix is 'd' or 'f' or 'm');
        if (real)
        {
            var content = suffix is 'd' or 'f' or 'm' ? text[..^1] : text;
            if (negative) content = "-" + content;
            if (suffix == 'm' && decimal.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var money)) return Decimal(money) with { Constant = true };
            if (suffix == 'f' && float.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var single) && float.IsFinite(single)) return Floating(12, single) with { Constant = true };
            if (suffix is not ('m' or 'f') && double.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var number) && double.IsFinite(number)) return Floating(13, number) with { Constant = true };
            throw Error("The floating or decimal literal is outside its finite representation.");
        }
        var end = text.Length;
        while (end > 0 && char.ToLowerInvariant(text[end - 1]) is 'l' or 'u') end--;
        var kind = text[end..].ToLowerInvariant() switch { "" => 8u, "l" => 10u, "u" => 9u, "ul" or "lu" => 11u, _ => throw Error("Invalid integral suffix.") };
        var digits = text[(hex ? 1 : 0)..end];
        if (!BigInteger.TryParse(hex ? "0" + digits : digits, hex ? NumberStyles.AllowHexSpecifier : NumberStyles.None, CultureInfo.InvariantCulture, out var integer)) throw Error("Invalid numeric literal.");
        return Integer(kind, negative ? -integer : integer) with { Constant = true };
    }

    internal static DebugBinding Boolean(bool value) => new("", "bool", "boolean", value ? "true" : "false");
    internal static DebugBinding Text(string value)
    {
        var text = StoredText.Read(value.AsSpan(), value.Length);
        return new("", "string", "string", text.Text, text.Status, text.Truncated, text.RawUtf16);
    }
    private static DebugBinding Decimal(decimal value) => new("", "decimal", "decimal", value.ToString(CultureInfo.InvariantCulture));
    private static DebugBinding Floating(uint kind, double value)
    {
        var bytes = kind == 12 ? BitConverter.GetBytes((float)value) : BitConverter.GetBytes(value);
        return ScalarValue.Read("", kind, bytes);
    }
    private static DebugBinding Integer(uint kind, BigInteger value, bool check = true)
    {
        var width = ScalarValue.Size(kind); var modulus = BigInteger.One << (width * 8);
        if (check && !Fits(value, kind)) throw new DebugException("overflow", "The integral result is outside its type's range.");
        var unsigned = (value % modulus + modulus) % modulus; var bytes = new byte[width];
        if (!unsigned.TryWriteBytes(bytes, out _, isUnsigned: true, isBigEndian: false)) throw new DebugException("overflow", "The integral result is outside its type's range.");
        return ScalarValue.Read("", kind, bytes);
    }
    internal static string StoredString(DebugBinding value)
    {
        if (value.RawUtf16 is null) return value.Scalar ?? "";
        var text = new char[value.RawUtf16.Length / 4];
        for (var index = 0; index < text.Length; index++) text[index] = (char)ushort.Parse(value.RawUtf16.AsSpan(index * 4, 4), NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture);
        return new(text);
    }
    private static DebugException Error(string message) => new("unsupported_expression", message);
}

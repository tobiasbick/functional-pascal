using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
using System.Globalization;
using System.Numerics;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression NumberLiteral(Token token, bool negative = false)
    {
        var text = token.Text.Replace("_", "", StringComparison.Ordinal);
        if (token.Kind == TokenKind.Integer)
        {
            var hex = text.StartsWith('$'); var end = text.Length;
            while (end > 0 && char.ToLowerInvariant(text[end - 1]) is 'u' or 'l') end--;
            var suffix = text[end..].ToLowerInvariant(); var digits = text[(hex ? 1 : 0)..end];
            var type = suffix switch { "l" => TypeSymbol.Int64, "u" => TypeSymbol.UInt32, "ul" or "lu" => TypeSymbol.UInt64, _ => TypeSymbol.Int32 };
            var value = BigInteger.Parse(hex ? "0" + digits : digits, hex ? NumberStyles.AllowHexSpecifier : NumberStyles.None, CultureInfo.InvariantCulture);
            if (negative) value = -value;
            if (!NumericFacts.Fits(value, type)) throw source.Error("FP3002", "The integral literal is outside its selected type's range; use an appropriate suffix.", token.Position, token.Length);
            return BoundNumber.Create(type, NumericValue.ConvertTo(value, type, true));
        }
        var suffixReal = char.ToLowerInvariant(text[^1]);
        var content = suffixReal is 'd' or 'f' or 'm' ? text[..^1] : text;
        if (negative) content = "-" + content;
        if (suffixReal == 'm' && decimal.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var money)) return new BoundNumber(TypeSymbol.Decimal, money);
        if (suffixReal == 'f' && float.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var single) && float.IsFinite(single)) return new BoundNumber(TypeSymbol.Single, single);
        if (suffixReal is not ('m' or 'f') && double.TryParse(content, NumberStyles.Float, CultureInfo.InvariantCulture, out var real) && double.IsFinite(real)) return new BoundDouble(real);
        throw source.Error("FP3002", "The floating/decimal literal is outside its finite representation.", token.Position, token.Length);
    }

    private static bool ConstantFits(BoundExpression value, TypeSymbol target)
    {
        if (!NumericFacts.Integral(value.Type) || !NumericFacts.Integral(target.NonNullable) || !ConstantValue.IsConstant(value)) return false;
        if (target.NonNullable == TypeSymbol.Char || value.Type != TypeSymbol.Int32 && !(value.Type == TypeSymbol.Int64 && target.NonNullable == TypeSymbol.UInt64)) return false;
        try { return NumericFacts.Fits(NumericValue.Integer(ConstantValue.Evaluate(value)!), target.NonNullable); }
        catch (Exception error) when (error is OverflowException or DivideByZeroException or InvalidOperationException) { return false; }
    }

    private BoundExpression NumericBinary(BoundExpression left, string op, BoundExpression right, int position, bool check)
    {
        var shift = op is "shl" or "shr";
        if (!shift)
        {
            if (left.Type.Kind is TypeKind.UInt32 or TypeKind.UInt64 or TypeKind.NativeUInt && ConstantFits(right, left.Type))
                right = Convert(right, left.Type, position);
            else if (right.Type.Kind is TypeKind.UInt32 or TypeKind.UInt64 or TypeKind.NativeUInt && ConstantFits(left, right.Type))
                left = Convert(left, right.Type, position);
        }
        var promoted = shift ? NumericFacts.Unary(left.Type, "+") : NumericFacts.Promote(left.Type, right.Type);
        var valid = op is "+" or "-" or "*" or "=" or "<>" or "<" or ">" or "<=" or ">=" ||
            op == "/" && (NumericFacts.Floating(promoted) || promoted == TypeSymbol.Decimal) ||
            op is "div" or "mod" or "shl" or "shr" or "and" or "or" or "xor" && NumericFacts.Integral(promoted);
        if (promoted == TypeSymbol.Unsupported || !valid) throw source.Error("FP3002", "The numeric operands cannot be promoted for this operator.", position);
        return new BoundBinary(Convert(left, promoted, position), op, Convert(right, shift ? TypeSymbol.Int32 : promoted, position),
            op is "=" or "<>" or "<" or ">" or "<=" or ">=" ? TypeSymbol.Boolean : promoted, check);
    }
}

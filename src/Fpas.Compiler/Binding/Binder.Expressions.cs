using System.Globalization;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundExpression BindExpression(ExpressionSyntax expression, bool check = true, TypeSymbol? expected = null)
    {
        check &= arithmeticChecked;
        if (bindingContract && expression is CallSyntax or NewSyntax or NewArraySyntax or ArrayLiteralSyntax or LambdaSyntax or WithSyntax or ArgumentModeSyntax or AwaitSyntax)
            throw source.Error("FP3027", "Calls, construction, and lambdas are forbidden in contract conditions.", expression.Position);
        switch (expression)
        {
            case AwaitSyntax awaited: return BindAwait(awaited);
            case ArgumentModeSyntax argument: return BindAddress(argument, check);
            case LiteralSyntax literal:
                return BindLiteral(literal);
            case ParenthesizedSyntax parenthesized:
                return BindExpression(parenthesized.Expression, check, expected);
            case NameExpressionSyntax name:
                return Value(name.Name.Text, name.Position, expected);
            case LambdaSyntax lambda:
                return BindLambda(lambda, expected, check);
            case GenericNameSyntax generic:
                return BindMethodGroup(generic.Name, new(generic.Arguments.Select(BindType)), expected);
            case MemberSyntax member:
                return BindMember(member, expected: expected, check: check);
            case CallSyntax call:
                return BindCall(call, check, expected);
            case NewArraySyntax array: return BindNewArray(array, check);
            case ArrayLiteralSyntax literal: return BindArrayLiteral(literal, check, expected);
            case IndexSyntax index: return BindIndex(index, check);
            case CaseExpressionSyntax match: return BindCaseExpression(match, check, expected);
            case TypeTestSyntax test: return BindTypeTest(test);
            case WithSyntax updated: return BindWith(updated, check);
            case NamedArgumentSyntax named: throw source.Error("FP3022", "Named arguments are currently supported only for records and union cases.", named.Position);
            case NewSyntax creation:
                var created = BindType(creation.Type);
                if (references.RecordInfo(created) is { } recordInfo) return BindRecordNew(creation, recordInfo, check);
                if (created.IsParameter && !created.NullableValue)
                {
                    if (!created.Parameter!.HasConstructor && !created.IsValue) throw source.Error("FP3018", "new T() requires new() or struct.", creation.Position);
                    if (created.IsNullable || creation.Arguments.Count != 0) throw source.Error("FP3026", "Generic parameter construction supports new T() only.", creation.Position);
                    var activator = references.Symbol(references.Required("System.Activator"));
                    return new BoundCall(references.FindMethod(activator, "CreateInstance", [], creation.Position, source, methodArguments: new([created])), []);
                }
                if (created.Kind == TypeKind.Union || created.IsNullable || created.Definition?.Definition.Attributes.HasFlag(System.Reflection.TypeAttributes.Abstract) == true)
                    throw source.Error("FP3005", "Cannot instantiate a nullable, abstract, or static type.", creation.Position);
                var constructors = references.Methods(created, ".ctor", false);
                if (created.IsValue && creation.Arguments.Count == 0 && !constructors.Any(method => method.Parameters.Count == 0)) return new BoundNew(created, null, []);
                return BindInvocation(constructors, creation.Arguments, null, check, creation.Position, created);
            case UncheckedSyntax wrapping:
                return BindExpression(wrapping.Operand, check: false, expected: expected);
            case CastSyntax cast:
                var targetType = BindType(cast.Type);
                return Convert(BindExpression(cast.Operand, check, targetType), targetType, cast.Position, explicitCast: true, check: check);
            case UnarySyntax unary:
                if (unary.Operator.Text == "-" && unary.Operand is LiteralSyntax { Token.Kind: TokenKind.Integer } number &&
                    number.Token.Text.Replace("_", "", StringComparison.Ordinal) == "2147483648")
                    return new BoundInteger(int.MinValue);
                var operand = BindExpression(unary.Operand, check);
                if (operand.Type == TypeSymbol.Char) operand = Convert(operand, TypeSymbol.Int32, unary.Position);
                if ((unary.Operator.Text is "+" or "-" && TypeFacts.Numeric(operand.Type)) ||
                    (unary.Operator.Text == "not" && operand.Type.Kind is TypeKind.Boolean or TypeKind.Int32))
                    return new BoundUnary(unary.Operator.Text, operand, operand.Type, check);
                throw source.Error("FP3002", "The unary operator does not accept this operand type.", unary.Position);
            case BinarySyntax binary:
                return BindBinary(binary, check);
            case ConditionalSyntax conditional:
                return BindConditional(conditional, check, expected);
            default:
                throw new InvalidOperationException("Unrecognized expression syntax.");
        }
    }

    private BoundExpression BindLiteral(LiteralSyntax literal)
    {
        var token = literal.Token;
        if (token.Kind == TokenKind.Character && int.TryParse(token.Text.Replace("_", "", StringComparison.Ordinal), out var character) && character is >= 0 and <= 65535)
            return new BoundCharacter((char)character);
        if (token.Kind == TokenKind.String)
            return new BoundString(token.Text);
        if (token.Text == "null" && token.Kind == TokenKind.Keyword) return new BoundNull();
        if (token.Kind == TokenKind.Keyword) return new BoundBoolean(token.Text == "true");
        var digits = token.Text.Replace("_", "", StringComparison.Ordinal);
        if (token.Kind == TokenKind.Integer && int.TryParse(digits, NumberStyles.None, CultureInfo.InvariantCulture, out var number))
            return new BoundInteger(number);
        if (token.Kind == TokenKind.Real && double.TryParse(digits.TrimEnd('d', 'D'), NumberStyles.Float, CultureInfo.InvariantCulture, out var real) && double.IsFinite(real))
            return new BoundDouble(real);
        throw source.Error("FP3002", "The literal is outside its int or finite double range.", literal.Position, token.Length);
    }

    private BoundExpression BindBinary(BinarySyntax syntax, bool check)
    {
        var left = BindExpression(syntax.Left, check);
        var incoming = nonNull.ToHashSet();
        if (left.Type == TypeSymbol.Boolean && syntax.Operator.Text is "and" or "or")
            SetFacts(Refine(syntax.Left, syntax.Operator.Text == "and", incoming));
        var right = BindExpression(syntax.Right, check);
        if (left.Type == TypeSymbol.Boolean && syntax.Operator.Text is "and" or "or" || syntax.Operator.Text == "??")
            SetFacts(Merge([incoming, nonNull.ToHashSet()]));
        var op = syntax.Operator.Text;
        if (op == "??")
        {
            if (left.Type == TypeSymbol.Null) return right;
            if (!left.Type.NeedsNullCheck && left.Type != TypeSymbol.Null)
                throw source.Error("FP3002", "Coalescing requires a nullable left operand.", syntax.Position);
            var commonResult = left.Type == TypeSymbol.Null ? right.Type : references.Common(left.Type.NonNullable, right.Type);
            if (commonResult.Kind is TypeKind.Unsupported or TypeKind.Void)
                throw source.Error("FP3002", "Coalescing operands require a common value type.", syntax.Position);
            return new BoundCoalesce(left, Convert(right, commonResult, syntax.Position), commonResult);
        }
        if (op is "=" or "<>" && (left.Type == TypeSymbol.Null || right.Type == TypeSymbol.Null))
        {
            var operand = left.Type == TypeSymbol.Null ? right : left;
            if (!operand.Type.CanHaveNull && !operand.Type.NullableValue && operand.Type != TypeSymbol.Null)
                throw source.Error("FP3002", "Only reference or nullable values can be compared with null.", syntax.Position);
            if ((operand.Type.Kind is TypeKind.Named or TypeKind.Record or TypeKind.Union) && operand.Type.IsReference)
            {
                var equality = references.FindMethod(operand.Type.NonNullable, op == "=" ? "op_Equality" : "op_Inequality",
                    [left.Type, right.Type], syntax.Position, source, optional: true);
                if (equality is not null) { ForgetCapturedFacts(); return new BoundCall(equality, ConvertArguments([left, right], equality.Parameters, syntax.Position)); }
            }
            return new BoundNullTest(operand, op == "=");
        }
        if (op == "+" && (left.Type == TypeSymbol.String && right.Type == TypeSymbol.Char || left.Type == TypeSymbol.Char && right.Type == TypeSymbol.String))
        {
            if (left.Type == TypeSymbol.Char) left = new BoundCall(references.FindMethod(TypeSymbol.Char, "ToString", [], syntax.Position, source, isStatic: false), [], left);
            if (right.Type == TypeSymbol.Char) right = new BoundCall(references.FindMethod(TypeSymbol.Char, "ToString", [], syntax.Position, source, isStatic: false), [], right);
        }
        var common = references.Common(left.Type, right.Type);
        if (common == TypeSymbol.Char) common = TypeSymbol.Int32;
        if (common.Kind == TypeKind.String && op is "+" or "=" or "<>")
        {
            var method = references.FindMethod(TypeSymbol.String, op == "+" ? "Concat" : op == "=" ? "op_Equality" : "op_Inequality",
                [left.Type, right.Type], syntax.Position, source);
            ForgetCapturedFacts();
            return new BoundCall(method, ConvertArguments([left, right], method.Parameters, syntax.Position));
        }
        if (common.IsReference && op is "=" or "<>")
        {
            var equality = references.FindMethod(common.NonNullable, op == "=" ? "op_Equality" : "op_Inequality",
                [left.Type, right.Type], syntax.Position, source, optional: true);
            if (equality is not null) { ForgetCapturedFacts(); return new BoundCall(equality, ConvertArguments([left, right], equality.Parameters, syntax.Position)); }
            return new BoundBinary(Convert(left, common, syntax.Position), op, Convert(right, common, syntax.Position), TypeSymbol.Boolean, check);
        }
        var arithmetic = (TypeFacts.Numeric(common) && (op is "+" or "-" or "*")) || (common == TypeSymbol.Double && op == "/") ||
            (common == TypeSymbol.Int32 && (op is "div" or "mod" or "shl" or "shr"));
        var comparison = (TypeFacts.Numeric(common) && (op is "=" or "<>" or "<" or "<=" or ">" or ">=")) ||
            (common == TypeSymbol.Boolean && (op is "=" or "<>"));
        var bitwise = (common.Kind is TypeKind.Boolean or TypeKind.Int32) && (op is "and" or "or" or "xor");
        if (!arithmetic && !comparison && !bitwise)
            throw source.Error("FP3002", $"Operator '{op}' does not accept {TypeFacts.Display(left.Type)} and {TypeFacts.Display(right.Type)}.", syntax.Position);
        return new BoundBinary(Convert(left, common, syntax.Position), op, Convert(right, common, syntax.Position),
            comparison ? TypeSymbol.Boolean : common, check);
    }
}

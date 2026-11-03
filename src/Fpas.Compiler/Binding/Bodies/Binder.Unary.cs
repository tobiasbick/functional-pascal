using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression BindUnaryOperation(BoundExpression operand, string op, int position, bool check, bool lifted = false)
    {
        if (BindConstrainedOperator(op, [operand], position, check, lifted) is { } call) return call;
        if (op == "not" && references.ScalarInfo(operand.Type) is { Category: ScalarCategory.Enum } enumeration)
        {
            var promoted = NumericFacts.Unary(enumeration.Base, "not");
            return new BoundConversion(new BoundUnary("not", new BoundConversion(operand, promoted, false), promoted, false), operand.Type, false);
        }
        if (TypeFacts.Numeric(operand.Type))
        {
            var promoted = NumericFacts.Unary(operand.Type, op);
            if (promoted == TypeSymbol.Unsupported || op == "not" && !NumericFacts.Integral(promoted))
                throw source.Error("FP3002", "Invalid numeric unary operator.", position);
            operand = Convert(operand, promoted, position);
        }
        if (op is "+" or "-" && TypeFacts.Numeric(operand.Type) ||
            op == "not" && (operand.Type == TypeSymbol.Boolean || NumericFacts.Integral(operand.Type)))
            return new BoundUnary(op, operand, operand.Type, check);
        throw source.Error("FP3002", "The unary operator does not accept this operand type.", position);
    }
}

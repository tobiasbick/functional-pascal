using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private ScalarSymbol? enumInitializer;
    internal object ScalarConstant(ExpressionSyntax syntax, TypeSymbol type, ScalarSymbol? owner = null)
    {
        InitializeImports(); currentRoutine = new("<type-constant>", TypeSymbol.Void, [], false, false, -1);
        enumInitializer = owner;
        var expression = BindExpression(syntax, expected: type);
        if (expression.Type.Scalar == owner && owner is not null) expression = new BoundConversion(expression, type, false);
        var value = Convert(expression, type, syntax.Position);
        if (!ConstantValue.IsConstant(value)) throw source.Error("FP3031", "A scalar bound or enum member must be a compile-time constant.", syntax.Position);
        return ConstantValue.Evaluate(value)!;
    }
}

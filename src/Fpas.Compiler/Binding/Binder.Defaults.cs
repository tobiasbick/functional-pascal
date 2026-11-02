using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    internal void BindDefaults()
    {
        InitializeImports();
        foreach (var syntax in unit.Syntax.Routines) BindDefaults(routines[syntax.Name.Text].Single(symbol => ReferenceEquals(symbol.Declaration, syntax)), syntax.Parameters);
        foreach (var record in unit.Records.Values)
        {
            var binder = new Binder(unit, catalog, references, recordContext: record);
            binder.InitializeImports();
            foreach (var syntax in record.Syntax.Methods) binder.BindDefaults(record.UserMethods[syntax.Name.Text].Single(symbol => ReferenceEquals(symbol.Declaration, syntax)), syntax.Parameters);
        }
    }

    private void BindDefaults(RoutineSymbol routine, IReadOnlyList<ParameterSyntax> syntax)
    {
        currentRoutine = routine;
        for (var index = 0; index < syntax.Count; index++)
        {
            if (syntax[index].Default is not { } expression) continue;
            var parameter = routine.Parameters[index];
            try
            {
                var value = Convert(BindExpression(expression, expected: parameter.Type), parameter.Type, expression.Position);
                if (!ConstantValue.IsConstant(value)) throw new InvalidOperationException();
                var constant = ConstantValue.Evaluate(value);
                var constantType = parameter.Type.NonNullable;
                if (constant is not null && constantType != TypeSymbol.String && constantType != TypeSymbol.Boolean && constantType != TypeSymbol.Int32 && constantType != TypeSymbol.Double && constantType != TypeSymbol.Char && constantType.Definition?.IsEnum != true)
                    throw new InvalidOperationException();
                parameter.Default = value;
            }
            catch (Exception failure) when (failure is InvalidOperationException or OverflowException or DivideByZeroException)
            {
                throw source.Error("FP3028", "An optional parameter default must be a representable CLR compile-time constant.", expression.Position);
            }
        }
    }
}

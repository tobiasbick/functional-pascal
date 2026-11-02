using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private bool bindingFilter;

    private BoundExpression BindAwait(AwaitSyntax syntax)
    {
        if (!currentRoutine.IsAsync || bindingFilter) throw source.Error("FP3030", "await requires an async body and cannot occur in a catch filter.", syntax.Position);
        var value = Receiver(syntax.Operand);
        ForgetCapturedFacts();
        return references.Await(value, source, syntax.Position);
    }
}

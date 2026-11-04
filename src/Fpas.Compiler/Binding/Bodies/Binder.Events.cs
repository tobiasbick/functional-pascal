using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundStatement BindSubscription(SubscriptionSyntax syntax)
    {
        var reference = syntax.Event;
        while (reference is ParenthesizedSyntax wrapped) reference = wrapped.Expression;
        if (reference is not MemberSyntax member)
            throw source.Error("FP3034", "subscribe/unsubscribe requires a CLR event member reference.", reference.Position);
        var selected = StaticTarget(member.Receiver);
        BoundExpression? receiver = null;
        var owner = selected is null ? (receiver = Receiver(member.Receiver)).Type : ImportedValueType(selected, member.Position);
        var contract = owner == TypeSymbol.Unsupported ? null : references.Event(owner, member.Name.Text, source, member.Name.Position);
        var accessor = syntax.Subscribe ? contract?.Add : contract?.Remove;
        if (accessor is null || accessor.IsStatic != (receiver is null))
            throw source.Error("FP3034", "The selected event accessor is missing, inaccessible, unsupported, or requires a different receiver kind.", member.Name.Position);
        catalog.Analysis?.Event(source, member.Name, accessor, accessor.Parameters[0]);
        return new BoundExpressionStatement(BindInvocation([accessor], [syntax.Handler], receiver, true, syntax.Position), false);
    }

    private void RejectEventValue(TypeSymbol owner, string name, int position)
    {
        if (references.Event(owner, name, source, position) is not null)
            throw source.Error("FP3034", "An event is not a delegate value; use subscribe or unsubscribe with a handler.", position);
    }
}

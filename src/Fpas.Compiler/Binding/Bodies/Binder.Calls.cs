using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression BindCall(CallSyntax call, bool check, TypeSymbol? expected)
    {
        if (source.Editor is null) return BindCallCore(call, check, expected);
        EditorExpression(call.Target, check);
        var previous = editorInvocation;
        editorInvocation = source.Editor.Context == "signature" && source.Editor.Position == call.ArgumentsStart ? call.Position : -1;
        try { return BindCallCore(call, check, expected); }
        finally { editorInvocation = previous; }
    }

    private BoundExpression BindCallCore(CallSyntax call, bool check, TypeSymbol? expected)
    {
        var target = call.Target;
        TypeList? typeArguments = null;
        if (target is GenericNameSyntax generic) { typeArguments = new(generic.Arguments.Select(BindType)); target = generic.Name; }
        if (target is NameExpressionSyntax nestedName && FindNested(nestedName.Name.Text) is { } nested)
            return CallNested(nested, call, typeArguments, check);
        if (target is NameExpressionSyntax localName && !HasValue(localName.Name.Text) && recordContext is not null && recordContext.UserMethods.TryGetValue(localName.Name.Text, out var ownMethod))
            return CallRoutine(ownMethod, call.Arguments, typeArguments, check, call.Position, ImplicitReceiver(ownMethod, call.Position));
        if (target is NameExpressionSyntax name && !HasValue(name.Name.Text) && routines.TryGetValue(name.Name.Text, out var routine))
            return CallRoutine(routine, call.Arguments, typeArguments, check, call.Position);
        if (target is MemberSyntax member)
        {
            var selected = StaticTarget(member.Receiver, allowOpenGeneric: true);
            if (selected is SourceImport imported)
            {
                if (imported.Unit.Globals.TryGetValue(member.Name.Text, out var global))
                {
                    if (typeArguments is not null) throw source.Error("FP3003", "A delegate field does not take explicit method type arguments.", call.Position);
                    var value = GlobalValue(global, member.Position);
                    catalog.Analysis?.Expression(source, member, value);
                    return InvokeDelegate(value, call.Arguments, call.Position, check);
                }
                return CallRoutine(AccessibleSource(imported.Unit, member.Name.Text, member.Position), call.Arguments, typeArguments, check, call.Position);
            }
            if (OpenUnionTarget(selected) is { } open)
            {
                if (typeArguments is not null) throw source.Error("FP3026", "Union type arguments belong on the owner, not the case factory.", call.Position);
                return InferUnion(open, member.Name.Text, call.Arguments, expected, check, call.Position);
            }
            BoundExpression? receiver = null;
            var owner = selected is null ? (receiver = Receiver(member.Receiver, check)).Type : ImportedValueType(selected, member.Position);
            if (owner == TypeSymbol.Unsupported) throw source.Error("FP3001", "A method needs a type or value receiver.", member.Position);
            if (references.UnionInfo(owner)?.Cases.FirstOrDefault(item => item.Name == member.Name.Text) is { } unionCase && receiver is null)
            {
                if (typeArguments is not null) throw source.Error("FP3026", "Union type arguments belong on the owner, not the case factory.", call.Position);
                return BindUnionFactory(unionCase, call.Arguments, check, call.Position);
            }
            CheckRecordMember(owner, member.Name.Text, false, member.Position);
            RejectEventValue(owner, member.Name.Text, member.Name.Position);
            if (typeArguments is null && (references.Property(owner, member.Name.Text, receiver is null, recordContext, source, member.Name.Position) is not null || references.Field(owner, member.Name.Text, receiver is null) is not null))
            {
                var value = BindClrMember(owner, receiver, member);
                catalog.Analysis?.Expression(source, member, value);
                return InvokeDelegate(value, call.Arguments, call.Position, check);
            }
            var methods = references.Methods(owner, member.Name.Text, receiver is null, typeArguments, recordContext, includeOpen: typeArguments is null);
            if (methods.Count == 0) references.DiagnoseUnsupportedMethod(owner, member.Name.Text, receiver is null, source, member.Name.Position);
            return BindInvocation(methods, call.Arguments, receiver, check, call.Position, inferGenerics: typeArguments is null,
                noApplicable: () => references.DiagnoseUnsupportedMethod(owner, member.Name.Text, receiver is null, source, member.Name.Position, call.Arguments.Count));
        }
        if (typeArguments is not null) throw source.Error("FP3003", "Explicit type arguments require a named source or CLR method.", call.Position);
        return InvokeDelegate(Receiver(target, check), call.Arguments, call.Position, check);
    }

    private BoundExpression InvokeDelegate(BoundExpression target, IReadOnlyList<ExpressionSyntax> arguments, int position, bool check)
    {
        var invoke = references.DelegateSignature(target.Type);
        if (invoke is null)
        {
            references.DiagnoseUnsupportedMethod(target.Type, "Invoke", false, source, position);
            throw source.Error("FP3019", "Only a supported CLR delegate value can be invoked.", position);
        }
        if (target.Type.IsNullable) throw source.Error("FP3013", "The delegate may be null; validate a stable binding before invocation.", position);
        if (!invoke.Flexible && arguments.All(argument => argument is not (NamedArgumentSyntax or ArgumentModeSyntax)))
        {
            if (source.Editor is not null) EditorSignatures([invoke], arguments, position);
            if (arguments.Count != invoke.Parameters.Count) throw source.Error("FP3003", "The delegate argument count does not match.", position);
            var values = arguments.Select((argument, index) => BindExpression(argument, check, invoke.Parameters[index])).ToArray();
            ForgetCapturedFacts();
            return new BoundCall(invoke, ConvertArguments(values, invoke.Parameters, position), target);
        }
        return BindInvocation([invoke], arguments, target, check, position);
    }

    private BoundExpression BindMethodGroup(ExpressionSyntax expression, TypeList? arguments, TypeSymbol? expected)
    {
        if (expression is NameExpressionSyntax nestedName && FindNested(nestedName.Name.Text) is { } nested)
            return NestedDelegate(nested, expected, expression.Position, arguments);
        if (expression is NameExpressionSyntax name && !HasValue(name.Name.Text))
        {
            if (recordContext is not null && recordContext.UserMethods.TryGetValue(name.Name.Text, out var own))
                return RoutineDelegate(own, expected, expression.Position, arguments, ImplicitReceiver(own, expression.Position));
            if (routines.TryGetValue(name.Name.Text, out var routine)) return RoutineDelegate(routine, expected, expression.Position, arguments);
        }
        if (expression is not MemberSyntax member) throw source.Error("FP3019", "An imported method group needs a type or value receiver.", expression.Position);
        var target = StaticTarget(member.Receiver);
        if (target is SourceImport sourceUnit) return SourceDelegate(sourceUnit.Unit, member.Name.Text, member.Position, expected, arguments);
        BoundExpression? receiver = null;
        var owner = target is null ? (receiver = Receiver(member.Receiver)).Type : ImportedValueType(target, member.Position);
        if (owner == TypeSymbol.Unsupported) throw source.Error("FP3019", "Unsupported method group receiver.", expression.Position);
        return ImportedDelegate(owner, member.Name.Text, receiver, arguments, expected, expression.Position);
    }

    private BoundExpression ImportedDelegate(TypeSymbol owner, string name, BoundExpression? receiver, TypeList? arguments, TypeSymbol? expected, int position)
    {
        var signature = expected is null ? null : references.DelegateSignature(expected);
        var methods = references.Methods(owner, name, receiver is null, arguments, recordContext);
        if (methods.Count == 0) references.DiagnoseUnsupportedMethod(owner, name, receiver is null, source, position);
        var candidates = methods.Where(method =>
            references.ValidMethodArguments(method) && (signature is null || references.DelegateCompatible(expected!, method.DelegateParameters, method.Result, method.ParameterInfo))).ToArray();
        if (candidates.Length > 1 && signature is null) return new BoundMethodGroup(candidates, receiver);
        var method = signature is not null ? references.DelegateMethod(expected!, candidates) : candidates.SingleOrDefault();
        if (method is null) throw source.Error("FP3019", $"Method group '{owner}.{name}' requires a unique supported delegate signature.", position);
        if (!references.ValidMethodArguments(method)) throw source.Error("FP3018", "Method type arguments violate CLR constraints.", position);
        var natural = expected is null || signature is null ? references.Callable(method.DelegateParameters, method.Result, source, position) : expected.NonNullable;
        return new BoundMethodDelegate(method, receiver, natural);
    }
}

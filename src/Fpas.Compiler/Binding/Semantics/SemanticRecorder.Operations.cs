using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private sealed record OperationEntry(SourceOperation Fact, SymbolEntry? Target);
    private readonly Dictionary<(string Body, int Offset, int Length, string Kind), OperationEntry> operations = [];
    private readonly HashSet<ExcludedOperation> excludedOperations = [];
    private readonly HashSet<BoundExpression> indexedExpressions = new(ReferenceEqualityComparer.Instance);
    private readonly Dictionary<BoundMethodGroup, (SourceText Source, ExpressionSyntax Syntax, BodyEntry Body, string Phase)> operationGroups = new(ReferenceEqualityComparer.Instance);

    private IEnumerable<SourceOperation> OperationFacts() => operations.Values.Select(entry => entry.Fact with { Target = entry.Target?.Id })
        .OrderBy(site => site.Span.Path, StringComparer.Ordinal).ThenBy(site => site.Span.Offset).ThenBy(site => site.Kind, StringComparer.Ordinal);

    private void Operations(SourceText source, ExpressionSyntax syntax, BoundExpression bound, string role)
    {
        if (activeBody is null) return;
        if (syntax is ParenthesizedSyntax or UncheckedSyntax or ArgumentModeSyntax) return;
        var value = Unwrap(bound);
        var anchor = NameToken(syntax is CallSyntax call ? call.Target : syntax);
        var location = anchor is null ? null : Span(source, anchor.Position, anchor.Length);
        var span = Span(source, syntax.SpanStart, syntax.SpanEnd - syntax.SpanStart);
        if (value is BoundMethodGroup group)
            operationGroups[group] = (source, syntax, activeBody, operationPhase);
        else if (value is BoundMethodDelegate method && syntax is NameExpressionSyntax or MemberSyntax or GenericNameSyntax)
            Add(method.Method, "callable_reference");
        else if (value is BoundLambda lambda && syntax is LambdaSyntax && bodies.TryGetValue(lambda.Closure.Body.Symbol, out var body))
            Store(span, location, "callable_reference", null, body.Body.Id, null, "direct", null, Signature(references.DelegateSignature(lambda.Type)!), result: lambda.Type);
        else if (syntax is NewSyntax creation)
        {
            var name = creation.Type.Segments[^1].Name;
            location = Span(source, name.Position, name.Length);
            Type(value.Type);
            var target = value.Type.IsParameter ? null : External(TypeMetadata(value.Type), "type", TypeFacts.Display(value.Type));
            var constructor = value is BoundNew initialized ? initialized.Constructor : null;
            Store(span, location, "construct", target, null, constructor is null ? null : MethodMetadata(constructor),
                value.Type.IsParameter ? "constrained" : "direct", constructor is null ? value.Type.IsParameter ? "type_parameter_constructor" : "value_initialization" : null,
                constructor is null ? new(Array.Empty<SemanticParameter>(), Type(value.Type), Array.Empty<string>()) : Signature(constructor), value.Type, value.Type);
        }
        else if (syntax is CallSyntax && value is BoundCall invocation)
        {
            var isDelegate = invocation.Method.Name == "Invoke" && invocation.Method.SourceMethod is null && references.DelegateSignature(invocation.Method.Owner) is not null;
            if (isDelegate)
            {
                var receiver = ((CallSyntax)syntax).Target;
                var receiverSpan = Span(source, receiver.SpanStart, receiver.SpanEnd - receiver.SpanStart);
                var receiverSymbol = NameToken(receiver) is { } name ? occurrences.GetValueOrDefault((source.Path, name.Position, "read")).Symbol?.Id : null;
                Store(span, location, "delegate_invoke", null, null, MethodMetadata(invocation.Method), "delegate", "runtime_delegate_target", Signature(invocation.Method),
                    invocation.Method.Owner, invocation.Method.Result, new(receiverSpan, receiverSymbol, Type(invocation.Method.Owner)));
            }
            else Add(invocation.Method, VariantFactory(invocation.Method) ? "construct" : "invoke");
        }
        else if (value is BoundProperty { Property.Getter: { } factoryGetter } && syntax is MemberSyntax && VariantFactory(factoryGetter)) Add(factoryGetter, "construct");
        else if (value is BoundProperty property && syntax is NameExpressionSyntax or MemberSyntax or IndexSyntax)
        {
            var accessor = role == "write" ? property.Property.Setter : property.Property.Getter;
            if (accessor is not null && !Storage(accessor))
            {
                if (syntax is IndexSyntax index)
                {
                    var bracket = tokens[source.Path].Values.Where(token => token.Text == "[" && token.Position >= index.Receiver.SpanEnd && token.Position < syntax.SpanEnd)
                        .OrderBy(token => token.Position).FirstOrDefault();
                    if (bracket is not null) location = Span(source, bracket.Position, bracket.Length);
                }
                Add(accessor, role == "write" ? "property_set" : "property_get", Property(accessor));
            }
        }
        else if (value is BoundCall factory && syntax is NameExpressionSyntax or MemberSyntax && VariantFactory(factory.Method)) Add(factory.Method, "construct");
        if (syntax is BinarySyntax or UnarySyntax)
        {
            foreach (var method in Operators(bound))
            {
                var operatorToken = syntax is BinarySyntax binary ? binary.Operator : ((UnarySyntax)syntax).Operator;
                location = Span(source, operatorToken.Position, operatorToken.Length);
                Add(method, "operator");
            }
        }
        if (syntax is AwaitSyntax) Exclude(source, "await_protocol", syntax.SpanStart, syntax.SpanEnd - syntax.SpanStart);
        indexedExpressions.Add(bound);

        void Add(ImportedMethod method, string kind, SymbolEntry? logical = null)
        {
            if (kind is not ("construct" or "property_get" or "property_set") && Storage(method))
            {
                Exclude(source, "generated_infrastructure", span.Offset, span.Length);
                return;
            }
            var dispatch = method.ConstrainedType is not null ? "constrained" : method.Owner.IsInterface ? "interface" : method.IsVirtual ? "virtual" : "direct";
            var unknown = dispatch == "direct" ? null : "runtime_dispatch_target";
            Store(span, location, kind, logical ?? Method(method), null, method.Owner.Kind == TypeKind.Generated ? null : MethodMetadata(method), dispatch, unknown, Signature(method),
                method.Owner, kind == "callable_reference" ? bound.Type : method.Result);
        }
    }

    private IEnumerable<ImportedMethod> Operators(BoundExpression expression)
    {
        if (indexedExpressions.Contains(expression)) yield break;
        if (expression is BoundCall call && call.Method.Name.StartsWith("op_", StringComparison.Ordinal) && !Storage(call.Method)) yield return call.Method;
        foreach (var child in BoundTree.Children(expression)) foreach (var method in Operators(child)) yield return method;
    }

    private bool VariantFactory(ImportedMethod method) => method.SourceMethod?.Generated == GeneratedMethodKind.Factory ||
        references.VariantInfo(method.Owner)?.Cases.Any(member => member.Factory.Name == method.Name) == true;

    private bool Storage(ImportedMethod method)
    {
        if (method.SourceMethod is { Generated: not GeneratedMethodKind.None, InterfaceOwner: null }) return true;
        var record = references.RecordInfo(method.Owner);
        if (record is not null && (record.Fields.Any(field => field.Getter.Name == method.Name) || method.Name is "Equals" or "GetHashCode" or "op_Equality" or "op_Inequality" || method.Name.StartsWith("<With:", StringComparison.Ordinal) || method.Name.StartsWith("<Default:", StringComparison.Ordinal))) return true;
        if (references.ScalarInfo(method.Owner) is not null && (method.Name == "get_Value" || method.Name.StartsWith("op_", StringComparison.Ordinal))) return true;
        if (method.Owner.Definition?.DeclaringType is { } parent && references.VariantInfo(references.Symbol(parent) with { Arguments = method.Owner.Arguments }) is { } variant &&
            variant.Cases.Any(member => member.Type == method.Owner && (member.Fields.Any(field => field.Getter.Name == method.Name) ||
                method.Name is "get_Tag" or "Equals" or "GetHashCode" or "op_Equality" or "op_Inequality"))) return true;
        return references.VariantInfo(method.Owner) is not null && method.Name is "Equals" or "GetHashCode" or "op_Equality" or "op_Inequality" or "get_Tag";
    }

    private void Store(SourceSpan span, SourceSpan? anchor, string kind, SymbolEntry? target, string? targetBody,
        MetadataIdentity? operation, string dispatch, string? unknown, CallableSignature? signature,
        TypeSymbol? owner = null, TypeSymbol? result = null, OperationReceiver? receiver = null)
    {
        if (activeBody is null) return;
        var key = (activeBody.Body.Id, span.Offset, span.Length, kind);
        var id = operations.TryGetValue(key, out var previous) ? previous.Fact.Id : snapshotId + ":o" + operations.Count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        operations[key] = new(new(id, activeBody.Body.Id, span, anchor, kind, operationPhase, target?.Id, targetBody, operation, dispatch, unknown, signature,
            owner is null || owner.Kind == TypeKind.Generated ? null : Type(owner), result is null ? null : Type(result), receiver), target);
    }

    private void RetargetOperation(BoundMethodGroup group, ImportedMethod method)
    {
        if (!operationGroups.TryGetValue(group, out var origin)) return;
        using var body = new BodyScope(this, origin.Body);
        using var phase = Phase(origin.Phase);
        Operations(origin.Source, origin.Syntax, new BoundMethodDelegate(method, group.Receiver, references.Callable(method.DelegateParameters, method.Result, origin.Source, origin.Syntax.Position)), "read");
    }

    private void EventOperation(SourceText source, Token name, ImportedMethod accessor, SymbolEntry symbol, int start, int end)
    {
        var span = Span(source, start, end - start);
        var anchor = Span(source, name.Position, name.Length);
        var dispatch = accessor.ConstrainedType is not null ? "constrained" : accessor.Owner.IsInterface ? "interface" : accessor.IsVirtual ? "virtual" : "direct";
        Store(span, anchor, accessor.Name.StartsWith("add_", StringComparison.Ordinal) ? "event_add" : "event_remove", symbol, null,
            MethodMetadata(accessor), dispatch, dispatch == "direct" ? null : "runtime_dispatch_target", Signature(accessor), accessor.Owner, accessor.Result);
    }
}

using Fpas.Debugging.Core;

namespace Fpas.Debugging.Evaluation;

internal interface IPassiveContext
{
    DebugBinding? Name(string name);
    DebugBinding Static(string type, string member);
    DebugBinding Member(DebugBinding receiver, string name);
    DebugBinding MemberType(DebugBinding receiver, string name);
    DebugBinding Index(DebugBinding receiver, DebugBinding[] indices);
}

internal sealed class PassiveEvaluator(IPassiveContext context)
{
    private readonly Dictionary<PassiveExpression, DebugBinding> values = new(ReferenceEqualityComparer.Instance);
    private int reads;

    internal DebugBinding Evaluate(PassiveExpression expression)
    {
        if (++reads > 16384) throw new DebugException("limit_exceeded", "Passive expression traversal exceeds 16384 operations.");
        if (values.TryGetValue(expression, out var previous)) return previous;
        var result = expression switch
        {
            LiteralExpression literal => ScalarOperations.Literal(literal.Token),
            NameExpression name => context.Name(name.Name) ?? throw Unknown(name.Name),
            ParenthesizedExpression parent => Evaluate(parent.Operand),
            UnaryExpression { Operator: "-", Operand: LiteralExpression literal } when Numeric(literal.Token)
                => ScalarOperations.Literal(literal.Token, negative: true),
            UnaryExpression unary => ScalarOperations.Unary(unary.Operator, Evaluate(unary.Operand)),
            BinaryExpression binary => Binary(binary),
            MemberExpression member => Member(member),
            IndexExpression index => context.Index(Evaluate(index.Receiver), index.Indices.Select(Evaluate).ToArray()),
            _ => throw new DebugException("unsupported_expression", "Unsupported passive expression.")
        };
        ScalarOperations.Available(result); values.Add(expression, result); return result;
    }

    private DebugBinding Binary(BinaryExpression binary)
    {
        var left = Evaluate(binary.Left);
        if (binary.Operator is "and" or "or" && left.Type == "bool")
        {
            var rightType = Type(binary.Right);
            if (rightType.Type != "bool") throw new DebugException("unsupported_expression", "Boolean and/or require a Boolean right operand.");
            if (binary.Operator == "and" && left.Scalar == "false" || binary.Operator == "or" && left.Scalar == "true")
                return ScalarOperations.Boolean(left.Scalar == "true");
        }
        return ScalarOperations.Binary(left, binary.Operator, Evaluate(binary.Right));
    }

    private DebugBinding Member(MemberExpression member)
    {
        var names = new List<string>(); PassiveExpression root = member;
        while (root is MemberExpression current) { names.Add(current.Name); root = current.Receiver; }
        if (root is NameExpression name)
        {
            var receiver = context.Name(name.Name);
            names.Reverse();
            if (receiver is null) return context.Static(string.Join('.', new[] { name.Name }.Concat(names.SkipLast(1))), names[^1]);
            foreach (var field in names) receiver = context.Member(receiver, field);
            return receiver;
        }
        return context.Member(Evaluate(member.Receiver), member.Name);
    }

    private DebugBinding Type(PassiveExpression expression)
    {
        if (++reads > 16384) throw new DebugException("limit_exceeded", "Passive type traversal exceeds 16384 operations.");
        if (values.TryGetValue(expression, out var value)) return value;
        return expression switch
        {
            LiteralExpression or NameExpression => Evaluate(expression),
            MemberExpression member => MemberType(member),
            ParenthesizedExpression parent => Type(parent.Operand),
            UnaryExpression { Operator: "-", Operand: LiteralExpression literal } when Numeric(literal.Token) => Evaluate(expression),
            UnaryExpression unary => ScalarOperations.UnaryType(unary.Operator, Type(unary.Operand)),
            BinaryExpression binary => ScalarOperations.BinaryType(Type(binary.Left), binary.Operator, Type(binary.Right)),
            IndexExpression index => IndexType(index),
            _ => throw new DebugException("unsupported_expression", "The expression's passive type is unavailable.")
        };
    }

    private DebugBinding IndexType(IndexExpression index)
    {
        var receiver = Type(index.Receiver);
        foreach (var operand in index.Indices)
            if (Type(operand).Type is not ("sbyte" or "byte" or "short" or "ushort" or "char" or "int"))
                throw new DebugException("unsupported_expression", "An index must convert implicitly to int.");
        var rank = receiver.Type == "string" ? 1 : receiver.RuntimeType?.Rank;
        if (rank != index.Indices.Length) throw new DebugException("invalid_expression", "The index count must match the receiver's rank.");
        if (receiver.Type == "string") return new("", "char", "character", null);
        if (receiver.RuntimeType?.ElementKind is 20 or 29 && receiver.RuntimeType.Arguments is [var element])
            return new("", element.Name, "unknown", null, RuntimeType: element);
        throw new DebugException("unsupported_expression", "Indexing requires an array or string.");
    }

    private DebugBinding MemberType(MemberExpression member)
    {
        if (member.Receiver is NameExpression name && context.Name(name.Name) is null) return Member(member);
        return context.MemberType(Type(member.Receiver), member.Name);
    }

    private static bool Numeric(ExpressionToken token) => token.Text.Length != 0 && (char.IsAsciiDigit(token.Text[0]) || token.Text[0] == '$');
    private static DebugException Unknown(string name) => new("unresolved_name", $"No visible source or accessible static binding is named '{name}'.");
}

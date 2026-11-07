using Fpas.Debugging.Core;

namespace Fpas.Debugging.Evaluation;

internal sealed record ExpressionToken(string Kind, string Text, int Position);
internal abstract record PassiveExpression(int Depth);
internal sealed record LiteralExpression(ExpressionToken Token) : PassiveExpression(1);
internal sealed record NameExpression(string Name) : PassiveExpression(1);
internal sealed record UnaryExpression(string Operator, PassiveExpression Operand) : PassiveExpression(Operand.Depth + 1);
internal sealed record BinaryExpression(PassiveExpression Left, string Operator, PassiveExpression Right) : PassiveExpression(Math.Max(Left.Depth, Right.Depth) + 1);
internal sealed record MemberExpression(PassiveExpression Receiver, string Name) : PassiveExpression(Receiver.Depth + 1);
internal sealed record IndexExpression(PassiveExpression Receiver, PassiveExpression[] Indices) : PassiveExpression(Math.Max(Receiver.Depth, Indices.Max(index => index.Depth)) + 1);
internal sealed record ParenthesizedExpression(PassiveExpression Operand) : PassiveExpression(Operand.Depth + 1);

internal sealed class ExpressionParser
{
    private readonly ExpressionToken[] tokens;
    private int position, nodes, nesting;
    private ExpressionToken Current => tokens[position];
    private ExpressionParser(string source) => tokens = ExpressionLexer.Read(source);

    internal static PassiveExpression Parse(string source)
    {
        if (string.IsNullOrWhiteSpace(source) || source.Length > 8192) throw new DebugException("invalid_expression", "Expression text must contain 1..8192 UTF-16 code units.");
        var parser = new ExpressionParser(source); var expression = parser.Expression();
        if (parser.Current.Kind != "end") throw Unsupported(parser.Current);
        return expression;
    }

    private const int LogicalPrecedence = 4;

    private static int Precedence(ExpressionToken token) => token.Kind is "name" or "literal" or "end" ? 0 : token.Text switch
    {
        "or" or "xor" or "and" => LogicalPrecedence, "=" or "<>" => 5, "<" or "<=" or ">" or ">=" => 6,
        "shl" or "shr" => 7, "+" or "-" => 8, "*" or "/" or "div" or "mod" => 9, _ => 0
    };

    private PassiveExpression Expression(int minimum = 1)
    {
        if (++nesting > 64) throw new DebugException("limit_exceeded", "Expression parsing exceeds 64 nesting levels.");
        try
        {
            var left = Unary();
            string? logicalOperator = null;
            while (Precedence(Current) >= minimum)
            {
                var op = tokens[position++];
                var precedence = Precedence(op);
                if (precedence == LogicalPrecedence)
                {
                    logicalOperator ??= op.Text;
                    if (logicalOperator != op.Text)
                        throw new DebugException("invalid_expression", $"Group mixed logical operators with parentheses at column {op.Position + 1}.");
                }
                var right = Expression(precedence + 1);
                if (precedence is 5 or 6 && (Comparison(left) || Comparison(right))) throw new DebugException("invalid_expression", "Comparisons cannot be chained; use parentheses or Boolean operators.");
                left = Add(new BinaryExpression(left, op.Text, right));
            }
            return left;
        }
        finally { nesting--; }
    }

    private PassiveExpression Unary()
    {
        if (Current.Kind != "name" && Current.Text is "+" or "-" or "not")
        {
            if (++nesting > 64) throw new DebugException("limit_exceeded", "Expression parsing exceeds 64 nesting levels.");
            try { var op = tokens[position++]; return Add(new UnaryExpression(op.Text, Unary())); }
            finally { nesting--; }
        }
        PassiveExpression expression;
        if (Current.Text == "(" && Current.Kind == "symbol")
        { position++; expression = Add(new ParenthesizedExpression(Expression())); Expect(")"); }
        else if (Current.Kind == "literal") expression = Add(new LiteralExpression(tokens[position++]));
        else if (Current.Kind == "name") expression = Add(new NameExpression(tokens[position++].Text));
        else throw Unsupported(Current);
        while (true)
        {
            if (Current.Text == ".")
            {
                position++; if (Current.Kind != "name") throw Unsupported(Current);
                expression = Add(new MemberExpression(expression, tokens[position++].Text));
            }
            else if (Current.Text == "[")
            {
                position++; var indices = new List<PassiveExpression> { Expression() };
                while (Current.Text == ",") { position++; if (indices.Count == 32) throw new DebugException("limit_exceeded", "Array indexing exceeds rank 32."); indices.Add(Expression()); }
                Expect("]"); expression = Add(new IndexExpression(expression, indices.ToArray()));
            }
            else break;
        }
        return expression;
    }

    private T Add<T>(T node) where T : PassiveExpression
    {
        if (++nodes > 4096 || node.Depth > 64) throw new DebugException("limit_exceeded", "Expressions are bounded at 4096 nodes and depth 64.");
        return node;
    }
    private void Expect(string symbol) { if (Current.Text != symbol || Current.Kind != "symbol") throw Unsupported(Current); position++; }
    private static bool Comparison(PassiveExpression node) => node is BinaryExpression { Operator: "=" or "<>" or "<" or ">" or "<=" or ">=" };
    private static DebugException Unsupported(ExpressionToken token) => new("unsupported_expression", $"Passive expressions do not support '{token.Text}' at column {token.Position + 1}. Calls, construction, mutation and conversion execution are unavailable.");
}

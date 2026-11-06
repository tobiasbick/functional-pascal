using Fpas.Debugging.Core;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Evaluation;

internal static partial class ScalarOperations
{
    internal static DebugBinding UnaryType(string op, DebugBinding operand)
    {
        if (op == "not" && operand.Type == "bool") return new("", "bool", "boolean", null);
        var kind = UnaryKind(Kind(operand.Type), op);
        if (kind == 0 || op == "not" && !Integral(kind)) throw Error("The unary operator requires a supported primitive scalar.");
        return TypeValue(kind);
    }

    internal static DebugBinding BinaryType(DebugBinding left, string op, DebugBinding right)
    {
        if (left.Type == "bool" && right.Type == "bool" && op is "and" or "or" or "xor" or "=" or "<>") return new("", "bool", "boolean", null);
        if (op is "=" or "<>" && (left.Kind == "null" || right.Kind == "null"))
        {
            var operand = left.Kind == "null" ? right : left;
            if (operand.Kind != "null" && operand.Type != "string" && operand.RuntimeType?.ElementKind is not (14 or 18 or 20 or 28 or 29)) throw Error("Only reference values can be compared with null in passive expressions.");
            return new("", "bool", "boolean", null);
        }
        if (left.Type == "string" && right.Type == "string" && op is "+" or "=" or "<>")
            return new("", op == "+" ? "string" : "bool", op == "+" ? "string" : "boolean", null);
        if (op == "+" && (left.Type == "string" && right.Type == "char" || left.Type == "char" && right.Type == "string")) return new("", "string", "string", null);
        var shift = op is "shl" or "shr";
        var kind = shift ? UnaryKind(Kind(left.Type), "+") : Promote(left with { Constant = false }, right with { Constant = false });
        if (kind == 0 || op is "and" or "or" or "xor" or "div" or "mod" or "shl" or "shr" && (!Integral(kind) || !Integral(Kind(right.Type)))
            || op == "/" && kind is not (12 or 13 or 80)) throw Error("The numeric operator does not accept these types.");
        return Comparison(op) ? new("", "bool", "boolean", null) : TypeValue(kind);
    }

    private static DebugBinding TypeValue(uint kind) => new("", kind == 80 ? "decimal" : ScalarValue.TypeName(kind)!, "unknown", null);
}

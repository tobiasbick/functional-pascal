using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal static class SemanticPolicy
{
    internal static IReadOnlyDictionary<Type, string> Expressions { get; } = new Dictionary<Type, string>
    {
        [typeof(BoundLocal)] = "original local; declared and flow types",
        [typeof(BoundParameter)] = "original parameter; declared and flow types",
        [typeof(BoundCapture)] = "original captured binding; declared and flow types",
        [typeof(BoundField)] = "resolved field definition",
        [typeof(BoundRecordField)] = "resolved source property",
        [typeof(BoundProperty)] = "resolved property accessor and definition",
        [typeof(BoundArrayLength)] = "explicit System.Array.Length property",
        [typeof(BoundCall)] = "explicit call target; no implicit Invoke/operator/accessor reference",
        [typeof(BoundMethodDelegate)] = "selected callable target",
        [typeof(BoundMethodGroup)] = "defer target facts until delegate overload selection",
        [typeof(BoundAddress)] = "written ref/out/in role; managed expression type",
        [typeof(BoundSequence)] = "source extent/type; resolve explicit target through temporary storage",
        [typeof(BoundConversion)] = "source extent/type; preserve child checked types",
        [typeof(BoundString)] = "literal type only", [typeof(BoundInteger)] = "literal type only",
        [typeof(BoundBoolean)] = "literal type only", [typeof(BoundDouble)] = "literal type only",
        [typeof(BoundCharacter)] = "literal type only", [typeof(BoundNumber)] = "literal type only", [typeof(BoundNull)] = "literal type only",
        [typeof(BoundNew)] = "construction type; written type/named fields indexed separately",
        [typeof(BoundTypeOf)] = "expression type; written operand type indexed separately",
        [typeof(BoundCoalesce)] = "checked expression and child types",
        [typeof(BoundNullTest)] = "checked expression and child types",
        [typeof(BoundUnary)] = "checked expression and child types",
        [typeof(BoundBinary)] = "checked expression and child types",
        [typeof(BoundConditional)] = "checked expression and child types",
        [typeof(BoundNewArray)] = "array and length expression types",
        [typeof(BoundArrayLiteral)] = "array and element expression types",
        [typeof(BoundArrayIndex)] = "element and index types; no implicit indexer reference",
        [typeof(BoundSafeCast)] = "checked result; written type indexed separately",
        [typeof(BoundTypeTest)] = "bool result; written type indexed separately",
        [typeof(BoundLambda)] = "callable expression; parameter/body identities indexed during binding",
        [typeof(BoundAwait)] = "await result; no generated awaiter references",
        [typeof(BoundFailure)] = "case result; no generated failure occurrence",
        [typeof(BoundDereference)] = "generated storage; source address facts retained",
        [typeof(BoundCaptureStorage)] = "generated storage; never an extra occurrence",
        [typeof(BoundLocalStorage)] = "generated storage; never an extra occurrence",
        [typeof(BoundClosureSelf)] = "generated closure; never a source declaration",
        [typeof(BoundEnvironment)] = "generated environment; never a source declaration",
        [typeof(BoundNoValue)] = "generated async marker; never a source declaration",
        [typeof(BoundPendingException)] = "generated cleanup state; never a source declaration"
    };

    internal static IReadOnlySet<Type> Syntax { get; } = new HashSet<Type>
    {
        typeof(ArgumentModeSyntax), typeof(LiteralSyntax), typeof(ParenthesizedSyntax), typeof(NameExpressionSyntax), typeof(CallSyntax),
        typeof(UnarySyntax), typeof(BinarySyntax), typeof(CastSyntax), typeof(TypeOfSyntax), typeof(UncheckedSyntax), typeof(ConditionalSyntax),
        typeof(MemberSyntax), typeof(NewSyntax), typeof(GenericNameSyntax), typeof(NewArraySyntax), typeof(ArrayLiteralSyntax), typeof(IndexSyntax),
        typeof(LambdaSyntax), typeof(AwaitSyntax), typeof(NamedArgumentSyntax), typeof(WithSyntax), typeof(CaseExpressionSyntax), typeof(TypeTestSyntax)
    };
}

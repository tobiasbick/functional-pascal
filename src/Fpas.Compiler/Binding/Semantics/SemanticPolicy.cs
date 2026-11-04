using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal static class SemanticPolicy
{
    internal enum OperationIndexing { None, Source, Children, Protocol, Generated }
    internal sealed record ExpressionPolicy(string Navigation, OperationIndexing Operations);
    internal static IReadOnlyDictionary<Type, ExpressionPolicy> Expressions { get; } = new Dictionary<Type, ExpressionPolicy>
    {
        [typeof(BoundLocal)] = new("original local; declared and flow types", OperationIndexing.None),
        [typeof(BoundParameter)] = new("original parameter; declared and flow types", OperationIndexing.None),
        [typeof(BoundCapture)] = new("original captured binding; declared and flow types", OperationIndexing.None),
        [typeof(BoundField)] = new("resolved field definition", OperationIndexing.Children),
        [typeof(BoundRecordField)] = new("resolved source property", OperationIndexing.Children),
        [typeof(BoundProperty)] = new("resolved property accessor and definition", OperationIndexing.Source),
        [typeof(BoundArrayLength)] = new("explicit System.Array.Length property", OperationIndexing.Children),
        [typeof(BoundCall)] = new("explicit call target; no implicit Invoke/operator/accessor reference", OperationIndexing.Source),
        [typeof(BoundMethodDelegate)] = new("selected callable target", OperationIndexing.Source),
        [typeof(BoundMethodGroup)] = new("defer target facts until delegate overload selection", OperationIndexing.Source),
        [typeof(BoundAddress)] = new("written ref/out/in role; managed expression type", OperationIndexing.Children),
        [typeof(BoundSequence)] = new("source extent/type; resolve explicit target through temporary storage", OperationIndexing.Source),
        [typeof(BoundConversion)] = new("source extent/type; preserve child checked types", OperationIndexing.Source),
        [typeof(BoundString)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundInteger)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundBoolean)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundDouble)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundCharacter)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundNumber)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundNull)] = new("literal type only", OperationIndexing.None),
        [typeof(BoundNew)] = new("construction type; written type/named fields indexed separately", OperationIndexing.Source),
        [typeof(BoundTypeOf)] = new("expression type; written operand type indexed separately", OperationIndexing.None),
        [typeof(BoundCoalesce)] = new("checked expression and child types", OperationIndexing.Children),
        [typeof(BoundNullTest)] = new("checked expression and child types", OperationIndexing.Children),
        [typeof(BoundUnary)] = new("checked expression and child types", OperationIndexing.Source),
        [typeof(BoundBinary)] = new("checked expression and child types", OperationIndexing.Source),
        [typeof(BoundConditional)] = new("checked expression and child types; lifted operators retain source operations", OperationIndexing.Source),
        [typeof(BoundNewArray)] = new("array and length expression types", OperationIndexing.Children),
        [typeof(BoundArrayLiteral)] = new("array and element expression types", OperationIndexing.Children),
        [typeof(BoundArrayIndex)] = new("element and index types; no implicit indexer reference", OperationIndexing.Children),
        [typeof(BoundSafeCast)] = new("checked result; written type indexed separately", OperationIndexing.Children),
        [typeof(BoundTypeTest)] = new("bool result; written type indexed separately", OperationIndexing.Children),
        [typeof(BoundLambda)] = new("callable expression; parameter/body identities indexed during binding", OperationIndexing.Source),
        [typeof(BoundAwait)] = new("await result; no generated awaiter references", OperationIndexing.Protocol),
        [typeof(BoundFailure)] = new("case result; no generated failure occurrence", OperationIndexing.Generated),
        [typeof(BoundDereference)] = new("generated storage; source address facts retained", OperationIndexing.Generated),
        [typeof(BoundCaptureStorage)] = new("generated storage; never an extra occurrence", OperationIndexing.Generated),
        [typeof(BoundLocalStorage)] = new("generated storage; never an extra occurrence", OperationIndexing.Generated),
        [typeof(BoundClosureSelf)] = new("generated closure; never a source declaration", OperationIndexing.Generated),
        [typeof(BoundEnvironment)] = new("generated environment; never a source declaration", OperationIndexing.Generated),
        [typeof(BoundNoValue)] = new("generated async marker; never a source declaration", OperationIndexing.Generated),
        [typeof(BoundPendingException)] = new("generated cleanup state; never a source declaration", OperationIndexing.Generated)
    };

    internal static IReadOnlySet<Type> Syntax { get; } = new HashSet<Type>
    {
        typeof(ArgumentModeSyntax), typeof(LiteralSyntax), typeof(ParenthesizedSyntax), typeof(NameExpressionSyntax), typeof(CallSyntax),
        typeof(UnarySyntax), typeof(BinarySyntax), typeof(CastSyntax), typeof(TypeOfSyntax), typeof(UncheckedSyntax), typeof(ConditionalSyntax),
        typeof(MemberSyntax), typeof(NewSyntax), typeof(GenericNameSyntax), typeof(NewArraySyntax), typeof(ArrayLiteralSyntax), typeof(IndexSyntax),
        typeof(LambdaSyntax), typeof(AwaitSyntax), typeof(NamedArgumentSyntax), typeof(WithSyntax), typeof(CaseExpressionSyntax), typeof(TypeTestSyntax)
    };
}

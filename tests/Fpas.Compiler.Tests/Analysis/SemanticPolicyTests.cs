using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.Semantics;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class SemanticPolicyTests
{
    [Fact]
    public void Every_bound_and_syntax_expression_has_an_explicit_source_index_policy()
    {
        var types = typeof(FpasCompiler).Assembly.GetTypes().Where(type => !type.IsAbstract).ToArray();
        var expressions = types.Where(type => type.IsSubclassOf(typeof(BoundExpression))).OrderBy(type => type.FullName).ToArray();
        Assert.Equal(expressions, SemanticPolicy.Expressions.Keys.OrderBy(type => type.FullName));
        Assert.All(SemanticPolicy.Expressions.Values, policy => { Assert.NotEmpty(policy.Navigation); Assert.True(Enum.IsDefined(policy.Operations)); });
        var syntax = types.Where(type => type.IsSubclassOf(typeof(ExpressionSyntax))).OrderBy(type => type.FullName).ToArray();
        Assert.Equal(syntax, SemanticPolicy.Syntax.OrderBy(type => type.FullName));
    }
}

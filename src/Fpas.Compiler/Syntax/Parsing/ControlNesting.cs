// [Documentation](../../../../docs/architecture/workspaces.md#semantic-lint)
namespace Fpas.Compiler.Syntax.Parsing;

internal sealed class ControlNesting(int limit)
{
    private int depth;
    internal List<Token> Exceeded { get; } = [];

    internal Scope EnterControl(Token keyword)
    {
        var scope = new Scope(this, depth);
        if (depth == limit)
            Exceeded.Add(keyword);
        depth++;
        return scope;
    }

    internal Scope EnterRoutine()
    {
        var scope = new Scope(this, depth);
        depth = 0;
        return scope;
    }

    internal readonly struct Scope(ControlNesting owner, int previousDepth) : IDisposable
    {
        public void Dispose() => owner.depth = previousDepth;
    }
}

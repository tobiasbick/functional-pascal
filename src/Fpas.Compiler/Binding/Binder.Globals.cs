namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundBlock BindGlobalInitializers()
    {
        scopes.Push(new(StringComparer.Ordinal));
        try
        {
            var statements = new List<BoundStatement>();
            foreach (var global in unit.Globals.Values)
            {
                var syntax = global.Syntax.Binding;
                var value = Convert(BindExpression(syntax.Initializer, expected: global.Type), global.Type, syntax.Initializer.Position);
                statements.Add(new BoundSequencePoint(source.StatementLocation(syntax.Position), new BoundAssignment(new BoundField(global.Field, null), value)));
            }
            return new(statements, scopes.Peek().Values.ToArray());
        }
        finally { scopes.Pop(); }
    }

    private BoundExpression GlobalValue(GlobalSymbol global, int position, bool writing = false)
    {
        if (!global.Syntax.IsPublic && global.Owner != unit) throw source.Error("FP3011", $"Global '{global.Name}' is private to its unit.", position);
        if (writing && !global.Syntax.Binding.Mutable) throw source.Error("FP3009", "A global const cannot be assigned.", position);
        return new BoundField(global.Field, null);
    }
}

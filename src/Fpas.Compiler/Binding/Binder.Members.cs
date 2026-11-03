using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private static IReadOnlyList<string>? NameParts(ExpressionSyntax expression) => expression switch
    {
        NameExpressionSyntax name => [name.Name.Text],
        MemberSyntax member when NameParts(member.Receiver) is { } parts => parts.Append(member.Name.Text).ToArray(),
        _ => null
    };

    private ImportTarget? StaticTarget(ExpressionSyntax expression, int arity = 0, bool allowOpenGeneric = false)
    {
        var path = TypePath(expression);
        if (path is null || HasValue(path[0].Name.Text)) return null;
        if (expression is MemberSyntax member)
        {
            var prefix = StaticTarget(member.Receiver, allowOpenGeneric: true);
            if (prefix is null) return null;
            if (prefix is SourceImport imported &&
                (imported.Unit.Globals.ContainsKey(member.Name.Text) || imported.Unit.Routines.ContainsKey(member.Name.Text))) return null;
            var owner = ImportedValueType(prefix, expression.Position, allowOpen: true);
            if (owner != TypeSymbol.Unsupported &&
                (references.Property(owner, member.Name.Text, true, recordContext) is not null || references.Field(owner, member.Name.Text, true) is not null ||
                 references.Event(owner, member.Name.Text, source, member.Name.Position) is not null)) return null;
            if (owner != TypeSymbol.Unsupported)
                references.DiagnoseUnsupportedMember(owner, member.Name.Text, true, false, source, member.Name.Position);
        }
        return imports.ContainsKey(path[0].Name.Text) || unit.HasType(path[0].Name.Text) || TypeFacts.Primitive(path[0].Name.Text) != TypeSymbol.Unsupported
            ? catalog.ResolveTypePath(path, unit, references, TypeParameters, inferredArity: arity, allowOpenGeneric: allowOpenGeneric) : null;
    }

    private static IReadOnlyList<TypeSegmentSyntax>? TypePath(ExpressionSyntax expression) => expression switch
    {
        NameExpressionSyntax name => [new(name.Name)],
        MemberSyntax member when TypePath(member.Receiver) is { } prefix => [.. prefix, new(member.Name)],
        GenericNameSyntax generic when TypePath(generic.Name) is { } path => [.. path.SkipLast(1), path[^1] with { Arguments = generic.Arguments }],
        _ => null
    };

    private BoundExpression Receiver(ExpressionSyntax syntax, bool check = true)
    {
        var receiver = BindExpression(syntax, check);
        if (receiver.Type.CanHaveNull && receiver.Type.NeedsNullCheck)
            throw source.Error("FP3013", "The receiver may be null; check a stable local/parameter first or use coalescing.", syntax.Position);
        return receiver;
    }

    private BoundExpression BindMember(MemberSyntax member, bool writing = false, TypeSymbol? expected = null, bool check = true)
    {
        var target = StaticTarget(member.Receiver, allowOpenGeneric: true);
        if (!writing && OpenUnionTarget(target) is { } open && open.Info.Cases.Any(item => item.Name == member.Name.Text))
            return InferUnion(open, member.Name.Text, null, expected, check, member.Position);
        if (target is SourceImport imported)
        {
            if (imported.Unit.Globals.TryGetValue(member.Name.Text, out var global)) return GlobalValue(global, member.Position, writing);
            if (!writing) return SourceDelegate(imported.Unit, member.Name.Text, member.Position, expected);
            throw source.Error("FP3009", "Assignment requires a writable global field.", member.Position);
        }
        BoundExpression? receiver = null;
        var owner = target is null ? (receiver = Receiver(member.Receiver, check)).Type : ImportedValueType(target, member.Position);
        if (owner == TypeSymbol.Unsupported)
            throw source.Error("FP3001", "The selected namespace/unit is not a CLR value member receiver.", member.Position);
        return BindClrMember(owner, receiver, member, writing, expected);
    }

    private BoundExpression BindClrMember(TypeSymbol owner, BoundExpression? receiver, MemberSyntax member, bool writing = false, TypeSymbol? expected = null)
    {
        RejectEventValue(owner, member.Name.Text, member.Name.Position);
        CheckRecordMember(owner, member.Name.Text, writing, member.Position);
        if (owner.Kind == TypeKind.Array && member.Name.Text == "Length" && !writing) return new BoundArrayLength(receiver!);
        if (references.Property(owner, member.Name.Text, receiver is null, recordContext) is { } property)
        {
            if (writing ? property.Setter is null : property.Getter is null)
            {
                references.DiagnoseUnsupportedMember(owner, member.Name.Text, receiver is null, writing, source, member.Name.Position);
                throw source.Error("FP3015", "The requested property accessor is not public or supported.", member.Name.Position);
            }
            if (writing && receiver?.Type.IsValue == true && !Writable(receiver))
                throw source.Error("FP3009", "A value-type property assignment requires writable storage.", member.Position);
            ForgetCapturedFacts();
            return new BoundProperty(writing ? property with { Type = property.Setter!.Parameters[0] } : property, receiver);
        }
        if (references.Field(owner, member.Name.Text, receiver is null) is { } field)
        {
            if (writing && (field.IsReadOnly || receiver?.Type.IsValue == true && !Writable(receiver)))
                throw source.Error("FP3009", "The field or its value-type receiver is read-only.", member.Position);
            return new BoundField(field, receiver);
        }
        references.DiagnoseUnsupportedMember(owner, member.Name.Text, receiver is null, writing, source, member.Name.Position);
        if (!writing) return ImportedDelegate(owner, member.Name.Text, receiver, null, expected, member.Position);
        throw source.Error("FP3001", $"No supported public field/property '{owner}.{member.Name.Text}'.", member.Name.Position);
    }

    private BoundExpression[] ConvertArguments(IReadOnlyList<BoundExpression> arguments, IReadOnlyList<TypeSymbol> types, int position)
        => arguments.Select((argument, index) => Convert(argument, types[index], position)).ToArray();

    private static bool Writable(BoundExpression expression) => expression switch
    {
        BoundLocal local => local.Local.Mutable,
        BoundCapture capture => capture.Capture.Mutable,
        BoundParameter parameter => parameter.Parameter.Mode is "ref" or "out",
        BoundArrayIndex => true,
        BoundField field => !field.Field.IsReadOnly && (field.Receiver is null || field.Receiver.Type.IsReference || Writable(field.Receiver)),
        _ => false
    };
}

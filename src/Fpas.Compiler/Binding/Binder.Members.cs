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
        if (expression is GenericNameSyntax generic && NameParts(generic.Name) is { } typeParts &&
            !HasValue(typeParts[0]))
        {
            var tokens = typeParts.Select(part => new Token(TokenKind.Identifier, part, expression.Position, part.Length)).ToArray();
            return new ClrTypeImport(BindType(new(new(tokens), Arguments: generic.Arguments)));
        }
        var parts = NameParts(expression);
        if (parts is not null && (HasValue(parts[0]))) return null;
        return parts is not null && (imports.ContainsKey(parts[0]) || unit.Records.ContainsKey(parts[0]) || unit.Unions.ContainsKey(parts[0]) || TypeFacts.Primitive(parts[0]) != TypeSymbol.Unsupported)
            ? catalog.ResolvePath(parts, unit, references, expression.Position, arity, allowOpenGeneric) : null;
    }

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
        BoundExpression? receiver = null;
        var owner = target is null ? (receiver = Receiver(member.Receiver, check)).Type : ImportedValueType(target, member.Position);
        if (!writing && target is SourceImport sourceImport) return SourceDelegate(sourceImport.Unit, member.Name.Text, member.Position, expected);
        if (owner == TypeSymbol.Unsupported)
            throw source.Error("FP3001", "The selected namespace/unit is not a CLR value member receiver.", member.Position);
        return BindClrMember(owner, receiver, member, writing, expected);
    }

    private BoundExpression BindClrMember(TypeSymbol owner, BoundExpression? receiver, MemberSyntax member, bool writing = false, TypeSymbol? expected = null)
    {
        CheckRecordMember(owner, member.Name.Text, writing, member.Position);
        if (owner.Kind == TypeKind.Array && member.Name.Text == "Length" && !writing) return new BoundArrayLength(receiver!);
        if (references.Property(owner, member.Name.Text, receiver is null, recordContext) is { } property)
        {
            if (writing ? property.Setter is null : property.Getter is null)
                throw source.Error("FP3015", "The requested property accessor is not public or supported.", member.Name.Position);
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
